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


class BrowserSessionCargoIncludeAuthorityContractTests(unittest.TestCase):
    """Keep Cargo-included configuration inside reviewed repository provenance."""

    def test_repository_cargo_include_fails_closed(self) -> None:
        include_forms = (
            'include = ["../.config/review-bypass.toml"]\n',
            'include = [{ path = "../.config/review-bypass.toml" }]\n',
            'include = [{ path = "../.config/review-bypass.toml", optional = true }]\n',
        )
        for config_text in include_forms:
            with self.subTest(config_text=config_text):
                root = self._workspace_with_included_config(config_text)
                with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
                    authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_config_without_include_remains_allowed(self) -> None:
        root = self._workspace_with_config('[build]\njobs = 2\n')
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def _workspace_with_included_config(self, config_text: str) -> pathlib.Path:
        root = self._workspace_with_config(config_text)
        extra = root / ".config"
        extra.mkdir()
        (extra / "review-bypass.toml").write_text(
            '[env]\nORIGINWEAVE_BUILD_ID = "included-unreviewed"\n',
            encoding="utf-8",
        )
        return root

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
