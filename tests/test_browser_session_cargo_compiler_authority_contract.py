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
UNSTABLE_TOOLCHAIN_INPUT_KEYS = frozenset(
    {"build-std", "build-std-features", "codegen-backend"}
)
LINKER_PLUGIN_OPTIONS = frozenset({"-plugin", "--plugin"})
LINKER_SCRIPT_OPTIONS = frozenset({"-T", "--script"})
LINKER_OPTIONS_WITH_SEPARATE_OPERAND = frozenset({"-z"})
LINKER_PLUGIN_LTO_BOOLEAN_VALUES = frozenset(
    {"y", "yes", "on", "true", "n", "no", "off", "false"}
)


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


def _rustc_argument_extends_external_inputs(argument: str) -> bool:
    """Return whether one rustc/rustdoc argument widens opaque or external compiler inputs."""
    if argument.startswith("@"):
        return True
    if argument == "--sysroot" or argument.startswith("--sysroot="):
        return True
    if argument == "--extern" or argument.startswith("--extern="):
        return True
    return _linker_argument_extends_external_inputs(argument)


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


def _codegen_option_selects_linker_plugin(option: str) -> bool:
    """Return whether one rustc codegen option names an explicit linker-plugin artifact."""
    if not option.startswith("linker-plugin-lto="):
        return False
    value = option.partition("=")[2]
    return value not in LINKER_PLUGIN_LTO_BOOLEAN_VALUES


def _codegen_option_extends_external_inputs(option: str) -> bool:
    """Return whether one rustc codegen option consumes external optimization input."""
    return option.startswith(("profile-use=", "profile-sample-use="))


def _linker_argument_is_positional_native_input(argument: str) -> bool:
    """Return whether one unconsumed linker token is a positional external input."""
    return bool(argument) and not argument.startswith("-")


def _direct_linker_arguments_extend_authority(arguments: tuple[str, ...] | list[str]) -> bool:
    """Parse direct-linker tokens without misclassifying modeled option operands as inputs."""
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument in LINKER_OPTIONS_WITH_SEPARATE_OPERAND:
            if index + 1 >= len(arguments):
                return True
            index += 2
            continue
        if (
            _linker_option_loads_plugin(argument)
            or _linker_option_selects_script(argument)
            or _linker_option_uses_response_file(argument)
            or _linker_argument_extends_external_inputs(argument)
            or _linker_argument_is_positional_native_input(argument)
        ):
            return True
        index += 1
    return False


def _forwarded_linker_arguments(argument: str) -> tuple[str, ...]:
    """Return direct-linker arguments encoded by a single compiler-driver forwarding option."""
    if argument.startswith("-Wl,"):
        return tuple(argument.removeprefix("-Wl,").split(","))
    if argument.startswith("--for-linker="):
        return tuple(argument.removeprefix("--for-linker=").split(","))
    return ()


def _linker_driver_arguments_select_executable(arguments: list[str]) -> bool:
    """Return whether driver arguments can replace tools, extend link inputs, or load linker code."""
    direct_arguments: list[str] = []
    xlinker_arguments: list[str] = []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if _linker_driver_argument_selects_executable(argument):
            return True
        if _linker_argument_extends_external_inputs(argument):
            return True

        forwarded = _forwarded_linker_arguments(argument)
        if forwarded and _direct_linker_arguments_extend_authority(forwarded):
            return True

        if argument == "-Xlinker":
            if index + 1 >= len(arguments):
                return True
            xlinker_arguments.append(arguments[index + 1])
            index += 2
            continue

        direct_arguments.append(argument)
        index += 1

    return _direct_linker_arguments_extend_authority(
        direct_arguments
    ) or _direct_linker_arguments_extend_authority(xlinker_arguments)


def _flag_arguments(value: object) -> list[str]:
    """Normalize one Cargo rustflags value without inventing shell semantics."""
    if isinstance(value, str):
        return value.split()
    if isinstance(value, list) and all(isinstance(argument, str) for argument in value):
        return value
    return []


def _flags_extend_external_link_inputs(value: object) -> bool:
    """Return whether Git-owned Rust flags widen external compiler/documentation inputs."""
    arguments = _flag_arguments(value)
    for index, argument in enumerate(arguments):
        if _rustc_argument_extends_external_inputs(argument):
            return True

        option: str | None = None
        if argument in {"-C", "--codegen"} and index + 1 < len(arguments):
            option = arguments[index + 1]
        elif argument.startswith("-C") and len(argument) > 2:
            option = argument[2:]
        elif argument.startswith("--codegen="):
            option = argument.removeprefix("--codegen=")

        if option is not None and _codegen_option_extends_external_inputs(option):
            return True
    return False


