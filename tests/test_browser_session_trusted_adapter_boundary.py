import pathlib
import re
import tempfile
import tomllib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
ROOT_CARGO = ROOT / "Cargo.toml"
BROWSER_SESSION_CARGO = ROOT / "crates/originweave-browser-session/Cargo.toml"
THREAT_MODEL = ROOT / "docs/THREAT_MODEL.md"
DOSSIER = ROOT / "docs/traceability/browser-session-trusted-adapter-boundary.md"
BROWSER_SESSION_SOURCE_ROOT = "crates/originweave-browser-session/src/"

# Any production reference to the lifecycle SPI outside the Browser Session owner is an explicit
# review surface. Allow only production surfaces that exist and were reviewed on this exact branch.
APPROVED_PRODUCTION_PORT_REFERENCES: set[str] = set()
APPROVED_BROWSER_SESSION_DEPENDENCIES: set[str] = set()

PORT_REFERENCE = re.compile(r"\bDisposableContextPort\b")
LIFECYCLE_BINDING = re.compile(r"\bbind_lifecycle_port\b")
BROWSER_SESSION_DEPENDENCY = re.compile(r"\boriginweave-browser-session\b")
BROWSER_SESSION_PACKAGE = "originweave-browser-session"


def _has_port_reference(text: str) -> bool:
    return PORT_REFERENCE.search(text) is not None


def _has_lifecycle_binding(text: str) -> bool:
    return LIFECYCLE_BINDING.search(text) is not None


def _has_browser_session_dependency(text: str) -> bool:
    return BROWSER_SESSION_DEPENDENCY.search(text) is not None


def _dependency_package_name(
    dependency_name: str,
    dependency_spec: object,
    workspace_dependencies: dict[str, object],
) -> str:
    if not isinstance(dependency_spec, dict):
        return dependency_name

    package = dependency_spec.get("package")
    if isinstance(package, str):
        return package

    if dependency_spec.get("workspace") is True:
        workspace_spec = workspace_dependencies.get(dependency_name)
        if isinstance(workspace_spec, dict):
            workspace_package = workspace_spec.get("package")
            if isinstance(workspace_package, str):
                return workspace_package
        if workspace_spec is not None:
            return dependency_name

    return dependency_name


def _manifest_dependency_sections(manifest: dict[str, object]) -> list[dict[str, object]]:
    sections: list[dict[str, object]] = []
    dependencies = manifest.get("dependencies")
    if isinstance(dependencies, dict):
        sections.append(dependencies)

    targets = manifest.get("target")
    if isinstance(targets, dict):
        for target in targets.values():
            if not isinstance(target, dict):
                continue
            target_dependencies = target.get("dependencies")
            if isinstance(target_dependencies, dict):
                sections.append(target_dependencies)

    return sections


def _manifest_links_browser_session(member_text: str, workspace_text: str) -> bool:
    member = tomllib.loads(member_text)
    workspace_manifest = tomllib.loads(workspace_text)
    workspace = workspace_manifest.get("workspace")
    workspace_dependencies: dict[str, object] = {}
    if isinstance(workspace, dict):
        declared = workspace.get("dependencies")
        if isinstance(declared, dict):
            workspace_dependencies = declared

    for section in _manifest_dependency_sections(member):
        for dependency_name, dependency_spec in section.items():
            if (
                _dependency_package_name(
                    dependency_name,
                    dependency_spec,
                    workspace_dependencies,
                )
                == BROWSER_SESSION_PACKAGE
            ):
                return True
    return False


def _workspace_member_manifests(root: pathlib.Path) -> list[pathlib.Path]:
    """Return every explicitly reviewed Cargo workspace package manifest."""
    root_manifest_path = root / "Cargo.toml"
    root_manifest = tomllib.loads(root_manifest_path.read_text(encoding="utf-8"))
    workspace = root_manifest.get("workspace")
    if not isinstance(workspace, dict):
        raise AssertionError("repository root must declare a Cargo workspace")
    members = workspace.get("members")
    if not isinstance(members, list):
        raise AssertionError("Cargo workspace members must be an explicit reviewed list")

    root_resolved = root.resolve()
    manifests: set[pathlib.Path] = set()
    if isinstance(root_manifest.get("package"), dict):
        manifests.add(root_manifest_path)

    for member in members:
        if not isinstance(member, str) or not member:
            raise AssertionError("Cargo workspace member paths must be non-empty strings")
        if any(token in member for token in ("*", "?", "[")):
            raise AssertionError(
                "Cargo workspace member globs require an explicit trusted-adapter contract update"
            )
        manifest = (root / member / "Cargo.toml").resolve()
        try:
            manifest.relative_to(root_resolved)
        except ValueError as exc:
            raise AssertionError(
                f"Cargo workspace member escapes repository review root: {member}"
            ) from exc
        if not manifest.is_file():
            raise AssertionError(f"workspace member manifest is missing: {member}/Cargo.toml")
        manifests.add(manifest)
    return sorted(manifests)


