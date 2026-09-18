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


class BrowserSessionRustdocRenderInputAuthorityContractTests(unittest.TestCase):
    """Keep rustdoc-rendered file inputs inside the reviewed documentation provenance boundary."""

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

    def _assert_render_input_fails_closed(self, config_text: str) -> None:
        root = self._workspace_with_config(config_text)
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_build_rustdocflags_render_file_inputs_fail_closed(self) -> None:
        selectors = (
            ("--html-in-header", "tools/review-bypass-header.html"),
            ("--html-before-content", "tools/review-bypass-before.html"),
            ("--html-after-content", "tools/review-bypass-after.html"),
            ("--markdown-before-content", "tools/review-bypass-before.md"),
            ("--markdown-after-content", "tools/review-bypass-after.md"),
            ("--extend-css", "tools/review-bypass.css"),
            ("--theme", "tools/review-bypass-theme.css"),
            ("--check-theme", "tools/review-bypass-theme.css"),
            ("-e", "tools/review-bypass-short.css"),
        )
        for selector, path in selectors:
            with self.subTest(selector=selector):
                self._assert_render_input_fails_closed(
                    f'[build]\nrustdocflags = ["{selector}", "{path}"]\n'
                )

    def test_build_rustdocflags_unstable_index_page_input_fails_closed(self) -> None:
        self._assert_render_input_fails_closed(
            '[build]\nrustdocflags = ["-Z", "unstable-options", "--index-page", "tools/review-bypass-index.md"]\n'
        )

    def test_target_rustdocflags_equals_render_file_input_fails_closed(self) -> None:
        self._assert_render_input_fails_closed(
            "[target.'cfg(unix)']\nrustdocflags = [\"--html-before-content=tools/review-bypass-before.html\"]\n"
        )

    def test_unrelated_rustdoc_render_flags_remain_allowed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustdocflags = ["--document-private-items", "--default-theme", "ayu", "--markdown-css", "reviewed.css"]\n'
        )
        authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
