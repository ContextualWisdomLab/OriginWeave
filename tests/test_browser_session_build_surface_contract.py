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


class BrowserSessionBuildSurfaceContractTests(unittest.TestCase):
    """Keep generated-code build surfaces out of the Browser Session TCB closure."""

    def _workspace(self, manifest_suffix: str = "") -> tuple[tempfile.TemporaryDirectory[str], pathlib.Path]:
        directory = tempfile.TemporaryDirectory()
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n',
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "src").mkdir(parents=True)
        (adapter / "src/lib.rs").write_text("pub fn adapter() {}\n", encoding="utf-8")
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n'
            + manifest_suffix,
            encoding="utf-8",
        )
        return directory, root

    def test_default_build_rs_fails_closed(self) -> None:
        directory, root = self._workspace()
        with directory:
            (root / "adapter/build.rs").write_text("fn main() {}\n", encoding="utf-8")
            with self.assertRaisesRegex(AssertionError, "production Cargo build script"):
                boundary._production_package_manifests(root)

    def test_custom_package_build_path_fails_closed(self) -> None:
        directory, root = self._workspace('build = "tools/generate.rs"\n')
        with directory:
            (root / "adapter/tools").mkdir()
            (root / "adapter/tools/generate.rs").write_text("fn main() {}\n", encoding="utf-8")
            with self.assertRaisesRegex(AssertionError, "production Cargo build script"):
                boundary._production_package_manifests(root)

    def test_build_dependencies_fail_closed(self) -> None:
        directory, root = self._workspace(
            '[build-dependencies]\nserde = "1"\n'
        )
        with directory:
            with self.assertRaisesRegex(AssertionError, "production Cargo build dependencies"):
                boundary._production_package_manifests(root)

    def test_target_specific_build_dependencies_fail_closed(self) -> None:
        directory, root = self._workspace(
            "[target.'cfg(unix)'.build-dependencies]\nserde = \"1\"\n"
        )
        with directory:
            with self.assertRaisesRegex(AssertionError, "production Cargo build dependencies"):
                boundary._production_package_manifests(root)


if __name__ == "__main__":
    unittest.main()
