"""Regression contract for bounded SPDX JSON-LD release input file types."""

from __future__ import annotations

import os
import pathlib
import runpy
import tempfile
import threading
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "release" / "validate_spdx_jsonld.py"


class ReleaseSpdxJsonLdFileTypeContractTests(unittest.TestCase):
    """Prevent streaming/indirect files from bypassing the bounded release-input contract."""

    def test_fifo_candidate_is_rejected_before_document_bytes_are_accepted(self) -> None:
        """A named pipe is not an immutable bounded release artifact input."""

        if not hasattr(os, "mkfifo"):
            self.fail("the release verifier file-type regression requires POSIX mkfifo support")

        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_file_type_contract")
        read_bounded = namespace["_read_bounded"]
        envelope_error = namespace["SpdxJsonLdEnvelopeError"]

        with tempfile.TemporaryDirectory(prefix="originweave-spdx-fifo-") as directory:
            candidate = pathlib.Path(directory) / "candidate.spdx.jsonld"
            os.mkfifo(candidate)

            writer_started = threading.Event()

            def write_candidate() -> None:
                writer_started.set()
                with candidate.open("wb") as sink:
                    sink.write(b"{}")

            writer = threading.Thread(target=write_candidate, daemon=True)
            writer.start()
            self.assertTrue(writer_started.wait(timeout=1.0))

            try:
                with self.assertRaises(envelope_error) as captured:
                    read_bounded(candidate)
            finally:
                if writer.is_alive():
                    with candidate.open("rb", buffering=0) as release_reader:
                        release_reader.read(2)
                writer.join(timeout=1.0)

            self.assertFalse(writer.is_alive(), "FIFO writer remained blocked after rejection")
            self.assertEqual(captured.exception.code, "invalid_file_type")

    def test_symlink_candidate_is_rejected_instead_of_following_indirect_release_input(self) -> None:
        """A symlink is not the regular local artifact path whose bytes are being admitted."""

        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_symlink_contract")
        read_bounded = namespace["_read_bounded"]
        envelope_error = namespace["SpdxJsonLdEnvelopeError"]

        with tempfile.TemporaryDirectory(prefix="originweave-spdx-symlink-") as directory:
            root = pathlib.Path(directory)
            target = root / "actual.spdx.jsonld"
            target.write_bytes(b"{}")
            candidate = root / "candidate.spdx.jsonld"
            candidate.symlink_to(target.name)

            with self.assertRaises(envelope_error) as captured:
                read_bounded(candidate)

            self.assertEqual(captured.exception.code, "invalid_file_type")

    def test_candidate_path_replaced_after_open_is_rejected(self) -> None:
        """The admitted descriptor must still own the candidate path after the bounded read."""

        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_leaf_swap_contract")
        read_bounded = namespace["_read_bounded"]
        envelope_error = namespace["SpdxJsonLdEnvelopeError"]
        original_opener = namespace["_nonblocking_read_opener"]

        with tempfile.TemporaryDirectory(prefix="originweave-spdx-leaf-swap-") as directory:
            root = pathlib.Path(directory)
            candidate = root / "candidate.spdx.jsonld"
            displaced = root / "displaced.spdx.jsonld"
            candidate.write_bytes(b'{"@context":"https://spdx.org/rdf/3.0.1/spdx-context.jsonld","@graph":[{"type":"SpdxDocument"}]}')

            def swap_after_open(path: str, flags: int) -> int:
                descriptor = original_opener(path, flags)
                candidate.rename(displaced)
                candidate.write_bytes(b"{}")
                return descriptor

            read_bounded.__globals__["_nonblocking_read_opener"] = swap_after_open

            with self.assertRaises(envelope_error) as captured:
                read_bounded(candidate)

            self.assertEqual(captured.exception.code, "invalid_file_type")
            self.assertEqual(candidate.read_bytes(), b"{}")
            self.assertTrue(displaced.exists())


if __name__ == "__main__":
    unittest.main()