def _in_repository_path_dependency(
    root: pathlib.Path,
    manifest: pathlib.Path,
    dependency_name: str,
    dependency_spec: object,
    workspace_dependencies: dict[str, object],
) -> pathlib.Path | None:
    spec = dependency_spec
    base = manifest.parent
    if isinstance(spec, dict) and spec.get("workspace") is True:
        spec = workspace_dependencies.get(dependency_name)
        base = root
    if not isinstance(spec, dict):
        return None

    declared_path = spec.get("path")
    if not isinstance(declared_path, str) or not declared_path:
        return None

    root_resolved = root.resolve()
    candidate = (base / declared_path / "Cargo.toml").resolve()
    try:
        candidate.relative_to(root_resolved)
    except ValueError as exc:
        raise AssertionError(
            f"production Cargo path dependency escapes repository review root: {declared_path}"
        ) from exc
    if not candidate.is_file():
        raise AssertionError(f"production Cargo path dependency manifest is missing: {declared_path}")
    return candidate


def _production_package_manifests(root: pathlib.Path) -> list[pathlib.Path]:
    """Return workspace packages plus recursive in-repository production path dependencies."""
    root_manifest_path = root / "Cargo.toml"
    root_manifest = tomllib.loads(root_manifest_path.read_text(encoding="utf-8"))
    workspace = root_manifest.get("workspace")
    workspace_dependencies: dict[str, object] = {}
    if isinstance(workspace, dict):
        declared = workspace.get("dependencies")
        if isinstance(declared, dict):
            workspace_dependencies = declared

    manifests = set(_workspace_member_manifests(root))
    pending = list(manifests)
    while pending:
        manifest = pending.pop()
        parsed = tomllib.loads(manifest.read_text(encoding="utf-8"))
        for section in _manifest_dependency_sections(parsed):
            for dependency_name, dependency_spec in section.items():
                candidate = _in_repository_path_dependency(
                    root,
                    manifest,
                    dependency_name,
                    dependency_spec,
                    workspace_dependencies,
                )
                if candidate is not None and candidate not in manifests:
                    manifests.add(candidate)
                    pending.append(candidate)
    return sorted(manifests)


def _declared_production_target_sources(
    root: pathlib.Path,
    manifest: pathlib.Path,
) -> set[pathlib.Path]:
    """Return explicitly configured library and binary sources under the repository review root."""
    parsed = tomllib.loads(manifest.read_text(encoding="utf-8"))
    declared_paths: list[str] = []

    library = parsed.get("lib")
    if isinstance(library, dict):
        library_path = library.get("path")
        if isinstance(library_path, str) and library_path:
            declared_paths.append(library_path)

    binaries = parsed.get("bin")
    if isinstance(binaries, list):
        for binary in binaries:
            if not isinstance(binary, dict):
                continue
            binary_path = binary.get("path")
            if isinstance(binary_path, str) and binary_path:
                declared_paths.append(binary_path)

    root_resolved = root.resolve()
    sources: set[pathlib.Path] = set()
    for declared_path in declared_paths:
        candidate = (manifest.parent / declared_path).resolve()
        try:
            candidate.relative_to(root_resolved)
        except ValueError as exc:
            raise AssertionError(
                f"production Cargo target source escapes repository review root: {declared_path}"
            ) from exc
        if not candidate.is_file():
            raise AssertionError(f"declared production Cargo target source is missing: {declared_path}")
        sources.add(candidate)
    return sources


def _workspace_production_sources(root: pathlib.Path) -> list[pathlib.Path]:
    """Return the canonical production Rust source closure reviewed by the TCB contract."""
    root_resolved = root.resolve()
    sources: set[pathlib.Path] = set()
    for manifest in _production_package_manifests(root):
        sources.update(manifest.parent.glob("src/**/*.rs"))
        sources.update(_declared_production_target_sources(root, manifest))

    reviewed: list[pathlib.Path] = []
    for source in sources:
        resolved = source.resolve()
        try:
            resolved.relative_to(root_resolved)
        except ValueError as exc:
            raise AssertionError(
                f"production Cargo source escapes repository review root: {source}"
            ) from exc
        if not resolved.is_file():
            raise AssertionError(f"production Cargo source is missing: {source}")
        reviewed.append(source)
    return sorted(reviewed)


