"""Fail closed when a collected workflow-registry page is not an HTTP 200 response."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"
DEFAULT_SHA = "0c376acf059be9ddddddfbde1d0189e4f39ef014"


def _load_module():
    """Load the read-only registry auditor without packaging scripts as modules."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload(status_code: int) -> dict:
    """Build one complete one-page registry payload with a selected HTTP result."""

    return {
        "schema_version": 1,
        "expected_default_branch_sha": DEFAULT_SHA,
        "observed_default_branch_sha": DEFAULT_SHA,
        "observed_at": "2026-08-16T09:40:00Z",
        "reported_total_count": 1,
        "protected_workflow_paths": [".github/workflows/ci.yml"],
        "active_pr_workflow_paths": [],
        "registry_pages": [
            {
                "page": 1,
                "status_code": status_code,
                "has_next": False,
                "workflows": [
                    {
                        "id": 9001,
                        "name": "orphan candidate",
                        "path": ".github/workflows/orphan.yml",
                        "state": "active",
                    }
                ],
            }
        ],
    }


class WorkflowRegistryHttpStatusContractTests(unittest.TestCase):
    """Treat permission, absence, throttling, and server failures as non-evidence."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_permission_absence_rate_limit_and_server_errors_fail_closed(self) -> None:
        """A non-200 page must never be interpreted as a complete registry snapshot."""

        for status_code in (403, 404, 429, 500, 502, 503, 504):
            with self.subTest(status_code=status_code):
                with self.assertRaisesRegex(
                    self.audit.WorkflowAuditError,
                    "registry page 1 did not return HTTP 200",
                ):
                    self.audit.audit_workflow_registry(_payload(status_code))

    def test_http_failures_expose_safe_retryability_without_becoming_success(self) -> None:
        """Only throttling/server failures should tell a collector that retry is safe."""

        for status_code, retryable in (
            (403, False),
            (404, False),
            (429, True),
            (500, True),
            (502, True),
            (503, True),
            (504, True),
        ):
            with self.subTest(status_code=status_code):
                with self.assertRaises(self.audit.WorkflowAuditHttpStatusError) as raised:
                    self.audit.audit_workflow_registry(_payload(status_code))
                error = raised.exception
                self.assertEqual(error.page_number, 1)
                self.assertEqual(error.status_code, status_code)
                self.assertEqual(error.retryable, retryable)
                self.assertEqual(
                    str(error), "registry page 1 did not return HTTP 200"
                )

    def test_boolean_status_is_not_accepted_as_integer_200(self) -> None:
        """Python's bool-as-int relationship must not create synthetic HTTP success."""

        payload = _payload(200)
        payload["registry_pages"][0]["status_code"] = True
        with self.assertRaisesRegex(
            self.audit.WorkflowAuditError,
            "registry page 1 did not return HTTP 200",
        ):
            self.audit.audit_workflow_registry(payload)

    def test_exact_http_200_still_classifies_the_active_orphan_candidate(self) -> None:
        """The failure regression must preserve the reviewed successful evidence path."""

        evidence = self.audit.audit_workflow_registry(_payload(200))
        record = evidence["workflow_records"][0]
        self.assertEqual(record["classification"], "active_orphan_repository_workflow")
        self.assertTrue(record["disable_candidate"])
        self.assertFalse(evidence["mutation_performed"])


if __name__ == "__main__":
    unittest.main()
