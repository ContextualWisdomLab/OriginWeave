import importlib.util
import pathlib
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


class BrowserSessionPathMetaLexicalContractTests(unittest.TestCase):
    """Keep path-attribute authority tied to lexical Rust meta items, not data text."""

    def test_doc_string_path_text_is_not_path_meta(self) -> None:
        self.assertFalse(
            source_indirection._has_path_meta('doc = "path = \\"review_bypass.rs\\""')
        )

    def test_raw_doc_string_path_text_is_not_path_meta(self) -> None:
        self.assertFalse(
            source_indirection._has_path_meta('doc = r#"path = \\"review_bypass.rs\\""#')
        )

    def test_comment_path_text_is_not_path_meta(self) -> None:
        self.assertFalse(
            source_indirection._has_path_meta(
                'cfg_attr(unix, /* path = "review_bypass.rs" */ allow(dead_code))'
            )
        )

    def test_nested_cfg_attr_path_meta_remains_authority(self) -> None:
        self.assertTrue(
            source_indirection._has_path_meta(
                'cfg_attr(unix, path /* reviewed trivia */ = "unix_adapter.rs")'
            )
        )


if __name__ == "__main__":
    unittest.main()