class BrowserSessionTrustedAdapterBoundaryTests(unittest.TestCase):
    """Keep the privileged lifecycle adapter inside the reviewed product TCB."""

    def test_browser_session_crate_is_internal_and_zone_c_is_trusted(self) -> None:
        cargo = BROWSER_SESSION_CARGO.read_text(encoding="utf-8")
        threat_model = THREAT_MODEL.read_text(encoding="utf-8")

        self.assertRegex(cargo, r"(?m)^publish\s*=\s*false\s*$")
        self.assertIn(
            "Zone C — Chromium browser process and privileged adapters",
            threat_model,
        )
        self.assertIn("trusted browser integration code", threat_model)

    def test_trusted_adapter_dossier_states_the_supported_security_boundary(self) -> None:
        dossier = DOSSIER.read_text(encoding="utf-8")

        for required in (
            "trusted computing base",
            "DisposableContextPort",
            "publish = false",
            "supply-chain compromise",
            "caller-selected production adapter",
            "request/completion correlation is not adapter authentication",
        ):
            self.assertIn(required, dossier)

    def test_production_disposable_context_port_references_are_allowlisted(self) -> None:
        discovered = set()
        for path in _workspace_production_sources(ROOT):
            relative = path.relative_to(ROOT).as_posix()
            if relative.startswith(BROWSER_SESSION_SOURCE_ROOT):
                continue
            text = path.read_text(encoding="utf-8")
            if _has_port_reference(text):
                discovered.add(relative)

        unexpected = discovered - APPROVED_PRODUCTION_PORT_REFERENCES
        self.assertEqual(
            unexpected,
            set(),
            f"unreviewed production lifecycle-port references: {sorted(unexpected)}",
        )

    def test_browser_session_dependencies_are_allowlisted(self) -> None:
        discovered = set()
        workspace_text = ROOT_CARGO.read_text(encoding="utf-8")
        for path in _production_package_manifests(ROOT):
            if path == BROWSER_SESSION_CARGO:
                continue
            text = path.read_text(encoding="utf-8")
            if _manifest_links_browser_session(text, workspace_text):
                discovered.add(path.relative_to(ROOT).as_posix())

        unexpected = discovered - APPROVED_BROWSER_SESSION_DEPENDENCIES
        self.assertEqual(
            unexpected,
            set(),
            f"unreviewed production Browser Session dependencies: {sorted(unexpected)}",
        )

    def test_allowlists_do_not_preapprove_absent_production_surfaces(self) -> None:
        discovered_port_references = set()
        for path in _workspace_production_sources(ROOT):
            relative = path.relative_to(ROOT).as_posix()
            if relative.startswith(BROWSER_SESSION_SOURCE_ROOT):
                continue
            if _has_port_reference(path.read_text(encoding="utf-8")):
                discovered_port_references.add(relative)

        discovered_dependencies = set()
        workspace_text = ROOT_CARGO.read_text(encoding="utf-8")
        for path in _production_package_manifests(ROOT):
            if path == BROWSER_SESSION_CARGO:
                continue
            if _manifest_links_browser_session(path.read_text(encoding="utf-8"), workspace_text):
                discovered_dependencies.add(path.relative_to(ROOT).as_posix())

        self.assertEqual(
            APPROVED_PRODUCTION_PORT_REFERENCES,
            discovered_port_references,
            "adapter source allowlist must describe current production references, not reserve future paths",
        )
        self.assertEqual(
            APPROVED_BROWSER_SESSION_DEPENDENCIES,
            discovered_dependencies,
            "dependency allowlist must describe current production links, not reserve future crates",
        )

    def test_product_sources_do_not_bind_a_caller_selected_lifecycle_port(self) -> None:
        callers = set()
        for path in _workspace_production_sources(ROOT):
            if path == ROOT / "crates/originweave-browser-session/src/browser_session.rs":
                continue
            text = path.read_text(encoding="utf-8")
            if _has_lifecycle_binding(text):
                callers.add(path.relative_to(ROOT).as_posix())

        self.assertEqual(
            callers,
            set(),
            "product composition must gain an explicit reviewed owner before binding a lifecycle port",
        )

    def test_scanners_cover_qualified_alias_and_ufcs_spellings(self) -> None:
        port_spellings = (
            "impl originweave_browser_session::DisposableContextPort for CandidatePort {}",
            (
                "use originweave_browser_session::DisposableContextPort as LifecyclePort;\n"
                "impl LifecyclePort for CandidatePort {}"
            ),
        )
        for source in port_spellings:
            self.assertTrue(
                _has_port_reference(source),
                f"production port spelling escaped review scanner: {source!r}",
            )

        binding_spellings = (
            "session.bind_lifecycle_port(port);",
            "BrowserSession::bind_lifecycle_port(session, port);",
            "session.bind_lifecycle_port (port);",
        )
        for source in binding_spellings:
            self.assertTrue(
                _has_lifecycle_binding(source),
                f"production lifecycle binding escaped review scanner: {source!r}",
            )

    def test_cross_file_alias_cannot_escape_dependency_review_surface(self) -> None:
        alias_only_source = "impl LifecyclePort for CandidatePort {}"
        dependency_manifests = (
            "[dependencies]\n"
            'originweave-browser-session = { path = "../originweave-browser-session" }\n',
            "[dependencies]\n"
            'browser = { package = "originweave-browser-session", path = "../originweave-browser-session" }\n',
            "[dependencies.originweave-browser-session]\n"
            'path = "../originweave-browser-session"\n',
        )

        self.assertFalse(_has_port_reference(alias_only_source))
        for manifest in dependency_manifests:
            self.assertTrue(
                _has_browser_session_dependency(manifest),
                f"Browser Session dependency spelling escaped review scanner: {manifest!r}",
            )

    def test_workspace_dependency_alias_cannot_escape_dependency_review_surface(self) -> None:
        workspace_manifest = (
            "[workspace.dependencies]\n"
            'browser_session = { package = "originweave-browser-session", path = "crates/originweave-browser-session" }\n'
        )
        member_manifests = (
            "[dependencies]\n"
            "browser_session = { workspace = true }\n",
            "[target.'cfg(unix)'.dependencies]\n"
            "browser_session = { workspace = true }\n",
        )

        for member_manifest in member_manifests:
            self.assertTrue(
                _manifest_links_browser_session(member_manifest, workspace_manifest),
                "workspace dependency aliases must remain an explicit Browser Session TCB review surface",
            )

    def test_workspace_member_outside_crates_glob_cannot_escape_review_surface(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["plugins/browser-adapter"]\n',
                encoding="utf-8",
            )
            member = root / "plugins/browser-adapter"
            member.mkdir(parents=True)
            member_manifest = member / "Cargo.toml"
            member_manifest.write_text(
                '[package]\nname = "browser-adapter"\nversion = "0.1.0"\n'
                '[dependencies]\noriginweave-browser-session = { path = "../../crates/originweave-browser-session" }\n',
                encoding="utf-8",
            )
            source = member / "src/lib.rs"
            source.parent.mkdir()
            source.write_text(
                "use originweave_browser_session::DisposableContextPort;\n",
                encoding="utf-8",
            )

            self.assertIn(member_manifest, _production_package_manifests(root))
            self.assertIn(source, _workspace_production_sources(root))

    def test_workspace_root_package_cannot_escape_review_surface(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            root_manifest = root / "Cargo.toml"
            root_manifest.write_text(
                '[package]\nname = "root-browser-adapter"\nversion = "0.1.0"\nedition = "2024"\n'
                '[workspace]\nmembers = []\n'
                '[dependencies]\noriginweave-browser-session = { path = "crates/originweave-browser-session" }\n',
                encoding="utf-8",
            )
            source = root / "src/lib.rs"
            source.parent.mkdir()
            source.write_text(
                "use originweave_browser_session::DisposableContextPort;\n",
                encoding="utf-8",
            )

            self.assertIn(root_manifest, _production_package_manifests(root))
            self.assertIn(source, _workspace_production_sources(root))

    def test_workspace_member_outside_repository_root_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            sandbox = pathlib.Path(directory)
            root = sandbox / "workspace"
            external = sandbox / "external-browser-adapter"
            root.mkdir()
            external.mkdir()
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["../external-browser-adapter"]\n',
                encoding="utf-8",
            )
            (external / "Cargo.toml").write_text(
                '[package]\nname = "external-browser-adapter"\nversion = "0.1.0"\n',
                encoding="utf-8",
            )

            with self.assertRaisesRegex(AssertionError, "member escapes repository review root"):
                _workspace_member_manifests(root)

    def test_workspace_member_globs_fail_closed_until_reviewed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["plugins/*"]\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(AssertionError, "member globs require"):
                _workspace_member_manifests(root)


if __name__ == "__main__":
    unittest.main()
