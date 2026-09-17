import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location("browser_session_cargo_compiler_authority", AUTHORITY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Cargo compiler-authority contract")
authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(authority)


class BrowserSessionLinkerDriverOverrideContractTests(unittest.TestCase):
    """Reject Cargo-owned codegen flags that re-select the linker behind the compiler driver."""

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

    def _assert_linker_driver_override_fails_closed(self, config_text: str) -> None:
        root = self._workspace_with_config(config_text)
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_build_link_arg_fuse_ld_override_fails_closed(self) -> None:
        self._assert_linker_driver_override_fails_closed(
            '[build]\nrustflags = ["-C", "link-arg=-fuse-ld=review-bypass-linker"]\n'
        )

    def test_target_long_link_args_fuse_ld_override_fails_closed(self) -> None:
        self._assert_linker_driver_override_fails_closed(
            "[target.'cfg(unix)']\nrustflags = [\"--codegen=link-args=-fuse-ld=review-bypass-linker\"]\n"
        )


if __name__ == "__main__":
    unittest.main()
