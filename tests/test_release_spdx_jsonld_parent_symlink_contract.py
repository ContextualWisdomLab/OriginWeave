"""Regression contract for indirect SPDX JSON-LD release parent paths."""

from __future__ import annotations

import pathlib
import runpy
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "release" / "validate_spdx_jsonld.py"


class ReleaseSpdxJsonLdParentSymlinkContractTests(unittest.TestCase):
    """Require every admitted release-artifact path component to be direct."""

    def test_symlinked_parent_directory_is_rejected_as_indirect_release_input(self) -> None:
        """An ancestor symlink may not redirect release-artifact path authority."""

        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_parent_symlink_contract")
        read_bounded = namespace["_read_bounded"]
        envelope_error = namespace["SpdxJsonLdEnvelopeError"]

        with tempfile.TemporaryDirectory(prefix="originweave-spdx-parent-symlink-") as directory:
            root = pathlib.Path(directory)
            actual_directory = root / "actual"
            actual_directory.mkdir()
            (actual_directory / "candidate.spdx.jsonld").write_bytes(b"{}")
            alias_directory = root / "alias"
            alias_directory.symlink_to(actual_directory.name, target_is_directory=True)

            with self.assertRaises(envelope_error) as captured:
                read_bounded(alias_directory / "candidate.spdx.jsonld")

            self.assertEqual(captured.exception.code, "invalid_file_type")


if __name__ == "__main__":
    unittest.main()
