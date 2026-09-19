import importlib.util
import pathlib
import re
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE_INDIRECTION_TEST = ROOT / "tests/test_browser_session_rust_source_indirection_contract.py"
EMBEDDED_FILE_MACRO_TOKEN = re.compile(
    r"(?<![\w#])(?:r#)?(?:include_bytes|include_str)(?!\w)",
    re.UNICODE,
)

spec = importlib.util.spec_from_file_location(
    "browser_session_rust_source_indirection_contract",
    SOURCE_INDIRECTION_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Rust source-indirection contract")
source_indirection = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source_indirection)


def _has_embedded_file_macro(text: str) -> bool:
    """Detect compile-time file embedding outside Rust comments and literals."""
    cursor = 0
    while cursor < len(text):
        trivia_end = source_indirection._skip_rust_trivia(text, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        raw_end = source_indirection._raw_string_end(text, cursor)
        if raw_end is not None:
            cursor = raw_end
            continue
        if text[cursor] == '"':
            cursor = source_indirection._quoted_string_end(text, cursor)
            continue
        if text[cursor] == "'":
            char_end = source_indirection._simple_char_literal_end(text, cursor)
            if char_end is not None:
                cursor = char_end
                continue

        match = EMBEDDED_FILE_MACRO_TOKEN.match(text, cursor)
        if match is not None:
            bang = source_indirection._skip_rust_trivia(text, match.end())
            if bang < len(text) and text[bang] == "!":
                return True
            cursor = match.end()
            continue
        cursor += 1
    return False


def _use_tree_aliases_embedded_file_macro(use_tree: str) -> bool:
    """Return whether one Rust use tree gives include_bytes!/include_str! a callable alias."""
    cursor = 0
    while cursor < len(use_tree):
        trivia_end = source_indirection._skip_rust_trivia(use_tree, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        match = EMBEDDED_FILE_MACRO_TOKEN.match(use_tree, cursor)
        if match is None:
            cursor += 1
            continue

        after_macro = source_indirection._skip_rust_trivia(use_tree, match.end())
        as_match = source_indirection.AS_TOKEN.match(use_tree, after_macro)
        if as_match is not None:
            alias_start = source_indirection._skip_rust_trivia(use_tree, as_match.end())
            if alias_start >= len(use_tree):
                return False
            if use_tree[alias_start] == "_":
                next_offset = alias_start + 1
                if next_offset >= len(use_tree) or not source_indirection._rust_keyword_is_identifier_adjacent(
                    use_tree[next_offset]
                ):
                    cursor = match.end()
                    continue
            return True
        cursor = match.end()
    return False


def _has_aliased_embedded_file_import(text: str) -> bool:
    """Detect lexical use aliases that would hide embedded-file macro names at invocation."""
    cursor = 0
    while cursor < len(text):
        trivia_end = source_indirection._skip_rust_trivia(text, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        raw_end = source_indirection._raw_string_end(text, cursor)
        if raw_end is not None:
            cursor = raw_end
            continue
        if text[cursor] == '"':
            cursor = source_indirection._quoted_string_end(text, cursor)
            continue
        if text[cursor] == "'":
            char_end = source_indirection._simple_char_literal_end(text, cursor)
            if char_end is not None:
                cursor = char_end
                continue

        use_match = source_indirection.USE_TOKEN.match(text, cursor)
        if use_match is None:
            cursor += 1
            continue

        statement_end = source_indirection._rust_use_statement_end(text, use_match.end())
        if statement_end is None:
            return False
        if _use_tree_aliases_embedded_file_macro(text[use_match.end():statement_end]):
            return True
        cursor = statement_end + 1
    return False


def _assert_no_unmodeled_rust_embedded_file_inputs(root: pathlib.Path) -> None:
    """Fail closed when reviewed Rust source embeds file bytes outside the source closure."""
    source_indirection._assert_no_unmodeled_rust_source_indirection(root)
    for source in source_indirection.boundary._workspace_production_sources(root):
        text = source.read_text(encoding="utf-8")
        if _has_embedded_file_macro(text) or _has_aliased_embedded_file_import(text):
            relative = source.relative_to(root).as_posix()
            raise AssertionError(
                "Rust embedded file input requires an explicit provenance contract: "
                f"{relative}"
            )


class BrowserSessionRustEmbeddedFileInputAuthorityContractTests(unittest.TestCase):
    """Keep compile-time embedded files inside explicit Browser Session source provenance."""

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
        (adapter / "unreviewed.bin").write_bytes(b"unreviewed-browser-runtime-bytes")
        (adapter / "unreviewed.txt").write_text(
            "unreviewed browser runtime text\n",
            encoding="utf-8",
        )
        return root

    def test_current_production_sources_have_no_unmodeled_embedded_file_inputs(self) -> None:
        _assert_no_unmodeled_rust_embedded_file_inputs(ROOT)

    def test_include_bytes_file_input_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'pub static EMBEDDED: &[u8] = include_bytes!("../unreviewed.bin");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_include_str_file_input_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'pub static EMBEDDED: &str = include_str!("../unreviewed.txt");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_namespaced_include_bytes_file_input_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'pub static EMBEDDED: &[u8] = core::include_bytes!("../unreviewed.bin");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_aliased_include_bytes_file_input_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use core::include_bytes as read_blob;\n'
            'pub static EMBEDDED: &[u8] = read_blob!("../unreviewed.bin");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_grouped_aliased_include_bytes_file_input_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use core::{include_bytes as read_blob};\n'
            'pub static EMBEDDED: &[u8] = read_blob!("../unreviewed.bin");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_underscore_prefixed_alias_still_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use core::include_bytes as _read_blob;\n'
            'pub static EMBEDDED: &[u8] = _read_blob!("../unreviewed.bin");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_unicode_continuation_include_bytes_alias_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use core::include_bytes as _\u0301;\n'
            'pub static EMBEDDED: &[u8] = _\u0301!("../unreviewed.bin");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_aliased_include_str_file_input_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use std::include_str as read_text;\n'
            'pub static EMBEDDED: &str = read_text!("../unreviewed.txt");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_unicode_continuation_include_str_alias_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use std::include_str as _\u0301;\n'
            'pub static EMBEDDED: &str = _\u0301!("../unreviewed.txt");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_underscore_import_is_not_callable_alias_authority(self) -> None:
        root = self._workspace_with_source(
            'use core::include_bytes as _;\n'
            'pub fn embedded_file_authority_control() -> usize { 0 }\n'
        )

        _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_raw_string_alias_text_is_not_embedded_file_authority(self) -> None:
        root = self._workspace_with_source(
            'pub const NOTE: &str = r#"use core::include_bytes as read_blob; '
            'read_blob!(\\"../unreviewed.bin\\")"#;\n'
        )

        _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_character_literal_does_not_hide_following_aliased_file_input(self) -> None:
        root = self._workspace_with_source(
            "pub const MARKER: char = 'x';\n"
            'use core::include_bytes as read_blob;\n'
            'pub static EMBEDDED: &[u8] = read_blob!("../unreviewed.bin");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust embedded file input"):
            _assert_no_unmodeled_rust_embedded_file_inputs(root)

    def test_comment_and_string_mentions_are_not_embedded_file_authority(self) -> None:
        root = self._workspace_with_source(
            '// include_bytes!("../unreviewed.bin")\n'
            '// use core::include_bytes as hidden_in_comment;\n'
            'pub const NOTE: &str = "include_str!(\\\"../unreviewed.txt\\\")";\n'
            'pub fn include_bytes_count() -> usize { 0 }\n'
        )

        _assert_no_unmodeled_rust_embedded_file_inputs(root)


if __name__ == "__main__":
    unittest.main()
