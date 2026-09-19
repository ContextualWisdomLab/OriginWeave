import importlib.util
import pathlib
import re
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BOUNDARY_TEST = ROOT / "tests/test_browser_session_trusted_adapter_boundary.py"

spec = importlib.util.spec_from_file_location("browser_session_trusted_adapter_boundary", BOUNDARY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session trusted-adapter boundary contract")
boundary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boundary)


INCLUDE_TOKEN = "include"
USE_TOKEN = re.compile(r"(?<![\w#])use(?!\w)", re.UNICODE)
AS_TOKEN = re.compile(r"(?<![\w#])as(?!\w)", re.UNICODE)
PATH_TOKEN = re.compile(r"(?<![\w#])path(?!\w)", re.UNICODE)
CUSTOM_TARGET_MOD_TOKEN = "mod"
RUST_PATTERN_WHITESPACE = frozenset(
    "\u0009\u000a\u000b\u000c\u000d\u0020\u0085\u200e\u200f\u2028\u2029"
)
APPROVED_RUST_PATH_ATTRIBUTES = {
    ("crates/originweave-core/src/root.rs", 'path = "lib.rs"'),
}


def _normalized_attribute_body(body: str) -> str:
    return " ".join(body.split())


def _skip_rust_trivia(text: str, offset: int) -> int:
    """Skip Rust whitespace and nested non-doc comments without changing token meaning."""
    index = offset
    while index < len(text):
        if text[index] in RUST_PATTERN_WHITESPACE:
            index += 1
            continue
        if text.startswith("//", index):
            newline = text.find("\n", index + 2)
            index = len(text) if newline < 0 else newline + 1
            continue
        if text.startswith("/*", index):
            depth = 1
            index += 2
            while index < len(text) and depth:
                if text.startswith("/*", index):
                    depth += 1
                    index += 2
                elif text.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            if depth:
                raise AssertionError("unterminated Rust block comment in source attribute")
            continue
        break
    return index


def _rust_keyword_is_identifier_adjacent(char: str) -> bool:
    """Conservatively reject token boundaries that could be Rust identifier continuation."""
    if char.isascii():
        return char.isalnum() or char == "_"
    return char not in RUST_PATTERN_WHITESPACE


def _rust_identifier_token_end(
    text: str,
    offset: int,
    spelling: str,
    *,
    allow_raw: bool = False,
) -> int | None:
    """Return a conservative Rust identifier-token end independent of Python Unicode tables."""
    token_start = offset
    identifier_start = offset
    if allow_raw and text.startswith("r#", offset) and text.startswith(spelling, offset + 2):
        identifier_start = offset + 2
    elif not text.startswith(spelling, offset):
        return None

    if token_start:
        previous = text[token_start - 1]
        if previous == "#" or _rust_keyword_is_identifier_adjacent(previous):
            return None

    end = identifier_start + len(spelling)
    if end < len(text) and _rust_keyword_is_identifier_adjacent(text[end]):
        return None
    return end


