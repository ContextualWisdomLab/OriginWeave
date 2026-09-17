import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location("browser_session_cargo_compiler_authority", AUTHORITY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Cargo compiler-authority contract")
authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(authority)


class BrowserSessionLinkerScriptProvenanceContractTests(unittest.TestCase):
    """Keep GNU linker-script input authority inside the reviewed Browser Session build boundary."""

    def _workspace_with_flags(self, rustflags: str) -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n',
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "src").mkdir(parents=True)
        (adapter / "src/lib.rs").write_text("pub fn adapter_surface() {}\n", encoding="utf-8")
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        tools = root / "tools"
        tools.mkdir()
        (tools / "review-bypass.ld").write_text(
            "INPUT(tools/review-bypass-object.o)\n",
            encoding="utf-8",
        )
        cargo = root / ".cargo"
        cargo.mkdir()
        (cargo / "config.toml").write_text(
            f'[build]\nrustflags = {rustflags}\n',
            encoding="utf-8",
        )
        return root

    def test_repository_driver_linker_script_fails_closed(self) -> None:
        root = self._workspace_with_flags(
            '["-C", "link-arg=-Ttools/review-bypass.ld"]'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_forwarded_linker_script_fails_closed(self) -> None:
        root = self._workspace_with_flags(
            '["-C", "link-arg=-Wl,--script=tools/review-bypass.ld"]'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_xlinker_script_fails_closed(self) -> None:
        root = self._workspace_with_flags(
            '["-C", "link-args=-Xlinker -T -Xlinker tools/review-bypass.ld"]'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_unrelated_driver_link_argument_remains_allowed(self) -> None:
        root = self._workspace_with_flags('["-C", "link-arg=-pthread"]')
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
