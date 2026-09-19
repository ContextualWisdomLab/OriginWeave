import importlib.util
import pathlib
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


if __name__ == "__main__":
    unittest.main()
