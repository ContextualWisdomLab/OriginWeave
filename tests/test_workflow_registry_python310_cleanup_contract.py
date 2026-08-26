"""Regression contract for Python 3.10-safe workflow-audit cleanup failures."""

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


class WorkflowRegistryPython310CleanupContractTests(unittest.TestCase):
    """Keep secondary cleanup diagnostics from replacing the causal write failure."""

    def test_cleanup_failure_does_not_require_exception_add_note(self) -> None:
        """The recovery path must remain correct when BaseException.add_note is absent."""

        namespace = runpy.run_path(
            str(AUDITOR), run_name="workflow_python310_cleanup_contract"
        )
        write_output = namespace["_write_output"]
        workflow_audit_error = namespace["WorkflowAuditError"]
        function_globals = write_output.__globals__
        original_open_parent = function_globals["_open_output_parent"]
        parent_descriptor: int | None = None

        class FailingDestination:
            def __init__(self, descriptor: int) -> None:
                self.descriptor = descriptor

            def __enter__(self):
                return self

            def __exit__(self, _type, _value, _traceback) -> bool:
                os.close(self.descriptor)
                return False

            def write(self, _serialized: str) -> None:
                os.write(self.descriptor, b"{")
                raise OSError("simulated primary write failure")

        def capture_output_parent(path: pathlib.Path) -> tuple[int, str]:
            nonlocal parent_descriptor
            parent_descriptor, leaf_name = original_open_parent(path)
            return parent_descriptor, leaf_name

        def fail_identity_cleanup(
            _parent_fd: int, _staging_name: str, _expected_identity: tuple[int, int]
        ) -> None:
            raise workflow_audit_error("simulated cleanup identity change")

        with tempfile.TemporaryDirectory(prefix="originweave-python310-cleanup-") as directory:
            output = pathlib.Path(directory) / "audit.json"
            with (
                unittest.mock.patch.dict(
                    function_globals,
                    {
                        "_open_output_parent": capture_output_parent,
                        "_unlink_matching_staging": fail_identity_cleanup,
                    },
                ),
                unittest.mock.patch.object(
                    workflow_audit_error,
                    "add_note",
                    side_effect=AttributeError("BaseException.add_note is unavailable"),
                    create=True,
                ),
                unittest.mock.patch(
                    "os.fdopen",
                    side_effect=lambda descriptor, *_args, **_kwargs: FailingDestination(
                        descriptor
                    ),
                ),
                self.assertRaises(workflow_audit_error) as raised,
            ):
                write_output(output, "{}\n")

            self.assertEqual(str(raised.exception), "output path is not writable")
            self.assertIsInstance(raised.exception.__cause__, OSError)
            self.assertIsNotNone(parent_descriptor)
            assert parent_descriptor is not None
            with self.assertRaises(OSError) as closed:
                os.fstat(parent_descriptor)
            self.assertEqual(closed.exception.errno, errno.EBADF)


if __name__ == "__main__":
    unittest.main()
