import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
COMPILER_AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location(
    "browser_session_cargo_compiler_authority_contract",
    COMPILER_AUTHORITY_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Cargo compiler-authority contract")
authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(authority)


class BrowserSessionNativeLibraryInputContractTests(unittest.TestCase):
    """Keep Git-owned native library/search-path inputs inside the reviewed Browser Session TCB."""

    def _workspace_with_config(self, config_text: str) -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n',
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "src").mkdir(parents=True)
        (adapter / "src/lib.rs").write_text("pub fn adapter_surface() {}\n", encoding="utf-8")
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        cargo = root / ".cargo"
        cargo.mkdir()
        (cargo / "config.toml").write_text(config_text, encoding="utf-8")
        return root

    def _assert_input_override_fails_closed(self, config_text: str) -> None:
        root = self._workspace_with_config(config_text)
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_git_owned_native_library_search_path_fails_closed(self) -> None:
        self._assert_input_override_fails_closed(
            '[build]\nrustflags = ["-L", "native=tools/review-bypass-native"]\n'
        )

    def test_git_owned_native_library_link_request_fails_closed(self) -> None:
        self._assert_input_override_fails_closed(
            '[build]\nrustflags = ["-l", "static:+whole-archive=review_bypass_native"]\n'
        )

    def test_codegen_link_arg_native_library_search_path_fails_closed(self) -> None:
        self._assert_input_override_fails_closed(
            '[build]\nrustflags = ["-C", "link-arg=-Ltools/review-bypass-native"]\n'
        )

    def test_codegen_link_args_native_library_search_path_fails_closed(self) -> None:
        self._assert_input_override_fails_closed(
            '[build]\nrustflags = ["--codegen", "link-args=-L tools/review-bypass-native"]\n'
        )

    def test_target_codegen_link_arg_native_library_request_fails_closed(self) -> None:
        self._assert_input_override_fails_closed(
            "[target.'cfg(unix)']\nrustflags = [\"-C\", \"link-arg=-lreview_bypass_native\"]\n"
        )

    def test_target_codegen_link_args_native_library_request_fails_closed(self) -> None:
        self._assert_input_override_fails_closed(
            "[target.'cfg(unix)']\nrustflags = [\"--codegen=link-args=-l review_bypass_native\"]\n"
        )

    def test_unrelated_codegen_flag_remains_allowed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustflags = ["-C", "debuginfo=1"]\n'
        )
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
