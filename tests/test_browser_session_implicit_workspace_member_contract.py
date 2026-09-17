import importlib.util
import pathlib
import tempfile
import tomllib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BOUNDARY_TEST = ROOT / "tests/test_browser_session_trusted_adapter_boundary.py"

spec = importlib.util.spec_from_file_location("browser_session_trusted_adapter_boundary", BOUNDARY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session trusted-adapter boundary contract")
boundary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boundary)


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
    except ValueError:
        return None
    return candidate if candidate.is_file() else None


def _production_package_manifests(root: pathlib.Path) -> list[pathlib.Path]:
    """Return reviewed workspace packages plus recursive in-repository path dependencies."""
    root_manifest_path = root / "Cargo.toml"
    root_manifest = tomllib.loads(root_manifest_path.read_text(encoding="utf-8"))
    workspace = root_manifest.get("workspace")
    workspace_dependencies: dict[str, object] = {}
    if isinstance(workspace, dict):
        declared = workspace.get("dependencies")
        if isinstance(declared, dict):
            workspace_dependencies = declared

    manifests = set(boundary._workspace_member_manifests(root))
    pending = list(manifests)
    while pending:
        manifest = pending.pop()
        parsed = tomllib.loads(manifest.read_text(encoding="utf-8"))
        for section in boundary._manifest_dependency_sections(parsed):
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


def _production_sources(root: pathlib.Path) -> list[pathlib.Path]:
    sources: list[pathlib.Path] = []
    for manifest in _production_package_manifests(root):
        sources.extend(sorted(manifest.parent.glob("src/**/*.rs")))
    return sources


class BrowserSessionImplicitWorkspaceMemberContractTests(unittest.TestCase):
    """Cover Cargo's automatic in-workspace path-dependency membership semantics."""

    def test_in_workspace_path_dependency_cannot_escape_review_surface(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["app"]\nresolver = "3"\n',
                encoding="utf-8",
            )

            app = root / "app"
            app.mkdir()
            (app / "Cargo.toml").write_text(
                '[package]\nname = "app"\nversion = "0.1.0"\nedition = "2024"\n'
                '[dependencies]\nbrowser-adapter = { path = "../plugins/browser-adapter" }\n',
                encoding="utf-8",
            )
            (app / "src").mkdir()
            (app / "src/lib.rs").write_text("pub fn app() {}\n", encoding="utf-8")

            adapter = root / "plugins/browser-adapter"
            adapter.mkdir(parents=True)
            adapter_manifest = adapter / "Cargo.toml"
            adapter_manifest.write_text(
                '[package]\nname = "browser-adapter"\nversion = "0.1.0"\nedition = "2024"\n'
                '[dependencies]\noriginweave-browser-session = { path = "../../crates/originweave-browser-session" }\n',
                encoding="utf-8",
            )
            adapter_source = adapter / "src/lib.rs"
            adapter_source.parent.mkdir()
            adapter_source.write_text(
                "use originweave_browser_session::DisposableContextPort;\n",
                encoding="utf-8",
            )

            self.assertIn(adapter_manifest, _production_package_manifests(root))
            self.assertIn(adapter_source, _production_sources(root))

    def test_workspace_inherited_path_dependency_cannot_escape_review_surface(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["app"]\nresolver = "3"\n'
                '[workspace.dependencies]\nbrowser_adapter = { path = "plugins/browser-adapter" }\n',
                encoding="utf-8",
            )

            app = root / "app"
            app.mkdir()
            (app / "Cargo.toml").write_text(
                '[package]\nname = "app"\nversion = "0.1.0"\nedition = "2024"\n'
                '[target.\'cfg(unix)\'.dependencies]\nbrowser_adapter = { workspace = true }\n',
                encoding="utf-8",
            )
            (app / "src").mkdir()
            (app / "src/lib.rs").write_text("pub fn app() {}\n", encoding="utf-8")

            adapter = root / "plugins/browser-adapter"
            adapter.mkdir(parents=True)
            adapter_manifest = adapter / "Cargo.toml"
            adapter_manifest.write_text(
                '[package]\nname = "browser-adapter"\nversion = "0.1.0"\nedition = "2024"\n'
                '[dependencies]\noriginweave-browser-session = { path = "../../crates/originweave-browser-session" }\n',
                encoding="utf-8",
            )
            adapter_source = adapter / "src/lib.rs"
            adapter_source.parent.mkdir()
            adapter_source.write_text(
                "use originweave_browser_session::DisposableContextPort;\n",
                encoding="utf-8",
            )

            self.assertIn(adapter_manifest, _production_package_manifests(root))
            self.assertIn(adapter_source, _production_sources(root))

    def test_recursive_local_path_dependencies_obey_existing_tcb_allowlists(self) -> None:
        workspace_text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")

        discovered_port_references = set()
        for path in _production_sources(ROOT):
            relative = path.relative_to(ROOT).as_posix()
            if relative.startswith(boundary.BROWSER_SESSION_SOURCE_ROOT):
                continue
            if boundary._has_port_reference(path.read_text(encoding="utf-8")):
                discovered_port_references.add(relative)

        discovered_dependencies = set()
        for path in _production_package_manifests(ROOT):
            if path == boundary.BROWSER_SESSION_CARGO:
                continue
            if boundary._manifest_links_browser_session(
                path.read_text(encoding="utf-8"),
                workspace_text,
            ):
                discovered_dependencies.add(path.relative_to(ROOT).as_posix())

        self.assertEqual(
            boundary.APPROVED_PRODUCTION_PORT_REFERENCES,
            discovered_port_references,
            "recursive local path dependencies must not add unreviewed lifecycle-port source",
        )
        self.assertEqual(
            boundary.APPROVED_BROWSER_SESSION_DEPENDENCIES,
            discovered_dependencies,
            "recursive local path dependencies must not link Browser Session outside the reviewed allowlist",
        )


if __name__ == "__main__":
    unittest.main()
