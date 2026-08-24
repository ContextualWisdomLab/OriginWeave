"""Regression contract for ancestor-swap races in SPDX release input paths."""

from __future__ import annotations

import os
import pathlib
import runpy
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "release" / "validate_spdx_jsonld.py"


class ReleaseSpdxJsonLdParentSwapContractTests(unittest.TestCase):
    """Require path traversal to remain direct for the actual open operation."""

    @unittest.skipUnless(hasattr(os, "symlink") and hasattr(os, "link"), "requires links")
    def test_parent_swap_to_symlink_during_open_fails_closed(self) -> None:
        """Pre/post pathname checks must not miss a transient ancestor symlink."""

        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_parent_swap_contract")
        read_bounded = namespace["_read_bounded"]
        envelope_error = namespace["SpdxJsonLdEnvelopeError"]
        original_opener = namespace["_nonblocking_read_opener"]

        with tempfile.TemporaryDirectory(prefix="originweave-spdx-parent-swap-") as directory:
            root = pathlib.Path(directory)
            direct_directory = root / "direct"
            direct_directory.mkdir()
            actual_directory = root / "actual"
            actual_directory.mkdir()

            actual_candidate = actual_directory / "candidate.spdx.jsonld"
            actual_candidate.write_bytes(b"{\"indirect\":true}")
            direct_candidate = direct_directory / "candidate.spdx.jsonld"
            os.link(actual_candidate, direct_candidate)
            parked_directory = root / "direct-parked"

            def swapping_opener(path: str, flags: int) -> int:
                direct_directory.rename(parked_directory)
                direct_directory.symlink_to(actual_directory.name, target_is_directory=True)
                try:
                    return original_opener(path, flags)
                finally:
                    direct_directory.unlink()
                    parked_directory.rename(direct_directory)

            read_bounded.__globals__["_nonblocking_read_opener"] = swapping_opener
            try:
                with self.assertRaises(envelope_error) as captured:
                    read_bounded(direct_candidate)
            finally:
                read_bounded.__globals__["_nonblocking_read_opener"] = original_opener

            self.assertEqual(captured.exception.code, "invalid_file_type")


if __name__ == "__main__":
    unittest.main()
