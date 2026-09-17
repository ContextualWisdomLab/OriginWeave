import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BOUNDARY_TEST = ROOT / "tests/test_browser_session_trusted_adapter_boundary.py"

spec = importlib.util.spec_from_file_location("browser_session_trusted_adapter_boundary", BOUNDARY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session trusted-adapter boundary contract")
boundary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boundary)


def _reviewed_production_sources(root: pathlib.Path) -> list[pathlib.Path]:
    """Validate canonical Cargo production sources against exact-head repository provenance."""
    root_resolved = root.resolve()
    reviewed: list[pathlib.Path] = []
    for source in boundary._workspace_production_sources(root):
        resolved = source.resolve()
        try:
            resolved.relative_to(root_resolved)
        except ValueError as exc:
            raise AssertionError(
                f"production Cargo source escapes repository review root: {source}"
            ) from exc
        if not resolved.is_file():
            raise AssertionError(f"production Cargo source is missing: {source}")
        reviewed.append(source)
    return reviewed


class BrowserSessionProductionSourceContainmentContractTests(unittest.TestCase):
    """Keep every Cargo production source inside exact-head repository provenance."""

    def test_current_production_source_closure_stays_under_repository_root(self) -> None:
        reviewed = _reviewed_production_sources(ROOT)
        self.assertEqual(reviewed, boundary._workspace_production_sources(ROOT))

    def test_default_rust_source_symlink_outside_repository_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            sandbox = pathlib.Path(directory)
            root = sandbox / "workspace"
            external = sandbox / "external-lifecycle-adapter.rs"
            root.mkdir()
            external.write_text(
                "use originweave_browser_session::DisposableContextPort;\n",
                encoding="utf-8",
            )
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
            source = adapter / "src/lib.rs"
            source.symlink_to(external)

            self.assertIn(source, boundary._workspace_production_sources(root))
            with self.assertRaisesRegex(
                AssertionError,
                "production Cargo source escapes repository review root",
            ):
                _reviewed_production_sources(root)


if __name__ == "__main__":
    unittest.main()
