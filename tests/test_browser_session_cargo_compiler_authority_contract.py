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
LINKER_PLUGIN_OPTIONS = frozenset({"-plugin", "--plugin"})
LINKER_SCRIPT_OPTIONS = frozenset({"-T", "--script"})


def _linker_driver_argument_selects_executable(argument: str) -> bool:
    """Return whether one compiler-driver argument can re-select driver execution authority."""
    if argument.startswith("@"):
        return True
    if argument == "-specs" or argument.startswith("-specs="):
        return True
    if argument == "-wrapper":
        return True
    if argument.startswith("-fuse-ld="):
        return True
    return argument == "-B" or argument.startswith("-B")


def _linker_argument_extends_external_inputs(argument: str) -> bool:
    """Return whether one compiler/linker-driver argument widens external library inputs."""
    if argument in {"-L", "-l"}:
        return True
    if argument.startswith("-L") and len(argument) > 2:
        return True
    return argument.startswith("-l") and len(argument) > 2 and not argument.startswith("--")


def _linker_option_loads_plugin(argument: str) -> bool:
    """Return whether one direct linker option requests dynamically loaded plugin code."""
    if argument in LINKER_PLUGIN_OPTIONS:
        return True
    return argument.startswith(("-plugin=", "--plugin="))


def _linker_option_selects_script(argument: str) -> bool:
    """Return whether one linker option selects a script that can introduce link inputs."""
    if argument in LINKER_SCRIPT_OPTIONS:
        return True
    return (argument.startswith("-T") and len(argument) > 2) or argument.startswith("--script=")


def _linker_option_uses_response_file(argument: str) -> bool:
    """Return whether a direct-linker argument delegates parsing to an opaque response file."""
    return argument.startswith("@")


def _forwarded_linker_arguments(argument: str) -> tuple[str, ...]:
    """Return direct-linker arguments encoded by a single compiler-driver forwarding option."""
    if argument.startswith("-Wl,"):
        return tuple(argument.removeprefix("-Wl,").split(","))
    if argument.startswith("--for-linker="):
        return tuple(argument.removeprefix("--for-linker=").split(","))
    return ()


def _linker_driver_arguments_select_executable(arguments: list[str]) -> bool:
    """Return whether driver arguments can replace tools, extend link inputs, or load linker code."""
    for index, argument in enumerate(arguments):
        if _linker_driver_argument_selects_executable(argument):
            return True
        if _linker_argument_extends_external_inputs(argument):
            return True
        if _linker_option_selects_script(argument):
            return True
        if any(
            _linker_option_loads_plugin(forwarded)
            or _linker_option_selects_script(forwarded)
            or _linker_option_uses_response_file(forwarded)
            or _linker_argument_extends_external_inputs(forwarded)
            for forwarded in _forwarded_linker_arguments(argument)
        ):
            return True
        if (
            argument == "-Xlinker"
            and index + 1 < len(arguments)
            and (
                _linker_option_loads_plugin(arguments[index + 1])
                or _linker_option_selects_script(arguments[index + 1])
                or _linker_option_uses_response_file(arguments[index + 1])
                or _linker_argument_extends_external_inputs(arguments[index + 1])
            )
        ):
            return True
    return False


def _flag_arguments(value: object) -> list[str]:
    """Normalize one Cargo rustflags value without inventing shell semantics."""
    if isinstance(value, str):
        return value.split()
    if isinstance(value, list) and all(isinstance(argument, str) for argument in value):
        return value
    return []


def _flags_extend_external_link_inputs(value: object) -> bool:
    """Return whether Git-owned rustc flags widen external crate or native-library inputs."""
    return any(_linker_argument_extends_external_inputs(argument) for argument in _flag_arguments(value))


def _flags_select_linker(value: object) -> bool:
    """Return whether Cargo-owned rustc/rustdoc flags extend linker execution or input authority."""
    arguments = _flag_arguments(value)
    if not arguments:
        return False

    linker_driver_arguments: list[str] = []
    for index, argument in enumerate(arguments):
        option: str | None = None
        if argument in {"-C", "--codegen"} and index + 1 < len(arguments):
            option = arguments[index + 1]
        elif argument.startswith("-C") and len(argument) > 2:
            option = argument[2:]
        elif argument.startswith("--codegen="):
            option = argument.removeprefix("--codegen=")

        if option is None:
            continue
        if option.startswith("linker="):
            return True
        if option == "link-self-contained" or option.startswith("link-self-contained="):
            return True
        if option == "linker-features" or option.startswith("linker-features="):
            return True
        if option == "linker-flavor" or option.startswith("linker-flavor="):
            return True
        if option == "dlltool" or option.startswith("dlltool="):
            return True
        if option.startswith("link-arg="):
            linker_driver_arguments.append(option.partition("=")[2])
        elif option.startswith("link-args="):
            linker_driver_arguments.extend(option.partition("=")[2].split())

    return _linker_driver_arguments_select_executable(linker_driver_arguments)


def _assert_no_repository_cargo_compiler_execution_overrides(root: pathlib.Path) -> None:
    """Reject Git-owned Cargo settings that replace Rust tools or widen external link inputs."""
    # The trusted-adapter boundary remains the single writer for production package/source topology
    # and dependency-source overrides. This contract owns Cargo-selected execution/input authority.
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
            if _flags_extend_external_link_inputs(build.get("rustflags")):
                build_configured.append("rustflags:external link input")
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
                if _flags_extend_external_link_inputs(settings.get("rustflags")):
                    configured.append("rustflags:external link input")
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

    def test_forwarded_linker_response_file_fails_closed(self) -> None:
        for forwarded in (
            "-Wl,@tools/review-bypass-linker.rsp",
            "--for-linker=@tools/review-bypass-linker.rsp",
            "-Xlinker @tools/review-bypass-linker.rsp",
        ):
            with self.subTest(forwarded=forwarded):
                root = self._workspace_with_config(
                    f'[build]\nrustflags = ["-C", "link-args={forwarded}"]\n'
                )
                with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
                    _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_non_linker_selecting_link_arg_remains_allowed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustflags = ["-C", "link-arg=-Wl,--as-needed"]\n'
        )
        _assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_linker_forwarded_bsymbolic_remains_allowed(self) -> None:
        root = self._workspace_with_config(
            '[build]\nrustflags = ["-C", "link-arg=-Wl,-Bsymbolic"]\n'
        )
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
