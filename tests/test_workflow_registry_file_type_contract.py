"""Regression contract for bounded workflow-registry audit input file types."""

from __future__ import annotations

import os
import pathlib
import runpy
import tempfile
import threading
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
AUDITOR = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"


class WorkflowRegistryFileTypeContractTests(unittest.TestCase):
    """Prevent indirect or streaming OS file types from becoming audit evidence."""

    def test_fifo_input_is_rejected_before_registry_bytes_are_accepted(self) -> None:
        """A named pipe is not immutable operator-collected registry evidence."""

        if not hasattr(os, "mkfifo"):
            self.fail("the workflow audit file-type regression requires POSIX mkfifo support")

        namespace = runpy.run_path(str(AUDITOR), run_name="workflow_file_type_contract")
        read_payload = namespace["_read_payload"]
        audit_error = namespace["WorkflowAuditError"]

        with tempfile.TemporaryDirectory(prefix="originweave-workflow-fifo-") as directory:
            candidate = pathlib.Path(directory) / "registry.json"
            os.mkfifo(candidate)

            writer_started = threading.Event()

            def write_candidate() -> None:
                writer_started.set()
                try:
                    with candidate.open("wb") as sink:
                        sink.write(b"{}")
                except BrokenPipeError:
                    # Expected when the reader rejects the FIFO before consuming bytes.
                    return

            writer = threading.Thread(target=write_candidate, daemon=True)
            writer.start()
            self.assertTrue(writer_started.wait(timeout=1.0))

            try:
                with self.assertRaisesRegex(audit_error, "input must be a regular file"):
                    read_payload(candidate)
            finally:
                if writer.is_alive():
                    with candidate.open("rb", buffering=0) as release_reader:
                        release_reader.read(2)
                writer.join(timeout=1.0)

            self.assertFalse(writer.is_alive(), "FIFO writer remained blocked after rejection")

    def test_symbolic_link_input_is_rejected_before_target_bytes_are_accepted(self) -> None:
        """Audit evidence must name the collected regular file directly, not through a symlink."""

        namespace = runpy.run_path(str(AUDITOR), run_name="workflow_symlink_contract")
        read_payload = namespace["_read_payload"]
        audit_error = namespace["WorkflowAuditError"]

        with tempfile.TemporaryDirectory(prefix="originweave-workflow-symlink-") as directory:
            root = pathlib.Path(directory)
            target = root / "collected-registry.json"
            target.write_text("{}", encoding="utf-8")
            candidate = root / "registry.json"
            try:
                candidate.symlink_to(target)
            except OSError as error:
                self.skipTest(f"symbolic links are unavailable on this platform: {error}")

            with self.assertRaisesRegex(audit_error, "input must not be a symbolic link"):
                read_payload(candidate)


if __name__ == "__main__":
    unittest.main()
