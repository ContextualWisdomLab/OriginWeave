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


class BrowserSessionLldCmseImportLibraryAuthorityContractTests(unittest.TestCase):
    """Keep repository-selected ARM CMSE import libraries inside reviewed provenance."""

    def test_joined_cmse_input_import_library_fails_closed(self) -> None:
        rustflags = [
            "-C",
            "link-arg=-Wl,--in-implib=tools/previous-secure-image.lib",
        ]
        self.assertTrue(authority._flags_select_linker(rustflags))

    def test_split_cmse_input_import_library_fails_closed(self) -> None:
        rustflags = [
            "-C",
            "link-args=-Wl,--in-implib tools/previous-secure-image.lib",
        ]
        self.assertTrue(authority._flags_select_linker(rustflags))

    def test_rustdoc_doctest_forwarding_cannot_select_cmse_import_library(self) -> None:
        rustdocflags = [
            "--doctest-build-arg=-C",
            "--doctest-build-arg=link-arg=-Wl,--in-implib=tools/previous-secure-image.lib",
        ]
        self.assertTrue(authority._flags_select_rustdoc_doctest_compiler_authority(rustdocflags))

    def test_cmse_output_import_library_remains_output_only(self) -> None:
        self.assertFalse(
            authority._flags_select_linker(
                ["-C", "link-arg=-Wl,--out-implib=artifacts/secure-image.lib"]
            )
        )


if __name__ == "__main__":
    unittest.main()
