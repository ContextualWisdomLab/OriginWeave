import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
COMPILE_TIME_ENVIRONMENT_TEST = (
    ROOT / "tests/test_browser_session_rust_compile_time_environment_authority_contract.py"
)

spec = importlib.util.spec_from_file_location(
    "browser_session_rust_compile_time_environment_authority_contract",
    COMPILE_TIME_ENVIRONMENT_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session compile-time environment contract")
compile_time_environment = importlib.util.module_from_spec(spec)
spec.loader.exec_module(compile_time_environment)


class BrowserSessionRustCompileTimeEnvironmentIdentifierBoundaryContractTests(unittest.TestCase):
    """Keep env!/option_env! detection aligned with Rust identifier boundaries."""

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

    def test_unicode_identifier_continuation_before_env_is_not_builtin_macro(self) -> None:
        source = (
            "macro_rules! _\u0301env { () => { \"reviewed\" }; }\n"
            "pub const BUILD_ID: &str = _\u0301env!();\n"
        )

        self.assertFalse(
            compile_time_environment._has_compile_time_environment_macro(source),
            "a Rust XID_Continue character before env must keep env inside the user macro identifier",
        )

    def test_real_env_macro_remains_detected(self) -> None:
        self.assertTrue(
            compile_time_environment._has_compile_time_environment_macro(
                'pub const BUILD_ID: &str = env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
            )
        )

    def test_unicode_identifier_continuation_before_use_is_macro_name_data(self) -> None:
        root = self._workspace_with_source(
            "macro_rules! a\u0301use { ($($token:tt)*) => {}; }\n"
            "a\u0301use!(std::env as hidden_build_env);\n"
            'pub fn reviewed_runtime_environment() -> Option<String> { std::env::var("PATH").ok() }\n'
        )

        compile_time_environment._assert_no_unmodeled_rust_compile_time_environment_inputs(root)

    def test_real_use_alias_remains_compile_time_environment_authority(self) -> None:
        root = self._workspace_with_source(
            "use std::env as hidden_build_env;\n"
            'pub const BUILD_ID: &str = hidden_build_env!("ORIGINWEAVE_UNREVIEWED_BUILD_ID");\n'
        )

        with self.assertRaisesRegex(AssertionError, "Rust compile-time environment input"):
            compile_time_environment._assert_no_unmodeled_rust_compile_time_environment_inputs(root)


if __name__ == "__main__":
    unittest.main()