def _has_include_macro(text: str) -> bool:
    """Detect lexical include! macro syntax outside Rust comments and literals."""
    cursor = 0
    while cursor < len(text):
        trivia_end = _skip_rust_trivia(text, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        raw_end = _raw_string_end(text, cursor)
        if raw_end is not None:
            cursor = raw_end
            continue
        if text[cursor] == '"':
            cursor = _quoted_string_end(text, cursor)
            continue
        if text[cursor] == "'":
            char_end = _simple_char_literal_end(text, cursor)
            if char_end is not None:
                cursor = char_end
                continue

        token_end = _rust_identifier_token_end(text, cursor, INCLUDE_TOKEN, allow_raw=True)
        if token_end is None:
            cursor += 1
            continue
        bang = _skip_rust_trivia(text, token_end)
        if bang < len(text) and text[bang] == "!":
            delimiter = _skip_rust_trivia(text, bang + 1)
            if delimiter < len(text) and text[delimiter] in "([{":
                return True
        cursor = token_end
    return False


def _matches_custom_target_mod_token(text: str, offset: int) -> bool:
    """Match the Rust `mod` keyword without relying on Python's Unicode identifier table."""
    return _rust_identifier_token_end(text, offset, CUSTOM_TARGET_MOD_TOKEN) is not None


def _has_custom_target_mod_token(text: str) -> bool:
    """Detect lexical Rust mod tokens outside comments and string/character literals."""
    cursor = 0
    while cursor < len(text):
        trivia_end = _skip_rust_trivia(text, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        raw_end = _raw_string_end(text, cursor)
        if raw_end is not None:
            cursor = raw_end
            continue
        if text[cursor] == '"':
            cursor = _quoted_string_end(text, cursor)
            continue
        if text[cursor] == "'":
            char_end = _simple_char_literal_end(text, cursor)
            if char_end is not None:
                cursor = char_end
                continue

        if _matches_custom_target_mod_token(text, cursor):
            return True
        cursor += 1
    return False


def _rust_use_statement_end(text: str, offset: int) -> int | None:
    """Return the semicolon ending one Rust use declaration while ignoring comment trivia."""
    cursor = offset
    while cursor < len(text):
        trivia_end = _skip_rust_trivia(text, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue
        if text[cursor] == ";":
            return cursor
        cursor += 1
    return None


def _has_aliased_include_import(text: str) -> bool:
    """Detect lexical use-tree aliases that rename include! before invocation."""
    cursor = 0
    while cursor < len(text):
        trivia_end = _skip_rust_trivia(text, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        raw_end = _raw_string_end(text, cursor)
        if raw_end is not None:
            cursor = raw_end
            continue
        if text[cursor] == '"':
            cursor = _quoted_string_end(text, cursor)
            continue
        if text[cursor] == "'":
            char_end = _simple_char_literal_end(text, cursor)
            if char_end is not None:
                cursor = char_end
                continue

        use_match = USE_TOKEN.match(text, cursor)
        if use_match is None:
            cursor += 1
            continue
        statement_end = _rust_use_statement_end(text, use_match.end())
        if statement_end is None:
            return False
        use_tree = text[use_match.end():statement_end]
        use_cursor = 0
        while use_cursor < len(use_tree):
            trivia_end = _skip_rust_trivia(use_tree, use_cursor)
            if trivia_end != use_cursor:
                use_cursor = trivia_end
                continue

            raw_end = _raw_string_end(use_tree, use_cursor)
            if raw_end is not None:
                use_cursor = raw_end
                continue
            if use_tree[use_cursor] == '"':
                use_cursor = _quoted_string_end(use_tree, use_cursor)
                continue
            if use_tree[use_cursor] == "'":
                char_end = _simple_char_literal_end(use_tree, use_cursor)
                if char_end is not None:
                    use_cursor = char_end
                    continue

            include_match = re.match(r"(?<![\w#])(?:r#)?include(?!\w)", use_tree[use_cursor:], re.UNICODE)
            if include_match is None:
                use_cursor += 1
                continue
            include_end = use_cursor + include_match.end()
            include_cursor = _skip_rust_trivia(use_tree, include_end)
            as_match = AS_TOKEN.match(use_tree, include_cursor)
            if as_match is None:
                use_cursor = include_end
                continue
            alias_start = _skip_rust_trivia(use_tree, as_match.end())
            if alias_start >= len(use_tree):
                use_cursor = include_end
                continue
            if use_tree[alias_start] == "_":
                next_offset = alias_start + 1
                if next_offset >= len(use_tree) or not (
                    use_tree[next_offset].isalnum() or use_tree[next_offset] == "_"
                ):
                    use_cursor = include_end
                    continue
            return True
        cursor = statement_end + 1
    return False


def _raw_string_end(text: str, offset: int) -> int | None:
    """Return the end of a Rust raw string token beginning at offset, if present."""
    cursor = offset
    if text.startswith(("br", "cr"), cursor):
        cursor += 2
    elif cursor < len(text) and text[cursor] == "r":
        cursor += 1
    else:
        return None

    hashes_start = cursor
    while cursor < len(text) and text[cursor] == "#":
        cursor += 1
    if cursor >= len(text) or text[cursor] != '"':
        return None

    hashes = text[hashes_start:cursor]
    closing = '"' + hashes
    end = text.find(closing, cursor + 1)
    if end < 0:
        raise AssertionError("unterminated Rust raw string in source attribute")
    return end + len(closing)


def _quoted_string_end(text: str, offset: int) -> int:
    """Return the end of a conventional Rust string token beginning with a quote."""
    index = offset + 1
    escaped = False
    while index < len(text):
        char = text[index]
        if escaped:
            escaped = False
        elif char == "\\":
            escaped = True
        elif char == '"':
            return index + 1
        index += 1
    raise AssertionError("unterminated Rust string in source attribute")


def _simple_char_literal_end(text: str, offset: int) -> int | None:
    """Skip a simple Rust character literal while leaving lifetimes untouched."""
    if offset + 2 < len(text) and text[offset + 2] == "'":
        return offset + 3
    if offset + 1 >= len(text) or text[offset + 1] != "\\":
        return None

    index = offset + 2
    while index < len(text):
        if text[index] == "'":
            return index + 1
        if text[index] == "\n":
            return None
        index += 1
    return None


def _next_rust_attribute_marker(text: str, offset: int) -> int | None:
    """Find the next lexical `#` outside Rust comments and string/character literals."""
    cursor = offset
    while cursor < len(text):
        trivia_end = _skip_rust_trivia(text, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        raw_end = _raw_string_end(text, cursor)
        if raw_end is not None:
            cursor = raw_end
            continue
        if text[cursor] == '"':
            cursor = _quoted_string_end(text, cursor)
            continue
        if text[cursor] == "'":
            char_end = _simple_char_literal_end(text, cursor)
            if char_end is not None:
                cursor = char_end
                continue
        if text[cursor] == "#":
            return cursor
        cursor += 1
    return None


def _rust_attribute_bodies(text: str) -> list[str]:
    """Extract balanced Rust attribute token trees while respecting lexical trivia and literals."""
    bodies: list[str] = []
    search_from = 0
    while True:
        marker = _next_rust_attribute_marker(text, search_from)
        if marker is None:
            break

        cursor = _skip_rust_trivia(text, marker + 1)
        if cursor < len(text) and text[cursor] == "!":
            cursor = _skip_rust_trivia(text, cursor + 1)
        if cursor >= len(text) or text[cursor] != "[":
            search_from = marker + 1
            continue

        body_start = cursor + 1
        depth = 1
        cursor = body_start
        while cursor < len(text):
            trivia_end = _skip_rust_trivia(text, cursor)
            if trivia_end != cursor:
                cursor = trivia_end
                continue

            raw_end = _raw_string_end(text, cursor)
            if raw_end is not None:
                cursor = raw_end
                continue
            if text[cursor] == '"':
                cursor = _quoted_string_end(text, cursor)
                continue
            if text[cursor] == "'":
                char_end = _simple_char_literal_end(text, cursor)
                if char_end is not None:
                    cursor = char_end
                    continue

            if text[cursor] == "[":
                depth += 1
            elif text[cursor] == "]":
                depth -= 1
                if depth == 0:
                    bodies.append(text[body_start:cursor])
                    search_from = cursor + 1
                    break
            cursor += 1
        else:
            raise AssertionError("unterminated Rust attribute in production source")

    return bodies


def _has_path_meta(attribute_body: str) -> bool:
    """Return whether an attribute contains a path meta item followed by valid Rust trivia and '='."""
    for match in PATH_TOKEN.finditer(attribute_body):
        cursor = _skip_rust_trivia(attribute_body, match.end())
        if cursor < len(attribute_body) and attribute_body[cursor] == "=":
            return True
    return False


def _is_under_any_default_src(source: pathlib.Path, src_roots: list[pathlib.Path]) -> bool:
    """Return whether Cargo source discovery already reviews every sibling module under this source root."""
    resolved = source.resolve()
    for src_root in src_roots:
        try:
            resolved.relative_to(src_root)
            return True
        except ValueError:
            continue
    return False


def _assert_no_unmodeled_rust_source_indirection(root: pathlib.Path) -> None:
    """Fail closed when reviewed Rust source can pull unmodeled executable source bytes."""
    discovered_path_attributes: set[tuple[str, str]] = set()
    default_src_roots = [
        (manifest.parent / "src").resolve()
        for manifest in boundary._production_package_manifests(root)
    ]

    for source in boundary._workspace_production_sources(root):
        text = source.read_text(encoding="utf-8")
        relative = source.relative_to(root).as_posix()

        if _has_include_macro(text) or _has_aliased_include_import(text):
            raise AssertionError(
                f"Rust include! source indirection requires an explicit provenance contract: {relative}"
            )

        # A custom target root is outside the canonical src/**/*.rs sibling closure. Until
        # compiler-derived source inputs replace this guard, any lexical `mod` token is an
        # intentionally conservative provenance stop: comments/trivia, raw/Unicode names,
        # visibility spellings, and inline-vs-outlined grammar must not create bypasses.
        if not _is_under_any_default_src(source, default_src_roots) and _has_custom_target_mod_token(text):
            raise AssertionError(
                "Rust module source indirection from a custom Cargo target requires an explicit "
                f"provenance contract: {relative}"
            )

        for attribute_body in _rust_attribute_bodies(text):
            if _has_path_meta(attribute_body):
                discovered_path_attributes.add(
                    (relative, _normalized_attribute_body(attribute_body))
                )

    unexpected = discovered_path_attributes - APPROVED_RUST_PATH_ATTRIBUTES
    if unexpected:
        raise AssertionError(
            "Rust path attribute requires an explicit exact-tree provenance review: "
            f"{sorted(unexpected)}"
        )

    stale = APPROVED_RUST_PATH_ATTRIBUTES - discovered_path_attributes
    if root.resolve() == ROOT.resolve() and stale:
        raise AssertionError(
            f"Rust path-attribute allowlist preapproves absent production surfaces: {sorted(stale)}"
        )


class BrowserSessionRustSourceIndirectionContractTests(unittest.TestCase):
    """Keep Rust source indirection inside the exact-head production-source provenance boundary."""

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

    def _custom_target_workspace(self, source_text: str, module_file: str) -> pathlib.Path:
        """Create a custom-target crate whose outlined module is outside Cargo's default src tree."""
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
        (adapter / f"runtime/{module_file}").write_text(
            "pub fn helper_surface() {}\n",
            encoding="utf-8",
        )
        return root

    def _assert_include_form_fails_closed(self, source_text: str) -> None:
        root = self._workspace_with_source(source_text)
        (root / "adapter/generated_adapter.rs").write_text(
            "pub fn generated_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust include! source indirection"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_current_production_sources_have_no_unmodeled_source_indirection(self) -> None:
        _assert_no_unmodeled_rust_source_indirection(ROOT)

    def test_parenthesized_include_macro_fails_closed(self) -> None:
        self._assert_include_form_fails_closed('include!("../generated_adapter.rs");\n')

    def test_braced_include_macro_fails_closed(self) -> None:
        self._assert_include_form_fails_closed('include! { "../generated_adapter.rs" }\n')

    def test_bracketed_include_macro_fails_closed(self) -> None:
        self._assert_include_form_fails_closed('include!["../generated_adapter.rs"];\n')

    def test_bare_module_from_custom_target_fails_closed(self) -> None:
        root = self._custom_target_workspace(
            "mod helper;\npub fn lifecycle_adapter_surface() {}\n",
            "helper.rs",
        )

        with self.assertRaisesRegex(AssertionError, "Rust module source indirection"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_raw_identifier_bare_module_from_custom_target_fails_closed(self) -> None:
        root = self._custom_target_workspace(
            "mod r#type;\npub fn lifecycle_adapter_surface() {}\n",
            "type.rs",
        )

        with self.assertRaisesRegex(AssertionError, "Rust module source indirection"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_unicode_identifier_bare_module_from_custom_target_fails_closed(self) -> None:
        root = self._custom_target_workspace(
            "mod 관찰;\npub fn lifecycle_adapter_surface() {}\n",
            "관찰.rs",
        )

        with self.assertRaisesRegex(AssertionError, "Rust module source indirection"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_comment_trivia_after_mod_from_custom_target_fails_closed(self) -> None:
        root = self._custom_target_workspace(
            "mod /* reviewed trivia */ helper;\npub fn lifecycle_adapter_surface() {}\n",
            "helper.rs",
        )

        with self.assertRaisesRegex(AssertionError, "Rust module source indirection"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_bare_module_under_default_src_uses_existing_source_closure(self) -> None:
        root = self._workspace_with_source("mod nested;\npub fn adapter_surface() {}\n")
        (root / "adapter/src/nested.rs").write_text(
            "pub fn nested_adapter_surface() {}\n",
            encoding="utf-8",
        )

        _assert_no_unmodeled_rust_source_indirection(root)

    def test_new_path_attribute_fails_closed_even_when_target_is_in_tree(self) -> None:
        root = self._workspace_with_source('#[path = "nested.rs"]\nmod nested;\n')
        (root / "adapter/src/nested.rs").write_text(
            "pub fn nested_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust path attribute requires"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_cfg_attr_generated_path_attribute_fails_closed(self) -> None:
        root = self._workspace_with_source(
            '#[cfg_attr(unix, path = "unix_adapter.rs")]\nmod platform_adapter;\n'
        )
        (root / "adapter/src/unix_adapter.rs").write_text(
            "pub fn unix_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust path attribute requires"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_parent_traversal_path_attribute_fails_closed(self) -> None:
        root = self._workspace_with_source('#[path = "../shared_adapter.rs"]\nmod shared_adapter;\n')
        (root / "adapter/shared_adapter.rs").write_text(
            "pub fn shared_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust path attribute requires"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_commented_path_attribute_text_is_not_source_indirection(self) -> None:
        root = self._workspace_with_source(
            '// #[path = "review_bypass.rs"]\n'
            'pub fn reviewed_surface() {}\n'
        )

        _assert_no_unmodeled_rust_source_indirection(root)

    def test_string_containing_path_attribute_text_is_not_source_indirection(self) -> None:
        root = self._workspace_with_source(
            'pub const NOTE: &str = "#[path = \\"review_bypass.rs\\"]";\n'
        )

        _assert_no_unmodeled_rust_source_indirection(root)


if __name__ == "__main__":
    unittest.main()