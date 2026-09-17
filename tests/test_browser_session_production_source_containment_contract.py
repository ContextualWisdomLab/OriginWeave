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


class BrowserSessionProductionSourceContainmentContractTests(unittest.TestCase):
    """Keep every Cargo production source inside exact-head repository provenance."""

    def test_current_production_source_closure_is_canonical_and_nonempty(self) -> None:
        reviewed = boundary._workspace_production_sources(ROOT)
        self.assertGreater(len(reviewed), 0)
        self.assertTrue(all(source.is_file() for source in reviewed))

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

            with self.assertRaisesRegex(
                AssertionError,
                "production Cargo source escapes repository review root",
            ):
                boundary._workspace_production_sources(root)

    def test_dangling_default_rust_source_symlink_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
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
            source.symlink_to(adapter / "missing-generated.rs")

            with self.assertRaisesRegex(
                AssertionError,
                "production Cargo source is missing",
            ):
                boundary._workspace_production_sources(root)


if __name__ == "__main__":
    unittest.main()
