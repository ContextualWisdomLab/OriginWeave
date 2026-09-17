import importlib.util
import pathlib
import tempfile
import tomllib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BOUNDARY_TEST = ROOT / "tests/test_browser_session_trusted_adapter_boundary.py"

spec = importlib.util.spec_from_file_location("browser_session_trusted_adapter_boundary", BOUNDARY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session trusted-adapter boundary contract")
boundary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boundary)

COMPILER_EXECUTION_KEYS = frozenset(
    {"rustc", "rustc-wrapper", "rustc-workspace-wrapper", "rustdoc"}
)


def _assert_no_repository_cargo_compiler_execution_overrides(root: pathlib.Path) -> None:
    """Reject Git-owned Cargo settings that replace or wrap Rust tool executables."""
    # The trusted-adapter boundary remains the single writer for production package/source topology
    # and dependency-source overrides. This contract owns Cargo's Rust tool-execution authority.
    boundary._production_package_manifests(root)

    root_resolved = root.resolve()
    config_paths: set[pathlib.Path] = set()
    for pattern in (".cargo/config.toml", ".cargo/config"):
        config_paths.update(root.rglob(pattern))

    for config_path in sorted(config_paths):
        resolved = config_path.resolve()
        try:
            resolved.relative_to(root_resolved)
        except ValueError as exc:
            raise AssertionError(
                f"Cargo compiler config escapes repository review root: {config_path.relative_to(root).as_posix()}"
            ) from exc
        if not resolved.is_file():
            raise AssertionError(
                f"Cargo compiler config is missing: {config_path.relative_to(root).as_posix()}"
            )

        parsed = tomllib.loads(resolved.read_text(encoding="utf-8"))
        build = parsed.get("build")
        if not isinstance(build, dict):
            continue
        configured = sorted(COMPILER_EXECUTION_KEYS.intersection(build))
        if configured:
            relative = config_path.relative_to(root).as_posix()
            raise AssertionError(
                "Cargo compiler execution override requires an explicit Browser Session provenance contract: "
                f"{relative} keys={configured}"
            )


class BrowserSessionCargoCompilerAuthorityContractTests(unittest.TestCase):
    """Keep Git-owned Cargo Rust tool execution inside the reviewed Browser Session TCB."""

    def _workspace_with_config(
        self,
        config_text: str,
        *,
        config_name: str = "config.toml",
        nested: bool = False,
    ) -> pathlib.Path:
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
        config_root = adapter if nested else root
        cargo = config_root / ".cargo"
        cargo.mkdir()
        (cargo / config_name).write_text(config_text, encoding="utf-8")
        return root

    def _assert_compiler_override_fails_closed(
        self,
        config_text: str,
        *,
        config_name: str = "config.toml",
        nested: bool = False,
    ) -> None:
        root = self._workspace_with_config(
            config_text,
            config_name=config_name,
            nested=nested,
        )
        with self.assertRaisesRegex(AssertionError, "Cargo compiler execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_current_repository_has_no_unmodeled_cargo_compiler_execution_override(self) -> None:
        _assert_no_repository_cargo_compiler_execution_overrides(ROOT)

    def test_repository_rustc_wrapper_fails_closed(self) -> None:
        self._assert_compiler_override_fails_closed(
            '[build]\nrustc-wrapper = "tools/review-bypass-wrapper"\n'
        )

    def test_repository_rustc_workspace_wrapper_fails_closed(self) -> None:
        self._assert_compiler_override_fails_closed(
            '[build]\nrustc-workspace-wrapper = "tools/workspace-wrapper"\n'
        )

    def test_repository_custom_rustc_fails_closed(self) -> None:
        self._assert_compiler_override_fails_closed(
            '[build]\nrustc = "tools/custom-rustc"\n'
        )

    def test_repository_custom_rustdoc_fails_closed(self) -> None:
        self._assert_compiler_override_fails_closed(
            '[build]\nrustdoc = "tools/review-bypass-rustdoc"\n'
        )

    def test_nested_extensionless_cargo_config_compiler_override_fails_closed(self) -> None:
        self._assert_compiler_override_fails_closed(
            '[build]\nrustc-wrapper = "tools/nested-wrapper"\n',
            config_name="config",
            nested=True,
        )

    def test_unrelated_build_configuration_remains_allowed(self) -> None:
        root = self._workspace_with_config('[build]\njobs = 2\nincremental = false\n')
        _assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
