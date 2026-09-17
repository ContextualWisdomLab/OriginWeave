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


class BrowserSessionRawIdentifierIncludeContractTests(unittest.TestCase):
    """Keep raw-identifier macro spelling inside the include! provenance stop."""

    def test_raw_identifier_include_macro_fails_closed(self) -> None:
        self.assertTrue(
            source_indirection._has_include_macro(
                'r#include!("../generated_adapter.rs");\n'
            ),
            "Rust raw identifiers preserve the underlying macro identifier and must not bypass include! provenance",
        )


if __name__ == "__main__":
    unittest.main()
