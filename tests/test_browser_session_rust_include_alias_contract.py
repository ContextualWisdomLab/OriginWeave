import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE_INDIRECTION_TEST = ROOT / "tests/test_browser_session_rust_source_indirection_contract.py"

spec = importlib.util.spec_from_file_location(
    "browser_session_rust_source_indirection",
    SOURCE_INDIRECTION_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Rust source-indirection contract")
source_indirection = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source_indirection)


class BrowserSessionRustIncludeAliasContractTests(unittest.TestCase):
    """Prove that renaming Rust's include macro cannot bypass source provenance review."""

    def test_aliased_include_macro_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
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
            (adapter / "src/lib.rs").write_text(
                'use core::include as embed;\nembed!("../generated_adapter.rs");\n',
                encoding="utf-8",
            )
            (adapter / "generated_adapter.rs").write_text(
                "pub fn generated_adapter_surface() {}\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                AssertionError,
                "Rust include! source indirection",
            ):
                source_indirection._assert_no_unmodeled_rust_source_indirection(root)


if __name__ == "__main__":
    unittest.main()
