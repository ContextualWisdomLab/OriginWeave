"""Regression contract for persistent workflow-audit staging cleanup failures."""

from __future__ import annotations

import errno
import os
import pathlib
import runpy
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
AUDITOR = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"


class WorkflowRegistryPersistentCleanupContractTests(unittest.TestCase):
    """Preserve the first causal cleanup failure across a bounded final cleanup attempt."""

    def test_persistent_staging_cleanup_does_not_replace_first_failure(self) -> None:
        """A second staging cleanup failure is diagnostic, not a replacement cause."""

        namespace = runpy.run_path(
            str(AUDITOR), run_name="workflow_persistent_cleanup_contract"
        )
        write_output = namespace["_write_output"]
        workflow_audit_error = namespace["WorkflowAuditError"]
        function_globals = write_output.__globals__
        original_open_parent = function_globals["_open_output_parent"]
        original_unlink_with_retry = function_globals["_unlink_with_interrupted_retry"]
        parent_descriptor: int | None = None
        staging_cleanup_attempts = 0
        first_cleanup_failure = OSError(errno.EIO, "initial staging cleanup failure")
        repeated_cleanup_failure = OSError(errno.EBUSY, "final staging cleanup failure")

        def capture_output_parent(path: pathlib.Path) -> tuple[int, str]:
            nonlocal parent_descriptor
            parent_descriptor, leaf_name = original_open_parent(path)
            return parent_descriptor, leaf_name

        def persistently_fail_staging_cleanup(
            parent_fd: int, leaf_name: str
        ) -> OSError | None:
            nonlocal staging_cleanup_attempts
            if leaf_name.startswith(".originweave-audit-"):
                staging_cleanup_attempts += 1
                if staging_cleanup_attempts == 1:
                    return first_cleanup_failure
                return repeated_cleanup_failure
            return original_unlink_with_retry(parent_fd, leaf_name)

        with tempfile.TemporaryDirectory(prefix="originweave-workflow-output-") as directory:
            output = pathlib.Path(directory) / "audit.json"
            with (
                unittest.mock.patch.dict(
                    function_globals,
                    {
                        "_open_output_parent": capture_output_parent,
                        "_unlink_with_interrupted_retry": persistently_fail_staging_cleanup,
                    },
                ),
                self.assertRaises(workflow_audit_error) as raised,
            ):
                write_output(output, "{}\n")

            self.assertEqual(str(raised.exception), "output staging cleanup failed")
            self.assertIs(raised.exception.__cause__, first_cleanup_failure)
            self.assertEqual(staging_cleanup_attempts, 2)
            self.assertEqual(
                raised.exception.secondary_diagnostics,
                ["final staging cleanup failed: OSError"],
            )
            self.assertFalse(output.exists())
            self.assertIsNotNone(parent_descriptor)
            assert parent_descriptor is not None
            with self.assertRaises(OSError) as closed:
                os.fstat(parent_descriptor)
            self.assertEqual(closed.exception.errno, errno.EBADF)


if __name__ == "__main__":
    unittest.main()
