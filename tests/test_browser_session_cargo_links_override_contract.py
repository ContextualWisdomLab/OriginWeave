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


class BrowserSessionCargoLinksOverrideContractTests(unittest.TestCase):
    """Keep Cargo links build-script overrides inside reviewed compiler provenance."""

    def test_target_links_build_script_override_fails_closed(self) -> None:
        hostile_configs = (
            '[target.x86_64-unknown-linux-gnu.review_bypass]\nrustc-link-lib = ["review_bypass"]\n',
            '[target.x86_64-unknown-linux-gnu.review_bypass]\nrustc-link-search = ["tools/native"]\n',
            '[target.x86_64-unknown-linux-gnu.review_bypass]\nrustc-cfg = ["originweave_review_bypass"]\n',
            '[target.x86_64-unknown-linux-gnu.review_bypass]\nrustc-env = { ORIGINWEAVE_BUILD_ID = "unreviewed" }\n',
            '[target.x86_64-unknown-linux-gnu.review_bypass]\nrustc-cdylib-link-arg = ["tools/review-bypass.o"]\n',
        )
        for config_text in hostile_configs:
            with self.subTest(config_text=config_text):
                root = self._workspace_with_config(config_text)
                with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
                    authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_ordinary_target_table_without_links_override_remains_allowed(self) -> None:
        root = self._workspace_with_config(
            '[target.x86_64-unknown-linux-gnu]\nrustflags = ["--cfg", "originweave_reviewed"]\n'
        )
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
