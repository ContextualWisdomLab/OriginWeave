import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
IMPLICIT_WORKSPACE_CONTRACT = ROOT / "tests/test_browser_session_implicit_workspace_member_contract.py"

spec = importlib.util.spec_from_file_location(
    "browser_session_implicit_workspace_member_contract",
    IMPLICIT_WORKSPACE_CONTRACT,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session implicit-workspace contract")
implicit_workspace = importlib.util.module_from_spec(spec)
spec.loader.exec_module(implicit_workspace)


class BrowserSessionCustomTargetSourceContractTests(unittest.TestCase):
    """Keep non-standard Cargo production target paths inside the TCB review surface."""

    def test_custom_lib_and_bin_paths_cannot_escape_production_source_review(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["plugins/browser-adapter"]\nresolver = "3"\n',
                encoding="utf-8",
            )

            adapter = root / "plugins/browser-adapter"
            adapter.mkdir(parents=True)
            (adapter / "Cargo.toml").write_text(
                '[package]\nname = "browser-adapter"\nversion = "0.1.0"\nedition = "2024"\n'
                '[lib]\npath = "runtime/lifecycle_adapter.rs"\n'
                '[[bin]]\nname = "browser-adapter-cli"\npath = "command/adapter_cli.rs"\n',
                encoding="utf-8",
            )

            library_source = adapter / "runtime/lifecycle_adapter.rs"
            library_source.parent.mkdir()
            library_source.write_text("pub fn lifecycle_adapter() {}\n", encoding="utf-8")
            binary_source = adapter / "command/adapter_cli.rs"
            binary_source.parent.mkdir()
            binary_source.write_text("fn main() {}\n", encoding="utf-8")

            production_sources = implicit_workspace._production_sources(root)
            self.assertIn(library_source, production_sources)
            self.assertIn(binary_source, production_sources)


if __name__ == "__main__":
    unittest.main()