def _flags_select_codegen_backend(value: object) -> bool:
    """Return whether Git-owned flags replace or bypass the reviewed code-generation surface."""
    arguments = _flag_arguments(value)
    for index, argument in enumerate(arguments):
        if argument.startswith(("-Zcodegen-backend=", "-Zllvm-plugins=")):
            return True
        if (
            argument == "-Z"
            and index + 1 < len(arguments)
            and arguments[index + 1].startswith(("codegen-backend=", "llvm-plugins="))
        ):
            return True

        option: str | None = None
        if argument in {"-C", "--codegen"} and index + 1 < len(arguments):
            option = arguments[index + 1]
        elif argument.startswith("-C") and len(argument) > 2:
            option = argument[2:]
        elif argument.startswith("--codegen="):
            option = argument.removeprefix("--codegen=")
        if option == "llvm-args" or (option is not None and option.startswith("llvm-args=")):
            return True
    return False


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
        if _codegen_option_selects_linker_plugin(option):
            return True
        if option.startswith("link-arg="):
            linker_driver_arguments.append(option.partition("=")[2])
        elif option.startswith("link-args="):
            linker_driver_arguments.extend(option.partition("=")[2].split())

    return _linker_driver_arguments_select_executable(linker_driver_arguments)


def _flags_select_rustdoc_test_execution(value: object) -> bool:
    """Return whether Git-owned rustdoc flags select external doctest executables."""
    selectors = ("--test-runtool", "--test-builder", "--test-builder-wrapper")
    return any(
        argument in selectors or argument.startswith(tuple(f"{selector}=" for selector in selectors))
        for argument in _flag_arguments(value)
    )


def _flags_select_rustdoc_doctest_compiler_authority(value: object) -> bool:
    """Return whether rustdoc forwards authority-extending arguments to a doctest compiler."""
    arguments = _flag_arguments(value)
    forwarded: list[str] = []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument == "--doctest-build-arg":
            if index + 1 >= len(arguments):
                return True
            forwarded.append(arguments[index + 1])
            index += 2
            continue
        if argument.startswith("--doctest-build-arg="):
            forwarded.append(argument.partition("=")[2])
        index += 1

    if not forwarded:
        return False
    return (
        _flags_select_codegen_backend(forwarded)
        or _flags_select_linker(forwarded)
        or _flags_extend_external_link_inputs(forwarded)
    )


def _configured_unstable_toolchain_inputs(value: object) -> list[str]:
    """Return Git-owned unstable Cargo settings that alter compiler or standard-library inputs."""
    if not isinstance(value, dict):
        return []
    configured: list[str] = []
    for key in sorted(UNSTABLE_TOOLCHAIN_INPUT_KEYS.intersection(value)):
        setting = value[key]
        if setting is False or setting == []:
            continue
        configured.append(key)
    return configured


def _configured_profile_codegen_backends(value: object, prefix: str = "profile") -> list[str]:
    """Return Cargo profile paths that select a non-default rustc code generation backend."""
    if not isinstance(value, dict):
        return []
    configured: list[str] = []
    for key, setting in value.items():
        path = f"{prefix}.{key}"
        if key == "codegen-backend":
            if setting not in (None, ""):
                configured.append(path)
            continue
        if isinstance(setting, dict):
            configured.extend(_configured_profile_codegen_backends(setting, path))
    return sorted(configured)


def _configured_profile_rustflag_authority(value: object, prefix: str = "profile") -> list[str]:
    """Return profile rustflags that widen compiler execution or external input authority."""
    if not isinstance(value, dict):
        return []
    configured: list[str] = []
    for key, setting in value.items():
        path = f"{prefix}.{key}"
        if key == "rustflags":
            if _flags_select_codegen_backend(setting):
                configured.append(f"{path}:codegen backend")
            if _flags_select_linker(setting):
                configured.append(f"{path}:codegen linker")
            if _flags_extend_external_link_inputs(setting):
                configured.append(f"{path}:external compiler input")
            continue
        if isinstance(setting, dict):
            configured.extend(_configured_profile_rustflag_authority(setting, path))
    return sorted(configured)


def _configured_custom_target_specs(value: object) -> list[str]:
    """Return repository-selected custom rustc target specification paths."""
    if isinstance(value, str):
        targets = [value]
    elif isinstance(value, list) and all(isinstance(target, str) for target in value):
        targets = value
    else:
        return []
    return sorted(target for target in targets if target.endswith(".json"))


