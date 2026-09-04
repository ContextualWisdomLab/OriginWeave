"""Contract for surfacing disabled workflow identities that still exist on protected main."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"
DEFAULT_SHA = "67af7c87589edc2039545af335c95064d9b8391c"


def _load_module():
    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload(state: str) -> dict:
    return {
        "schema_version": 1,
        "expected_default_branch_sha": DEFAULT_SHA,
        "observed_default_branch_sha": DEFAULT_SHA,
        "observed_at": "2026-08-12T00:00:00Z",
        "reported_total_count": 1,
        "protected_workflow_paths": [".github/workflows/ci.yml"],
        "active_pr_workflow_paths": [],
        "registry_pages": [
            {
                "page": 1,
                "status_code": 200,
                "has_next": False,
                "workflows": [
                    {
                        "id": 1,
                        "name": "CI",
                        "path": ".github/workflows/ci.yml",
                        "state": state,
                    }
                ],
            }
        ],
    }


class DisabledPresentWorkflowContractTests(unittest.TestCase):
    """Require source presence and registry operational state to remain distinct evidence."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_disabled_protected_workflow_is_reported_as_operational_drift(self) -> None:
        """A current protected source path must not hide a disabled registry identity."""

        for state in (
            "deleted",
            "disabled_fork",
            "disabled_inactivity",
            "disabled_manually",
        ):
            with self.subTest(state=state):
                evidence = self.audit.audit_workflow_registry(_payload(state))
                record = evidence["workflow_records"][0]
                self.assertEqual(
                    record["classification"], "disabled_present_repository_workflow"
                )
                self.assertFalse(record["disable_candidate"])


if __name__ == "__main__":
    unittest.main()
