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


class BrowserSessionLinkerToolchainSelectionContractTests(unittest.TestCase):
    """Keep rustc-managed linker binary selection inside the reviewed build boundary."""

    def _workspace_with_flags(self, rustflags: str, *, target_scoped: bool = False) -> pathlib.Path:
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
        table = '[target.x86_64-unknown-linux-gnu]' if target_scoped else '[build]'
        (cargo / "config.toml").write_text(
            f'{table}\nrustflags = {rustflags}\n',
            encoding="utf-8",
        )
        return root

    def test_repository_link_self_contained_linker_selection_fails_closed(self) -> None:
        root = self._workspace_with_flags(
            '["-C", "link-self-contained=+linker"]'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_target_linker_features_selection_fails_closed(self) -> None:
        root = self._workspace_with_flags(
            '["--codegen=linker-features=+lld"]',
            target_scoped=True,
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_unrelated_codegen_option_remains_allowed(self) -> None:
        root = self._workspace_with_flags('["-C", "debuginfo=1"]')
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
