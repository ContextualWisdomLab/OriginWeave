"""Regression contracts for non-null Git commit identities in audit evidence."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"
VALID_SHA = "1" * 40
ZERO_SHA = "0" * 40
OBSERVED_AT = "2026-08-18T00:00:00Z"
ACTIVE_PR_PATH = ".github/workflows/current-diagnostic.yml"


def _load_module():
    """Load the repository utility without turning scripts into a package."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload() -> dict:
    """Return one complete empty registry export bound to a real-looking commit."""

    return {
        "schema_version": 1,
        "expected_default_branch_sha": VALID_SHA,
        "observed_default_branch_sha": VALID_SHA,
        "observed_at": OBSERVED_AT,
        "reported_total_count": 0,
        "protected_workflow_paths": [],
        "active_pr_workflow_paths": [],
        "active_pr_workflow_owners": [],
        "registry_pages": [
            {
                "page": 1,
                "status_code": 200,
                "has_next": False,
                "workflows": [],
            }
        ],
    }


class WorkflowRegistryShaContractTests(unittest.TestCase):
    """Reject Git's null object identifier wherever an exact commit is required."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_null_default_branch_commit_is_rejected(self) -> None:
        """A null object ID cannot bind registry evidence to protected main."""

        payload = _payload()
        payload["expected_default_branch_sha"] = ZERO_SHA
        payload["observed_default_branch_sha"] = ZERO_SHA

        with self.assertRaisesRegex(self.audit.WorkflowAuditError, "nonzero"):
            self.audit.audit_workflow_registry(payload)

    def test_null_active_pr_head_commit_is_rejected(self) -> None:
        """A null object ID cannot defer a workflow to an independently refetchable PR."""

        payload = _payload()
        payload["reported_total_count"] = 1
        payload["active_pr_workflow_paths"] = [ACTIVE_PR_PATH]
        payload["active_pr_workflow_owners"] = [
            {
                "path": ACTIVE_PR_PATH,
                "pull_request_number": 124,
                "head_sha": ZERO_SHA,
            }
        ]
        payload["registry_pages"][0]["workflows"] = [
            {
                "id": 124,
                "name": "Current diagnostic",
                "path": ACTIVE_PR_PATH,
                "state": "active",
            }
        ]

        with self.assertRaisesRegex(self.audit.WorkflowAuditError, "nonzero"):
            self.audit.audit_workflow_registry(payload)


if __name__ == "__main__":
    unittest.main()
