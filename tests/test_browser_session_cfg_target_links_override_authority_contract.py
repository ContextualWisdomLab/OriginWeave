import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location(
    "browser_session_cargo_compiler_authority_contract",
    AUTHORITY_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Cargo compiler-authority contract")
authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(authority)


class BrowserSessionCfgTargetLinksOverrideAuthorityContractTests(unittest.TestCase):
    """Keep Cargo cfg-target warnings separate from tuple links override authority."""

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
        cargo = root / ".cargo"
        cargo.mkdir()
        (cargo / "config.toml").write_text(config_text, encoding="utf-8")
        return root

    def test_cfg_target_unknown_nested_table_is_not_links_override_authority(self) -> None:
        root = self._workspace_with_config(
            "[target.'cfg(unix)'.review_bypass]\n"
            'rustc-link-search = ["tools/not-a-target-links-override"]\n'
        )
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_tuple_target_links_override_remains_authority(self) -> None:
        root = self._workspace_with_config(
            "[target.x86_64-unknown-linux-gnu.review_bypass]\n"
            'rustc-link-search = ["tools/review-bypass-native"]\n'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
