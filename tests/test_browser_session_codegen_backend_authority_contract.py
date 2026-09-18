import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location("browser_session_cargo_compiler_authority", AUTHORITY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Cargo compiler authority contract")
authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(authority)


class BrowserSessionCodegenBackendAuthorityContractTests(unittest.TestCase):
    """Keep repository-selected rustc code generation backends outside the Browser Session TCB."""

    def _workspace_with_config(
        self,
        config_text: str = "",
        *,
        root_profile_text: str = "",
    ) -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n' + root_profile_text,
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "src").mkdir(parents=True)
        (adapter / "src/lib.rs").write_text("pub fn adapter_surface() {}\n", encoding="utf-8")
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        if config_text:
            cargo = root / ".cargo"
            cargo.mkdir()
            (cargo / "config.toml").write_text(config_text, encoding="utf-8")
        return root

    def _assert_fails_closed(
        self,
        config_text: str = "",
        *,
        root_profile_text: str = "",
    ) -> None:
        root = self._workspace_with_config(config_text, root_profile_text=root_profile_text)
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_rustflags_codegen_backend_path_fails_closed(self) -> None:
        for config_text in (
            '[build]\nrustflags = ["-Zcodegen-backend=tools/review-bypass-backend.so"]\n',
            '[build]\nrustflags = ["-Z", "codegen-backend=tools/review-bypass-backend.so"]\n',
            "[target.'cfg(unix)']\nrustflags = [\"-Zcodegen-backend=tools/review-bypass-backend.so\"]\n",
        ):
            with self.subTest(config_text=config_text):
                self._assert_fails_closed(config_text)

    def test_repository_config_profile_codegen_backend_fails_closed(self) -> None:
        self._assert_fails_closed(
            '[unstable]\ncodegen-backend = true\n\n'
            '[profile.dev.package.adapter]\ncodegen-backend = "cranelift"\n'
        )

    def test_repository_manifest_profile_codegen_backend_fails_closed(self) -> None:
        self._assert_fails_closed(
            root_profile_text='\n[profile.dev]\ncodegen-backend = "cranelift"\n'
        )

    def test_unrelated_profile_setting_remains_allowed(self) -> None:
        root = self._workspace_with_config(root_profile_text='\n[profile.dev]\nopt-level = 1\n')
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
