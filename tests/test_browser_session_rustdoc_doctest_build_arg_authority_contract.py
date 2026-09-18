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


class BrowserSessionRustdocDoctestBuildArgAuthorityContractTests(unittest.TestCase):
    """Keep doctest compiler arguments inside the reviewed Cargo execution/input boundary."""

    def _assert_fails_closed(self, config_text: str) -> None:
        directory, root = _workspace_with_config(config_text)
        self.addCleanup(directory.cleanup)
        with self.assertRaisesRegex(AssertionError, "rustdocflags:doctest compiler authority"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_build_rustdocflags_forwarded_sysroot_fails_closed(self) -> None:
        self._assert_fails_closed(
            '[build]\nrustdocflags = ["-Z", "unstable-options", "--doctest-build-arg=--sysroot=tools/review-bypass-sysroot"]\n'
        )

    def test_target_rustdocflags_forwarded_linker_fails_closed(self) -> None:
        self._assert_fails_closed(
            "[target.'cfg(unix)']\nrustdocflags = [\"-Zunstable-options\", \"--doctest-build-arg\", \"-C\", \"--doctest-build-arg\", \"linker=tools/review-bypass-linker\"]\n"
        )

    def test_build_rustdocflags_forwarded_codegen_backend_fails_closed(self) -> None:
        self._assert_fails_closed(
            '[build]\nrustdocflags = ["-Zunstable-options", "--doctest-build-arg=-Zcodegen-backend=tools/review-bypass-codegen.so"]\n'
        )

    def test_non_authority_doctest_build_args_remain_allowed(self) -> None:
        directory, root = _workspace_with_config(
            '[build]\nrustdocflags = ["-Zunstable-options", "--doctest-build-arg=--cfg=originweave_reviewed", "--doctest-build-arg=-Copt-level=2"]\n'
        )
        self.addCleanup(directory.cleanup)
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
