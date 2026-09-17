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


class BrowserSessionImplicitWorkspaceMemberContractTests(unittest.TestCase):
    """Match Cargo's automatic in-workspace path-dependency membership semantics."""

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

            self.assertIn(adapter_manifest, boundary._workspace_member_manifests(root))
            self.assertIn(adapter_source, boundary._workspace_production_sources(root))


if __name__ == "__main__":
    unittest.main()
