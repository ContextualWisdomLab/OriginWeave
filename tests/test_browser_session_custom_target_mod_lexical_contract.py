import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
INDIRECTION_TEST = ROOT / "tests/test_browser_session_rust_source_indirection_contract.py"

spec = importlib.util.spec_from_file_location(
    "browser_session_rust_source_indirection_contract",
    INDIRECTION_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Rust source-indirection contract")
indirection = importlib.util.module_from_spec(spec)
spec.loader.exec_module(indirection)


class BrowserSessionCustomTargetModLexicalContractTests(unittest.TestCase):
    """Keep custom-target module detection lexical instead of raw-text based."""

    def _custom_target_workspace(self, source_text: str) -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n',
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "runtime").mkdir(parents=True)
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n'
            '[lib]\npath = "runtime/lifecycle_adapter.rs"\n',
            encoding="utf-8",
        )
        (adapter / "runtime/lifecycle_adapter.rs").write_text(source_text, encoding="utf-8")
        return root

    def test_line_comment_mod_text_is_lexical_data(self) -> None:
        root = self._custom_target_workspace(
            '// mod hidden;\npub fn lifecycle_adapter_surface() {}\n'
        )

        indirection._assert_no_unmodeled_rust_source_indirection(root)

    def test_nested_block_comment_mod_text_is_lexical_data(self) -> None:
        root = self._custom_target_workspace(
            '/* outer /* mod hidden; */ still comment */\n'
            'pub fn lifecycle_adapter_surface() {}\n'
        )

        indirection._assert_no_unmodeled_rust_source_indirection(root)

    def test_ordinary_string_mod_text_is_lexical_data(self) -> None:
        root = self._custom_target_workspace(
            'pub const NOTE: &str = "mod hidden;";\npub fn lifecycle_adapter_surface() {}\n'
        )

        indirection._assert_no_unmodeled_rust_source_indirection(root)

    def test_raw_string_mod_text_is_lexical_data(self) -> None:
        root = self._custom_target_workspace(
            'pub const NOTE: &str = r#"mod hidden;"#;\npub fn lifecycle_adapter_surface() {}\n'
        )

        indirection._assert_no_unmodeled_rust_source_indirection(root)

    def test_character_literal_does_not_hide_following_real_mod(self) -> None:
        root = self._custom_target_workspace(
            "pub const MARKER: char = 'm';\nmod helper;\n"
        )

        with self.assertRaisesRegex(AssertionError, "Rust module source indirection"):
            indirection._assert_no_unmodeled_rust_source_indirection(root)


if __name__ == "__main__":
    unittest.main()
