import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BOUNDARY_TEST = ROOT / "tests/test_browser_session_trusted_adapter_boundary.py"

spec = importlib.util.spec_from_file_location("browser_session_trusted_adapter_boundary", BOUNDARY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session trusted-adapter boundary contract")
boundary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boundary)

# The trusted-adapter boundary is the single writer for production Cargo topology.
# These aliases keep the hostile topology fixtures on that exact scanner rather than
# maintaining a second implementation that can drift away from the security gate.
_production_package_manifests = boundary._production_package_manifests
_production_sources = boundary._workspace_production_sources


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

    def test_recursive_path_dependency_enters_canonical_binding_review_surface(self) -> None:
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
            (adapter / "Cargo.toml").write_text(
                '[package]\nname = "browser-adapter"\nversion = "0.1.0"\nedition = "2024"\n',
                encoding="utf-8",
            )
            adapter_source = adapter / "src/lib.rs"
            adapter_source.parent.mkdir()
            adapter_source.write_text(
                "pub fn attach(session: &mut Session, port: Port) { session.bind_lifecycle_port(port); }\n",
                encoding="utf-8",
            )

            self.assertIn(adapter_source, _production_sources(root))
            self.assertTrue(boundary._has_lifecycle_binding(adapter_source.read_text(encoding="utf-8")))
            self.assertIn(
                adapter_source,
                boundary._workspace_production_sources(root),
                "canonical lifecycle-binding review must include recursive in-repository path dependencies",
            )

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
