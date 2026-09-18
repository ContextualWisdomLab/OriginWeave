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


class BrowserSessionCargoEnvironmentAuthorityContractTests(unittest.TestCase):
    """Keep Git-owned Cargo compiler environment inside reviewed provenance."""

    def test_repository_cargo_environment_inputs_fail_closed(self) -> None:
        hostile_configs = (
            '[env]\nORIGINWEAVE_BUILD_ID = "unreviewed"\n',
            '[env]\nORIGINWEAVE_BUILD_ID = { value = "unreviewed", force = true }\n',
            '[env]\nORIGINWEAVE_TOOL_ROOT = { value = "tools", relative = true, force = true }\n',
        )
        for config_text in hostile_configs:
            with self.subTest(config_text=config_text):
                root = self._workspace_with_config(config_text)
                with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
                    authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_empty_cargo_environment_table_remains_allowed(self) -> None:
        root = self._workspace_with_config("[env]\n")
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def _workspace_with_config(self, config_text: str) -> pathlib.Path:
        directory = self.enterContext(tempfile.TemporaryDirectory())
        root = pathlib.Path(directory)
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
        cargo = root / ".cargo"
        cargo.mkdir()
        (cargo / "config.toml").write_text(config_text, encoding="utf-8")
        return root


if __name__ == "__main__":
    unittest.main()
