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


class BrowserSessionLlvmPluginAuthorityContractTests(unittest.TestCase):
    """Reject repository-selected rustc LLVM pass plugins from Git-owned Cargo flags."""

    def _workspace(self, config_text: str, *, profile_manifest: str = "") -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n' + profile_manifest,
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

    def _assert_plugin_fails_closed(self, config_text: str, *, profile_manifest: str = "") -> None:
        root = self._workspace(config_text, profile_manifest=profile_manifest)
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_build_rustflags_split_llvm_plugin_fails_closed(self) -> None:
        self._assert_plugin_fails_closed(
            '[build]\nrustflags = ["-Z", "llvm-plugins=tools/review-bypass-pass.so"]\n'
        )

    def test_target_rustflags_compact_llvm_plugin_fails_closed(self) -> None:
        self._assert_plugin_fails_closed(
            "[target.'cfg(unix)']\nrustflags = [\"-Zllvm-plugins=tools/review-bypass-pass.so\"]\n"
        )

    def test_config_profile_rustflags_llvm_plugin_fails_closed(self) -> None:
        self._assert_plugin_fails_closed(
            '[unstable]\nprofile-rustflags = true\n\n[profile.release]\nrustflags = ["-Z", "llvm-plugins=tools/review-bypass-pass.so"]\n'
        )

    def test_root_profile_rustflags_llvm_plugin_fails_closed(self) -> None:
        self._assert_plugin_fails_closed(
            '',
            profile_manifest='\n[profile.release]\nrustflags = ["-Zllvm-plugins=tools/review-bypass-pass.so"]\n',
        )

    def test_non_plugin_rustflags_remain_allowed(self) -> None:
        root = self._workspace(
            '[build]\nrustflags = ["-C", "opt-level=2", "--cfg", "originweave_reviewed"]\n'
        )
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
