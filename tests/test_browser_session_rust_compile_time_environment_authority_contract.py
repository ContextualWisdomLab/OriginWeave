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


class BrowserSessionRustCompileTimeEnvironmentAuthorityContractTests(unittest.TestCase):
    """Expose ambient compile-time environment inputs outside reviewed Rust source provenance."""

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

    def test_env_macro_exposes_unmodeled_compile_time_environment_input(self) -> None:
        root = self._workspace_with_source(
            'pub const BUILD_ID: &str = env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            source_indirection._assert_no_unmodeled_rust_source_indirection(root)

    def test_option_env_macro_exposes_unmodeled_compile_time_environment_input(self) -> None:
        root = self._workspace_with_source(
            'pub const BUILD_ID: Option<&str> = option_env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            source_indirection._assert_no_unmodeled_rust_source_indirection(root)

    def test_comment_and_string_mentions_are_not_compile_time_environment_authority(self) -> None:
        root = self._workspace_with_source(
            '// env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID")\n'
            'pub const NOTE: &str = "option_env!(\\\"ORIGINWEAVE_UNREVIEWED_BUILD_ID\\\")";\n'
        )

        source_indirection._assert_no_unmodeled_rust_source_indirection(root)


if __name__ == "__main__":
    unittest.main()
