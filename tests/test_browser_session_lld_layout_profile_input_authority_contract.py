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


class BrowserSessionLldLayoutProfileInputAuthorityContractTests(unittest.TestCase):
    """Keep repository-selected LLD layout/profile files inside reviewed provenance."""

    def test_lld_layout_and_profile_file_inputs_fail_closed(self) -> None:
        hostile = (
            "--call-graph-ordering-file=tools/callgraph.order",
            "-call-graph-ordering-file=tools/callgraph.order",
            "--irpgo-profile=tools/startup.profdata",
            "--symbol-ordering-file=tools/symbols.order",
            "--lto-sample-profile=tools/sample.prof",
        )
        for linker_option in hostile:
            with self.subTest(linker_option=linker_option):
                rustflags = ["-C", f"link-arg=-Wl,{linker_option}"]
                self.assertTrue(authority._flags_select_linker(rustflags))

    def test_rustdoc_doctest_forwarding_cannot_select_lld_profile_file(self) -> None:
        rustdocflags = [
            "--doctest-build-arg=-C",
            "--doctest-build-arg=link-arg=-Wl,--lto-sample-profile=tools/sample.prof",
        ]
        self.assertTrue(authority._flags_select_rustdoc_doctest_compiler_authority(rustdocflags))

    def test_typed_linker_control_remains_allowed(self) -> None:
        self.assertFalse(authority._flags_select_linker(["-C", "link-arg=-Wl,-z,relro"]))


if __name__ == "__main__":
    unittest.main()
