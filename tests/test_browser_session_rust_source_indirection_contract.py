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


INCLUDE_MACRO = re.compile(r"(?<![A-Za-z0-9_])include\s*!\s*[([{]")
RUST_ATTRIBUTE = re.compile(r"#\s*\[([^\]]*)\]", re.DOTALL)
PATH_META = re.compile(r"\bpath\s*=")
APPROVED_RUST_PATH_ATTRIBUTES = {
    ("crates/originweave-core/src/root.rs", 'path = "lib.rs"'),
}


def _normalized_attribute_body(body: str) -> str:
    return " ".join(body.split())


def _assert_no_unmodeled_rust_source_indirection(root: pathlib.Path) -> None:
    """Fail closed when reviewed Rust source can pull unmodeled executable source bytes."""
    discovered_path_attributes: set[tuple[str, str]] = set()

    for source in boundary._workspace_production_sources(root):
        text = source.read_text(encoding="utf-8")
        relative = source.relative_to(root).as_posix()

        if INCLUDE_MACRO.search(text):
            raise AssertionError(
                f"Rust include! source indirection requires an explicit provenance contract: {relative}"
            )

        for match in RUST_ATTRIBUTE.finditer(text):
            attribute_body = _normalized_attribute_body(match.group(1))
            if PATH_META.search(attribute_body):
                discovered_path_attributes.add((relative, attribute_body))

    unexpected = discovered_path_attributes - APPROVED_RUST_PATH_ATTRIBUTES
    if unexpected:
        raise AssertionError(
            "Rust path attribute requires an explicit exact-tree provenance review: "
            f"{sorted(unexpected)}"
        )

    stale = APPROVED_RUST_PATH_ATTRIBUTES - discovered_path_attributes
    if root.resolve() == ROOT.resolve() and stale:
        raise AssertionError(
            f"Rust path-attribute allowlist preapproves absent production surfaces: {sorted(stale)}"
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

    def _assert_include_form_fails_closed(self, source_text: str) -> None:
        root = self._workspace_with_source(source_text)
        (root / "adapter/generated_adapter.rs").write_text(
            "pub fn generated_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust include! source indirection"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_current_production_sources_have_no_unmodeled_source_indirection(self) -> None:
        _assert_no_unmodeled_rust_source_indirection(ROOT)

    def test_parenthesized_include_macro_fails_closed(self) -> None:
        self._assert_include_form_fails_closed('include!("../generated_adapter.rs");\n')

    def test_braced_include_macro_fails_closed(self) -> None:
        self._assert_include_form_fails_closed('include! { "../generated_adapter.rs" }\n')

    def test_bracketed_include_macro_fails_closed(self) -> None:
        self._assert_include_form_fails_closed('include!["../generated_adapter.rs"];\n')

    def test_bare_module_from_custom_target_fails_closed(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n',
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "runtime").mkdir(parents=True)
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n'
            '[lib]\npath = "runtime/lifecycle_adapter.rs"\n',
            encoding="utf-8",
        )
        (adapter / "runtime/lifecycle_adapter.rs").write_text(
            "mod helper;\npub fn lifecycle_adapter_surface() {}\n",
            encoding="utf-8",
        )
        (adapter / "runtime/helper.rs").write_text(
            "pub fn helper_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust module source indirection"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_new_path_attribute_fails_closed_even_when_target_is_in_tree(self) -> None:
        root = self._workspace_with_source('#[path = "nested.rs"]\nmod nested;\n')
        (root / "adapter/src/nested.rs").write_text(
            "pub fn nested_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust path attribute requires"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_cfg_attr_generated_path_attribute_fails_closed(self) -> None:
        root = self._workspace_with_source(
            '#[cfg_attr(unix, path = "unix_adapter.rs")]\nmod platform_adapter;\n'
        )
        (root / "adapter/src/unix_adapter.rs").write_text(
            "pub fn unix_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust path attribute requires"):
            _assert_no_unmodeled_rust_source_indirection(root)

    def test_parent_traversal_path_attribute_fails_closed(self) -> None:
        root = self._workspace_with_source('#[path = "../shared_adapter.rs"]\nmod shared_adapter;\n')
        (root / "adapter/shared_adapter.rs").write_text(
            "pub fn shared_adapter_surface() {}\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(AssertionError, "Rust path attribute requires"):
            _assert_no_unmodeled_rust_source_indirection(root)


if __name__ == "__main__":
    unittest.main()
