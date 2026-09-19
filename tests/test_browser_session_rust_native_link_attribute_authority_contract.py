import importlib.util
import pathlib
import re
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BOUNDARY_TEST = ROOT / "tests/test_browser_session_trusted_adapter_boundary.py"
SOURCE_INDIRECTION_TEST = ROOT / "tests/test_browser_session_rust_source_indirection_contract.py"
LINK_META_TOKEN = re.compile(r"(?<![\w#])(?:r#)?link(?!\w)", re.UNICODE)


def _load_contract(path: pathlib.Path, module_name: str):
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load {module_name} contract")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


boundary = _load_contract(BOUNDARY_TEST, "browser_session_trusted_adapter_boundary")
source_indirection = _load_contract(
    SOURCE_INDIRECTION_TEST,
    "browser_session_rust_source_indirection_contract",
)


def _attribute_contains_native_link_meta(attribute_body: str) -> bool:
    """Detect link(...) meta while ignoring Rust comments and string/character literals."""
    cursor = 0
    while cursor < len(attribute_body):
        trivia_end = source_indirection._skip_rust_trivia(attribute_body, cursor)
        if trivia_end != cursor:
            cursor = trivia_end
            continue

        raw_end = source_indirection._raw_string_end(attribute_body, cursor)
        if raw_end is not None:
            cursor = raw_end
            continue
        if attribute_body[cursor] == '"':
            cursor = source_indirection._quoted_string_end(attribute_body, cursor)
            continue
        if attribute_body[cursor] == "'":
            char_end = source_indirection._simple_char_literal_end(attribute_body, cursor)
            if char_end is not None:
                cursor = char_end
                continue

        match = LINK_META_TOKEN.match(attribute_body, cursor)
        if match is not None:
            operand = source_indirection._skip_rust_trivia(attribute_body, match.end())
            if operand < len(attribute_body) and attribute_body[operand] == "(":
                return True
            cursor = match.end()
            continue
        cursor += 1
    return False


def _assert_no_unmodeled_rust_native_link_inputs(root: pathlib.Path) -> None:
    """Fail closed when reviewed Rust source selects unresolved native-library bytes."""
    source_indirection._assert_no_unmodeled_rust_source_indirection(root)
    for source in boundary._workspace_production_sources(root):
        text = source.read_text(encoding="utf-8")
        for attribute_body in source_indirection._rust_attribute_bodies(text):
            if _attribute_contains_native_link_meta(attribute_body):
                relative = source.relative_to(root).as_posix()
                raise AssertionError(
                    "Rust link attribute requires an explicit native-library provenance contract: "
                    f"{relative}"
                )


class BrowserSessionRustNativeLinkAttributeAuthorityContractTests(unittest.TestCase):
    """Keep source-selected native libraries inside reviewed Browser Session provenance."""

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

    def test_current_production_sources_have_no_unmodeled_native_link_attributes(self) -> None:
        _assert_no_unmodeled_rust_native_link_inputs(ROOT)

    def test_direct_native_link_attribute_fails_closed(self) -> None:
        root = self._workspace_with_source(
            '#[link(name = "review_bypass", kind = "static")]\n'
            'unsafe extern "C" { fn reviewed_symbol(); }\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust link attribute"):
            _assert_no_unmodeled_rust_native_link_inputs(root)

    def test_cfg_attr_native_link_attribute_fails_closed(self) -> None:
        root = self._workspace_with_source(
            '#[cfg_attr(unix, link(name = "review_bypass"))]\n'
            'unsafe extern "C" { fn reviewed_symbol(); }\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust link attribute"):
            _assert_no_unmodeled_rust_native_link_inputs(root)

    def test_link_word_inside_attribute_string_is_not_native_link_authority(self) -> None:
        root = self._workspace_with_source(
            '#[doc = "link(name = \\\"not_an_attribute\\\")"]\n'
            'pub fn documented() {}\n'
        )

        _assert_no_unmodeled_rust_native_link_inputs(root)

    def test_link_section_attribute_is_not_native_library_selection(self) -> None:
        root = self._workspace_with_source(
            '#[unsafe(link_section = ".reviewed_section")]\n'
            'pub static REVIEWED: u8 = 1;\n'
        )

        _assert_no_unmodeled_rust_native_link_inputs(root)


if __name__ == "__main__":
    unittest.main()
