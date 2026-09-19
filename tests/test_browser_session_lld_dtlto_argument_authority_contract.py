import importlib.util
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location("browser_session_cargo_compiler_authority", AUTHORITY_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session compiler authority contract")
authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(authority)


class BrowserSessionLldDtltoArgumentAuthorityContractTests(unittest.TestCase):
    """Keep DTLTO subprocess arguments inside reviewed linker execution provenance."""

    def test_dtlto_subprocess_argument_forwarding_fails_closed(self) -> None:
        hostile_options = (
            "--thinlto-distributor-arg=tools/unreviewed-distributor.py",
            "--thinlto-remote-compiler-prepend-arg=-fplugin=tools/unreviewed-pass.so",
            "--thinlto-remote-compiler-arg=-fplugin=tools/unreviewed-pass.so",
        )
        for option in hostile_options:
            with self.subTest(option=option):
                rustflags = ["-C", f"link-arg=-Wl,{option}"]
                self.assertTrue(authority._flags_select_linker(rustflags))

    def test_separated_dtlto_subprocess_argument_fails_closed(self) -> None:
        rustflags = [
            "-C",
            "link-arg=-Wl,--thinlto-remote-compiler-arg,--target=riscv64-unknown-linux-gnu",
        ]
        self.assertTrue(authority._flags_select_linker(rustflags))

    def test_rustdoc_doctest_forwarding_cannot_select_dtlto_subprocess_arguments(self) -> None:
        rustdocflags = [
            "--doctest-build-arg=-C",
            "--doctest-build-arg=link-arg=-Wl,--thinlto-remote-compiler-arg=-fplugin=tools/unreviewed-pass.so",
        ]
        self.assertTrue(authority._flags_select_rustdoc_doctest_compiler_authority(rustdocflags))

    def test_typed_linker_control_remains_allowed(self) -> None:
        rustflags = ["-C", "link-arg=-Wl,-z,relro"]
        self.assertFalse(authority._flags_select_linker(rustflags))


if __name__ == "__main__":
    unittest.main()
