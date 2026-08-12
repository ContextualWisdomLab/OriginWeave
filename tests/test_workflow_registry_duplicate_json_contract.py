"""Reject ambiguous duplicate-key JSON in workflow registry audit inputs."""

from __future__ import annotations

import importlib.util
import pathlib
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/ci/audit_workflow_registry.py"
DEFAULT_SHA = "67af7c87589edc2039545af335c95064d9b8391c"


def _load_module():
    """Load the read-only workflow audit utility without packaging scripts."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class WorkflowRegistryDuplicateJsonContractTests(unittest.TestCase):
    """Require duplicate object names to fail before semantic audit validation."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_duplicate_top_level_name_fails_closed(self) -> None:
        """A later duplicate JSON member must not silently replace prior evidence."""

        document = (
            '{"schema_version":1,"schema_version":1,'
            f'"expected_default_branch_sha":"{DEFAULT_SHA}",'
            f'"observed_default_branch_sha":"{DEFAULT_SHA}",'
            '"observed_at":"2026-08-12T00:00:00Z",'
            '"reported_total_count":0,'
            '"protected_workflow_paths":[],'
            '"active_pr_workflow_paths":[],'
            '"registry_pages":[{"page":1,"status_code":200,'
            '"has_next":false,"workflows":[]}]}'
        )
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "registry.json"
            path.write_text(document, encoding="utf-8")
            with self.assertRaisesRegex(
                self.audit.WorkflowAuditError, "duplicate JSON object member"
            ):
                self.audit._read_payload(path)


if __name__ == "__main__":
    unittest.main()
