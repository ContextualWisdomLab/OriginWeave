import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE_INDIRECTION_TEST = ROOT / "tests/test_browser_session_rust_source_indirection_contract.py"

spec = importlib.util.spec_from_file_location(
    "browser_session_rust_source_indirection_contract",
    SOURCE_INDIRECTION_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Rust source-indirection contract")
source_indirection = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source_indirection)


class BrowserSessionRustIncludeLexicalControlContractTests(unittest.TestCase):
    """Keep Rust include! authority lexical rather than matching comment or literal text."""

    def _workspace_with_source(self, source_text: str) -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n',
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "src").mkdir(parents=True)
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        (adapter / "src/lib.rs").write_text(source_text, encoding="utf-8")
        (adapter / "generated_adapter.rs").write_text(
            "pub fn generated_adapter_surface() {}\n",
            encoding="utf-8",
        )
        return root

    def test_commented_include_macro_text_is_not_source_indirection(self) -> None:
        root = self._workspace_with_source(
            '// include!("../generated_adapter.rs");\n'
            'pub fn reviewed_surface() {}\n'
        )

        source_indirection._assert_no_unmodeled_rust_source_indirection(root)

    def test_string_include_macro_text_is_not_source_indirection(self) -> None:
        root = self._workspace_with_source(
            'pub const NOTE: &str = "include!(\\\"../generated_adapter.rs\\\")";\n'
        )

        source_indirection._assert_no_unmodeled_rust_source_indirection(root)

    def test_real_include_macro_still_fails_closed(self) -> None:
        root = self._workspace_with_source('include!("../generated_adapter.rs");\n')

        with self.assertRaisesRegex(AssertionError, "Rust include! source indirection"):
            source_indirection._assert_no_unmodeled_rust_source_indirection(root)


if __name__ == "__main__":
    unittest.main()
