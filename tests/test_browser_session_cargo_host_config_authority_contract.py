import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location(
    "browser_session_cargo_compiler_authority_contract",
    AUTHORITY_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Cargo compiler-authority contract")
authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(authority)


class BrowserSessionCargoHostConfigAuthorityContractTests(unittest.TestCase):
    """Keep nightly Cargo host-target execution inside the canonical compiler-authority owner."""

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

    def _assert_host_authority_fails_closed(self, config_text: str) -> None:
        root = self._workspace_with_config(config_text)
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_host_linker_fails_closed(self) -> None:
        self._assert_host_authority_fails_closed(
            '[host]\nlinker = "tools/review-bypass-host-linker"\n'
        )

    def test_repository_host_arch_runner_fails_closed(self) -> None:
        self._assert_host_authority_fails_closed(
            '[host.x86_64-unknown-linux-gnu]\nrunner = "tools/review-bypass-host-runner"\n'
        )

    def test_repository_host_rustflags_external_input_fails_closed(self) -> None:
        self._assert_host_authority_fails_closed(
            '[host]\nrustflags = ["-C", "link-arg=-Wl,--library=review_bypass"]\n'
        )

    def test_repository_host_rustdocflags_external_input_fails_closed(self) -> None:
        self._assert_host_authority_fails_closed(
            '[host.x86_64-unknown-linux-gnu]\n'
            'rustdocflags = ["--extern=review_bypass=tools/libreview_bypass.rlib"]\n'
        )

    def test_repository_host_links_build_script_override_fails_closed(self) -> None:
        self._assert_host_authority_fails_closed(
            '[host.x86_64-unknown-linux-gnu.review_bypass]\n'
            'rustc-link-search = ["tools/review-bypass-native"]\n'
        )

    def test_repository_generic_host_links_build_script_override_fails_closed(self) -> None:
        self._assert_host_authority_fails_closed(
            '[host.review_bypass]\n'
            'rustc-link-search = ["tools/review-bypass-native"]\n'
        )

    def test_unrelated_host_rustflags_remain_allowed(self) -> None:
        root = self._workspace_with_config(
            '[host]\nrustflags = ["-C", "opt-level=2"]\n'
        )
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_unrelated_host_triple_rustflags_remain_allowed(self) -> None:
        root = self._workspace_with_config(
            '[host.x86_64-unknown-linux-gnu]\nrustflags = ["-C", "opt-level=2"]\n'
        )
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_unrelated_host_rustdocflags_remain_allowed(self) -> None:
        root = self._workspace_with_config(
            '[host]\nrustdocflags = ["--document-private-items"]\n'
        )
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
