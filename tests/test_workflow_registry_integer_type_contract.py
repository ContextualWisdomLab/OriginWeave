"""Regression contracts for strict workflow-registry integer fields."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/ci/audit_workflow_registry.py"
DEFAULT_SHA = "67af7c87589edc2039545af335c95064d9b8391c"


def _load_module():
    """Load the audit utility without making scripts a Python package."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload() -> dict:
    """Return one minimal complete audit fixture."""

    return {
        "schema_version": 1,
        "expected_default_branch_sha": DEFAULT_SHA,
        "observed_default_branch_sha": DEFAULT_SHA,
        "observed_at": "2026-08-12T00:00:00Z",
        "reported_total_count": 0,
        "protected_workflow_paths": [],
        "active_pr_workflow_paths": [],
        "registry_pages": [
            {
                "page": 1,
                "status_code": 200,
                "has_next": False,
                "workflows": [],
            }
        ],
    }


class WorkflowRegistryIntegerTypeContractTests(unittest.TestCase):
    """Reject Python booleans where the evidence schema requires integers."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_boolean_schema_version_is_rejected(self) -> None:
        """JSON true must not compare equal to schema version integer 1."""

        payload = _payload()
        payload["schema_version"] = True
        with self.assertRaises(self.audit.WorkflowAuditError):
            self.audit.audit_workflow_registry(payload)

    def test_boolean_page_number_is_rejected(self) -> None:
        """JSON true must not compare equal to registry page integer 1."""

        payload = _payload()
        payload["registry_pages"][0]["page"] = True
        with self.assertRaises(self.audit.WorkflowAuditError):
            self.audit.audit_workflow_registry(payload)


if __name__ == "__main__":
    unittest.main()
