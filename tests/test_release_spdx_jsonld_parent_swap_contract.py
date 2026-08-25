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

    @unittest.skipUnless(hasattr(os, "link"), "requires hard links")
    def test_parent_swap_after_post_read_identity_probe_fails_closed(self) -> None:
        """A final leaf check must not outlive its admitted parent identity."""

        namespace = runpy.run_path(
            str(VALIDATOR), run_name="spdx_parent_post_read_swap_contract"
        )
        read_bounded = namespace["_read_bounded"]
        envelope_error = namespace["SpdxJsonLdEnvelopeError"]
        original_parent_identities = namespace["_direct_parent_identities"]

        with tempfile.TemporaryDirectory(
            prefix="originweave-spdx-parent-post-read-swap-"
        ) as directory:
            root = pathlib.Path(directory)
            direct_directory = root / "direct"
            direct_directory.mkdir()
            replacement_directory = root / "replacement"
            replacement_directory.mkdir()
            parked_directory = root / "direct-parked"

            direct_candidate = direct_directory / "candidate.spdx.jsonld"
            direct_candidate.write_bytes(b"stable-release-sbom-bytes")
            replacement_candidate = replacement_directory / direct_candidate.name
            os.link(direct_candidate, replacement_candidate)

            probe_count = 0
            swapped = False

            def swap_after_post_read_probe(path: pathlib.Path) -> tuple[tuple[int, int], ...]:
                nonlocal probe_count, swapped
                identities = original_parent_identities(path)
                probe_count += 1
                if probe_count == 3:
                    direct_directory.rename(parked_directory)
                    replacement_directory.rename(direct_directory)
                    swapped = True
                return identities

            read_bounded.__globals__["_direct_parent_identities"] = swap_after_post_read_probe
            try:
                with self.assertRaises(envelope_error) as captured:
                    read_bounded(direct_candidate)
            finally:
                read_bounded.__globals__["_direct_parent_identities"] = (
                    original_parent_identities
                )
                if swapped:
                    direct_directory.rename(replacement_directory)
                    parked_directory.rename(direct_directory)

            self.assertEqual(captured.exception.code, "invalid_file_type")


if __name__ == "__main__":
    unittest.main()
