import importlib.util
import pathlib
import re
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BOUNDARY_TEST = ROOT / "tests/test_browser_session_trusted_adapter_boundary.py"

spec = importlib.util.spec_from_file_location("browser_session_trusted_adapter_boundary", BOUNDARY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session trusted-adapter boundary contract")
boundary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boundary)


INCLUDE_MACRO = re.compile(r"(?<![A-Za-z0-9_])include\s*!\s*\(")
PATH_ATTRIBUTE = re.compile(r'#\s*\[\s*path\s*=\s*"([^"]+)"\s*\]')
WINDOWS_ABSOLUTE_PATH = re.compile(r"^[A-Za-z]:[/\\]")


def _assert_no_unmodeled_rust_source_indirection(root: pathlib.Path) -> None:
    """Fail closed when reviewed Rust source can pull unmodeled executable source bytes."""
    for source in boundary._workspace_production_sources(root):
        text = source.read_text(encoding="utf-8")
        relative = source.relative_to(root).as_posix()

        if INCLUDE_MACRO.search(text):
            raise AssertionError(
                f"Rust include! source indirection requires an explicit provenance contract: {relative}"
            )

        for match in PATH_ATTRIBUTE.finditer(text):
            declared_path = match.group(1)
            normalized = declared_path.replace("\\", "/")
            parts = pathlib.PurePosixPath(normalized).parts
            if (
                pathlib.PurePosixPath(normalized).is_absolute()
                or WINDOWS_ABSOLUTE_PATH.match(declared_path)
                or ".." in parts
            ):
                raise AssertionError(
                    "Rust path attribute escapes canonical source closure and requires an explicit provenance contract: "
                    f"{relative} -> {declared_path}"
                )


class BrowserSessionRustSourceIndirectionContractTests(unittest.TestCase):
    """Keep Rust source indirection inside the exact-head production-source provenance boundary."""

    def _workspace_with_source(self, source_text: str) -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n',
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "src").mkdir(parents=True)
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        (adapter / "src/lib.rs").write_text(source_text, encoding="utf-8")
        return root

    def test_current_production_sources_have_no_unmodeled_source_indirection(self) -> None:
        _assert_no_unmodeled_rust_source_indirection(ROOT)

    def test_include_macro_fails_closed_until_included_source_provenance_is_modeled(self) -> None:
        root = self._workspace_with_source('include!("../generated_adapter.rs");\n')
        (root / "adapter/generated_adapter.rs").write_text(
            "pub fn generated_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust include! source indirection"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_parent_traversal_path_attribute_fails_closed_until_module_provenance_is_modeled(self) -> None:
        root = self._workspace_with_source('#[path = "../shared_adapter.rs"]\nmod shared_adapter;\n')
        (root / "adapter/shared_adapter.rs").write_text(
            "pub fn shared_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust path attribute escapes canonical source closure"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_in_tree_relative_path_attribute_remains_allowed(self) -> None:
        root = self._workspace_with_source('#[path = "nested.rs"]\nmod nested;\n')
        (root / "adapter/src/nested.rs").write_text(
            "pub fn nested_adapter_surface() {}\n",
            encoding="utf-8",
        )

        _assert_no_unmodeled_rust_source_indirection(root)


if __name__ == "__main__":
    unittest.main()