def _assert_no_repository_cargo_compiler_execution_overrides(root: pathlib.Path) -> None:
    """Reject Git-owned Cargo settings that replace Rust tools or widen compiler inputs."""
    # The trusted-adapter boundary remains the single writer for production package/source topology
    # and dependency-source overrides. This contract owns Cargo-selected execution/input authority.
    boundary._production_package_manifests(root)

    root_manifest_path = root / "Cargo.toml"
    root_manifest = tomllib.loads(root_manifest_path.read_text(encoding="utf-8"))
    manifest_profile_codegen_backends = _configured_profile_codegen_backends(
        root_manifest.get("profile")
    )
    manifest_profile_rustflag_authority = _configured_profile_rustflag_authority(
        root_manifest.get("profile")
    )
    if manifest_profile_codegen_backends or manifest_profile_rustflag_authority:
        raise AssertionError(
            "Cargo Rust tool/target execution override requires an explicit Browser Session provenance contract: "
            f"Cargo.toml profile_codegen_backends={manifest_profile_codegen_backends} "
            f"profile_rustflag_authority={manifest_profile_rustflag_authority}"
        )

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
        included_configs = parsed.get("include")
        include_configured = included_configs is not None
        environment = parsed.get("env")
        environment_configured = (
            sorted(str(name) for name in environment) if isinstance(environment, dict) else []
        )
        unstable_configured = _configured_unstable_toolchain_inputs(parsed.get("unstable"))
        profile_codegen_backends = _configured_profile_codegen_backends(parsed.get("profile"))
        profile_rustflag_authority = _configured_profile_rustflag_authority(parsed.get("profile"))

        build = parsed.get("build")
        build_configured = (
            sorted(COMPILER_EXECUTION_KEYS.intersection(build)) if isinstance(build, dict) else []
        )
        if isinstance(build, dict):
            build_configured.extend(
                f"target:custom target specification:{target_spec}"
                for target_spec in _configured_custom_target_specs(build.get("target"))
            )
            if _flags_select_codegen_backend(build.get("rustflags")):
                build_configured.append("rustflags:codegen backend")
            if _flags_select_linker(build.get("rustflags")):
                build_configured.append("rustflags:codegen linker")
            if _flags_extend_external_link_inputs(build.get("rustflags")):
                build_configured.append("rustflags:external link input")
            if _flags_select_codegen_backend(build.get("rustdocflags")):
                build_configured.append("rustdocflags:codegen backend")
            if _flags_select_linker(build.get("rustdocflags")):
                build_configured.append("rustdocflags:codegen linker")
            if _flags_extend_external_link_inputs(build.get("rustdocflags")):
                build_configured.append("rustdocflags:external link input")
            if _flags_select_rustdoc_test_execution(build.get("rustdocflags")):
                build_configured.append("rustdocflags:doctest execution")
            if _flags_select_rustdoc_doctest_compiler_authority(build.get("rustdocflags")):
                build_configured.append("rustdocflags:doctest compiler authority")

        target_configured: dict[str, list[str]] = {}
        target = parsed.get("target")
        if isinstance(target, dict):
            for target_name, settings in target.items():
                if not isinstance(settings, dict):
                    continue
                configured = sorted(TARGET_EXECUTION_KEYS.intersection(settings))
                linked_build_overrides = sorted(
                    str(name) for name, value in settings.items() if isinstance(value, dict)
                )
                configured.extend(
                    f"links build-script override:{name}" for name in linked_build_overrides
                )
                if _flags_select_codegen_backend(settings.get("rustflags")):
                    configured.append("rustflags:codegen backend")
                if _flags_select_linker(settings.get("rustflags")):
                    configured.append("rustflags:codegen linker")
                if _flags_extend_external_link_inputs(settings.get("rustflags")):
                    configured.append("rustflags:external link input")
                if _flags_select_codegen_backend(settings.get("rustdocflags")):
                    configured.append("rustdocflags:codegen backend")
                if _flags_select_linker(settings.get("rustdocflags")):
                    configured.append("rustdocflags:codegen linker")
                if _flags_extend_external_link_inputs(settings.get("rustdocflags")):
                    configured.append("rustdocflags:external link input")
                if _flags_select_rustdoc_test_execution(settings.get("rustdocflags")):
                    configured.append("rustdocflags:doctest execution")
                if _flags_select_rustdoc_doctest_compiler_authority(settings.get("rustdocflags")):
                    configured.append("rustdocflags:doctest compiler authority")
                if configured:
                    target_configured[str(target_name)] = configured

        if (
            build_configured
            or target_configured
            or environment_configured
            or include_configured
            or unstable_configured
            or profile_codegen_backends
            or profile_rustflag_authority
        ):
            relative = config_path.relative_to(root).as_posix()
            raise AssertionError(
                "Cargo Rust tool/target execution override requires an explicit Browser Session provenance contract: "
                f"{relative} build_keys={build_configured} target_keys={target_configured} "
                f"env_keys={environment_configured} include={include_configured} "
                f"unstable_keys={unstable_configured} profile_codegen_backends={profile_codegen_backends} "
                f"profile_rustflag_authority={profile_rustflag_authority}"
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
