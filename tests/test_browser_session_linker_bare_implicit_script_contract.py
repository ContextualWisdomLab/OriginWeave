import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location("browser_session_cargo_compiler_authority", AUTHORITY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Cargo compiler authority contract")
authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(authority)


class BrowserSessionLinkerBareImplicitScriptContractTests(unittest.TestCase):
    """Keep bare extensionless implicit linker inputs inside reviewed provenance."""

    def _workspace_with_config(self, config_text: str) -> pathlib.Path:
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
        (root / "review-bypass-input").write_text(
            "INPUT(tools/review-bypass-object.o)\n",
            encoding="utf-8",
        )
        cargo = root / ".cargo"
        cargo.mkdir()
        (cargo / "config.toml").write_text(config_text, encoding="utf-8")
        return root

    def _assert_bare_input_fails_closed(self, config_text: str) -> None:
        root = self._workspace_with_config(config_text)
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_direct_bare_extensionless_implicit_script_fails_closed(self) -> None:
        self._assert_bare_input_fails_closed(
            '[build]\nrustflags = ["-C", "link-arg=review-bypass-input"]\n'
        )

    def test_forwarded_bare_extensionless_implicit_script_fails_closed(self) -> None:
        for forwarded in (
            "-Wl,review-bypass-input",
            "--for-linker=review-bypass-input",
            "-Xlinker review-bypass-input",
        ):
            with self.subTest(forwarded=forwarded):
                self._assert_bare_input_fails_closed(
                    f'[build]\nrustflags = ["-C", "link-args={forwarded}"]\n'
                )

    def test_separate_z_operand_remains_allowed(self) -> None:
        for forwarded in (
            "-z relro",
            "-Wl,-z,relro",
            "--for-linker=-z,relro",
            "-Xlinker -z -Xlinker relro",
        ):
            with self.subTest(forwarded=forwarded):
                root = self._workspace_with_config(
                    f'[build]\nrustflags = ["-C", "link-args={forwarded}"]\n'
                )
                authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
