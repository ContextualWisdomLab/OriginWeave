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
TARGET_EXECUTION_KEYS = frozenset({"linker", "runner"})


def _flags_select_linker(value: object) -> bool:
    """Return whether Cargo-owned rustc/rustdoc flags select a linker executable."""
    if isinstance(value, str):
        arguments = value.split()
    elif isinstance(value, list) and all(isinstance(argument, str) for argument in value):
        arguments = value
    else:
        return False

    for index, argument in enumerate(arguments):
        if argument.startswith(("-Clinker=", "--codegen=linker=")):
            return True
        if argument in {"-C", "--codegen"} and index + 1 < len(arguments):
            if arguments[index + 1].startswith("linker="):
                return True
    return False


def _assert_no_repository_cargo_compiler_execution_overrides(root: pathlib.Path) -> None:
    """Reject Git-owned Cargo settings that replace Rust-tool or target executables."""
    # The trusted-adapter boundary remains the single writer for production package/source topology
    # and dependency-source overrides. This contract owns Cargo-selected execution authority.
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
        build_configured = (
            sorted(COMPILER_EXECUTION_KEYS.intersection(build)) if isinstance(build, dict) else []
        )
        if isinstance(build, dict):
            if _flags_select_linker(build.get("rustflags")):
                build_configured.append("rustflags:codegen linker")
            if _flags_select_linker(build.get("rustdocflags")):
                build_configured.append("rustdocflags:codegen linker")

        target_configured: dict[str, list[str]] = {}
        target = parsed.get("target")
        if isinstance(target, dict):
            for target_name, settings in target.items():
                if not isinstance(settings, dict):
                    continue
                configured = sorted(TARGET_EXECUTION_KEYS.intersection(settings))
                if _flags_select_linker(settings.get("rustflags")):
                    configured.append("rustflags:codegen linker")
                if _flags_select_linker(settings.get("rustdocflags")):
                    configured.append("rustdocflags:codegen linker")
                if configured:
                    target_configured[str(target_name)] = configured

        if build_configured or target_configured:
            relative = config_path.relative_to(root).as_posix()
            raise AssertionError(
                "Cargo Rust tool/target execution override requires an explicit Browser Session provenance contract: "
                f"{relative} build_keys={build_configured} target_keys={target_configured}"
            )


class BrowserSessionCargoCompilerAuthorityContractTests(unittest.TestCase):
    """Keep Git-owned Cargo executable selection inside the reviewed Browser Session TCB."""

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
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
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

    def test_repository_target_linker_fails_closed(self) -> None:
        root = self._workspace_with_config(
            '[target.x86_64-unknown-linux-gnu]\nlinker = "tools/review-bypass-linker"\n'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_target_runner_fails_closed(self) -> None:
        root = self._workspace_with_config(
            "[target.'cfg(unix)']\nrunner = \"tools/review-bypass-runner\"\n"
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_build_rustflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustflags = ["-C", "linker=tools/review-bypass-linker"]\n'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_target_rustflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            "[target.'cfg(unix)']\nrustflags = \"-C linker=tools/review-bypass-linker\"\n"
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_compact_rustflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustflags = ["-Clinker=tools/review-bypass-linker"]\n'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_long_build_rustflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustflags = ["--codegen", "linker=tools/review-bypass-linker"]\n'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_long_target_rustflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            "[target.'cfg(unix)']\nrustflags = \"--codegen=linker=tools/review-bypass-linker\"\n"
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_build_rustdocflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustdocflags = ["-C", "linker=tools/review-bypass-linker"]\n'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_target_rustdocflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            "[target.'cfg(unix)']\nrustdocflags = \"-Clinker=tools/review-bypass-linker\"\n"
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_long_build_rustdocflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustdocflags = ["--codegen", "linker=tools/review-bypass-linker"]\n'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_repository_long_target_rustdocflags_linker_override_fails_closed(self) -> None:
        root = self._workspace_with_config(
            "[target.'cfg(unix)']\nrustdocflags = \"--codegen=linker=tools/review-bypass-linker\"\n"
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_unrelated_rustflags_remain_allowed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustflags = ["-C", "opt-level=2", "--cfg", "originweave_reviewed"]\n'
        )
        _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_unrelated_rustdocflags_remain_allowed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustdocflags = ["--document-private-items", "--cfg", "docsrs"]\n'
        )
        _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_unrelated_build_configuration_remains_allowed(self) -> None:
        root = self._workspace_with_config('[build]\njobs = 2\nincremental = false\n')
        _assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
