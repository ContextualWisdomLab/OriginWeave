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


def _compile_time_environment_macro_token_end(text: str, offset: int) -> int | None:
    """Return the end of a lexical env/option_env identifier token at offset."""
    for spelling in ("env", "option_env"):
        end = source_indirection._rust_identifier_token_end(
            text,
            offset,
            spelling,
            allow_raw=True,
        )
        if end is not None:
            return end
    return None


def _has_compile_time_environment_macro(text: str) -> bool:
    """Detect compile-time environment reads outside Rust comments and literals."""
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

        token_end = _compile_time_environment_macro_token_end(text, cursor)
        if token_end is not None:
            bang = source_indirection._skip_rust_trivia(text, token_end)
            if bang < len(text) and text[bang] == "!":
                return True
            cursor = token_end
            continue
        cursor += 1
    return False


def _use_tree_aliases_compile_time_environment_macro(use_tree: str) -> bool:
    """Return whether one use tree gives env!/option_env! a callable alias."""
    cursor = 0
    while cursor < len(use_tree):
        trivia_end = source_indirection._skip_rust_trivia(use_tree, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        raw_end = source_indirection._raw_string_end(use_tree, cursor)
        if raw_end is not None:
            cursor = raw_end
            continue
        if use_tree[cursor] == '"':
            cursor = source_indirection._quoted_string_end(use_tree, cursor)
            continue
        if use_tree[cursor] == "'":
            char_end = source_indirection._simple_char_literal_end(use_tree, cursor)
            if char_end is not None:
                cursor = char_end
                continue

        token_end = _compile_time_environment_macro_token_end(use_tree, cursor)
        if token_end is None:
            cursor += 1
            continue

        after_macro = source_indirection._skip_rust_trivia(use_tree, token_end)
        as_end = source_indirection._rust_identifier_token_end(use_tree, after_macro, "as")
        if as_end is None:
            cursor = token_end
            continue

        alias_start = source_indirection._skip_rust_trivia(use_tree, as_end)
        if alias_start >= len(use_tree):
            return False
        if use_tree[alias_start] == "_":
            next_offset = alias_start + 1
            if next_offset >= len(use_tree) or not source_indirection._rust_keyword_is_identifier_adjacent(
                use_tree[next_offset]
            ):
                cursor = token_end
                continue
        return True
    return False


def _has_aliased_compile_time_environment_import(text: str) -> bool:
    """Detect lexical use aliases that hide compile-time environment macro names."""
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

        use_end = source_indirection._rust_identifier_token_end(text, cursor, "use")
        if use_end is None:
            cursor += 1
            continue
        statement_end = source_indirection._rust_use_statement_end(text, use_end)
        if statement_end is None:
            return False
        if _use_tree_aliases_compile_time_environment_macro(text[use_end:statement_end]):
            return True
        cursor = statement_end + 1
    return False


def _assert_no_unmodeled_rust_compile_time_environment_inputs(root: pathlib.Path) -> None:
    """Fail closed when reviewed Rust source reads ambient build environment values."""
    source_indirection._assert_no_unmodeled_rust_source_indirection(root)
    for source in source_indirection.boundary._workspace_production_sources(root):
        text = source.read_text(encoding="utf-8")
        if _has_compile_time_environment_macro(text) or _has_aliased_compile_time_environment_import(text):
            relative = source.relative_to(root).as_posix()
            raise AssertionError(
                "Rust compile-time environment input requires an explicit provenance contract: "
                f"{relative}"
            )


class BrowserSessionRustCompileTimeEnvironmentAuthorityContractTests(unittest.TestCase):
    """Keep Rust compile-time environment inputs inside reviewed provenance."""

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
        return root

    def test_current_production_sources_have_no_unmodeled_compile_time_environment_inputs(self) -> None:
        _assert_no_unmodeled_rust_compile_time_environment_inputs(ROOT)

    def test_env_macro_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'pub const BUILD_ID: &str = env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_option_env_macro_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'pub const BUILD_ID: Option<&str> = option_env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_namespaced_env_macro_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'pub const BUILD_ID: &str = std::env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_namespaced_option_env_macro_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'pub const BUILD_ID: Option<&str> = core::option_env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_aliased_env_macro_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use std::env as read_build_env;\n'
            'pub const BUILD_ID: &str = read_build_env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_grouped_aliased_option_env_macro_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use core::{option_env as read_optional_build_env};\n'
            'pub const BUILD_ID: Option<&str> = read_optional_build_env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_raw_identifier_aliased_env_macro_fails_closed(self) -> None:
        root = self._workspace_with_source(
            'use std::env as r#type;\n'
            'pub const BUILD_ID: &str = r#type!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_underscore_import_is_not_callable_compile_time_environment_authority(self) -> None:
        root = self._workspace_with_source(
            'use std::env as _;\n'
            'pub fn reviewed_runtime_environment() -> Option<String> { std::env::var("PATH").ok() }\n'
        )

        _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_unicode_continuation_after_underscore_is_not_discard_alias(self) -> None:
        root = self._workspace_with_source(
            'use std::env as _\u0301;\n'
            'pub const BUILD_ID: &str = _\u0301!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_comment_string_and_raw_string_mentions_are_not_compile_time_environment_authority(self) -> None:
        root = self._workspace_with_source(
            '// env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID")\n'
            '// use std::env as hidden_build_env;\n'
            'pub const NOTE: &str = "option_env!(\\\"ORIGINWEAVE_UNREVIEWED_BUILD_ID\\\")";\n'
            'pub const RAW_NOTE: &str = r#"use std::env as hidden_raw_build_env; env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID")"#;\n'
            'pub fn env_count() -> usize { 0 }\n'
        )

        _assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_character_literal_does_not_hide_following_real_macro(self) -> None:
        root = self._workspace_with_source(
            "pub const MARKER: char = 'x';\n"
            'pub const BUILD_ID: &str = env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            _assert_no_unmodeled_rust_compile_time_environment_inputs(root)


if __name__ == "__main__":
    unittest.main()
