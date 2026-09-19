import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE_CONTRACT = ROOT / "tests/test_browser_session_rust_source_indirection_contract.py"

spec = importlib.util.spec_from_file_location("browser_session_rust_source_indirection", SOURCE_CONTRACT)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Rust source-indirection contract")
source_contract = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source_contract)


class BrowserSessionIncludeCommentTriviaContractTests(unittest.TestCase):
    """Keep Rust lexical trivia from bypassing include! source-provenance review."""

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

    def _assert_include_trivia_fails_closed(self, source_text: str) -> None:
        root = self._workspace_with_source(source_text)
        with self.assertRaisesRegex(AssertionError, "Rust include! source indirection"):
            source_contract._assert_no_unmodeled_rust_source_indirection(root)

    def test_block_comment_between_include_and_bang_fails_closed(self) -> None:
        self._assert_include_trivia_fails_closed(
            'include /* provenance gap */ ! ("../generated_adapter.rs");\n'
        )

    def test_block_comment_between_bang_and_delimiter_fails_closed(self) -> None:
        self._assert_include_trivia_fails_closed(
            'include! /* provenance gap */ ("../generated_adapter.rs");\n'
        )

    def test_line_comment_between_include_and_bang_fails_closed(self) -> None:
        self._assert_include_trivia_fails_closed(
            'include // provenance gap\n! ("../generated_adapter.rs");\n'
        )

    def test_non_ascii_rust_whitespace_between_include_and_bang_fails_closed(self) -> None:
        self._assert_include_trivia_fails_closed(
            'include\u200e!("../generated_adapter.rs");\n'
        )

    def test_non_ascii_rust_whitespace_between_bang_and_delimiter_fails_closed(self) -> None:
        self._assert_include_trivia_fails_closed(
            'include!\u200f("../generated_adapter.rs");\n'
        )


if __name__ == "__main__":
    unittest.main()
