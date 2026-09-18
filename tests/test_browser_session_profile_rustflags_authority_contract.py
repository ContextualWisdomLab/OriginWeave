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


class BrowserSessionProfileRustflagsAuthorityContractTests(unittest.TestCase):
    """Keep profile-selected rustc arguments inside the reviewed Cargo authority boundary."""

    def _workspace(self, *, profile_text: str, config_text: str | None = None) -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            'cargo-features = ["profile-rustflags"]\n\n'
            '[workspace]\n'
            'members = ["adapter"]\n'
            'resolver = "3"\n\n'
            f"{profile_text}",
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "src").mkdir(parents=True)
        (adapter / "src/lib.rs").write_text("pub fn adapter_surface() {}\n", encoding="utf-8")
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        if config_text is not None:
            cargo = root / ".cargo"
            cargo.mkdir()
            (cargo / "config.toml").write_text(config_text, encoding="utf-8")
        return root

    def test_root_manifest_profile_rustflags_external_input_fails_closed(self) -> None:
        root = self._workspace(
            profile_text=(
                '[profile.release]\n'
                'rustflags = ["--extern", "review_bypass=tools/libreview_bypass.rlib"]\n'
            )
        )
        with self.assertRaisesRegex(AssertionError, "profile_rustflag_authority"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_cargo_config_profile_rustflags_linker_selection_fails_closed(self) -> None:
        root = self._workspace(
            profile_text='[profile.release]\nopt-level = 2\n',
            config_text=(
                '[unstable]\nprofile-rustflags = true\n\n'
                '[profile.release]\n'
                'rustflags = ["-C", "linker=tools/review-bypass-linker"]\n'
            ),
        )
        with self.assertRaisesRegex(AssertionError, "profile_rustflag_authority"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_non_authority_profile_rustflags_remain_allowed(self) -> None:
        root = self._workspace(
            profile_text=(
                '[profile.release]\n'
                'rustflags = ["-C", "opt-level=2", "--cfg", "originweave_reviewed"]\n'
            )
        )
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
