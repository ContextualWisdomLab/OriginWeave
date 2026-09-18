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


def _workspace_with_config(config_text: str) -> tuple[tempfile.TemporaryDirectory[str], pathlib.Path]:
    directory = tempfile.TemporaryDirectory()
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
    return directory, root


class BrowserSessionPgoProfileInputAuthorityContractTests(unittest.TestCase):
    """Keep repository-selected PGO data inside reviewed compiler-input provenance."""

    def _assert_fails_closed(self, config_text: str) -> None:
        directory, root = _workspace_with_config(config_text)
        self.addCleanup(directory.cleanup)
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_build_rustflags_profile_use_fails_closed(self) -> None:
        self._assert_fails_closed(
            '[build]\nrustflags = ["-Cprofile-use=tools/review-bypass.profdata"]\n'
        )

    def test_target_rustflags_profile_sample_use_fails_closed(self) -> None:
        self._assert_fails_closed(
            "[target.'cfg(unix)']\nrustflags = [\"-C\", \"profile-sample-use=tools/review-bypass.prof\"]\n"
        )

    def test_profile_rustflags_profile_use_fails_closed(self) -> None:
        self._assert_fails_closed(
            '[unstable]\nprofile-rustflags = true\n\n[profile.release]\nrustflags = ["--codegen=profile-use=tools/review-bypass.profdata"]\n'
        )

    def test_build_rustdocflags_profile_use_fails_closed(self) -> None:
        self._assert_fails_closed(
            '[build]\nrustdocflags = ["--codegen=profile-use=tools/review-bypass.profdata"]\n'
        )

    def test_doctest_build_arg_profile_use_fails_closed(self) -> None:
        self._assert_fails_closed(
            '[build]\nrustdocflags = ["--doctest-build-arg=-C", "--doctest-build-arg=profile-use=tools/review-bypass.profdata"]\n'
        )

    def test_profile_generate_output_remains_allowed(self) -> None:
        directory, root = _workspace_with_config(
            '[build]\nrustflags = ["-Cprofile-generate=target/pgo-data"]\n'
        )
        self.addCleanup(directory.cleanup)
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
