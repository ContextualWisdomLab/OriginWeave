"""Preserve bounded Retry-After guidance without turning failed collection into evidence."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"
DEFAULT_SHA = "0841d2ab3d8b5e60a03c0a8e818cf438e2716829"


def _load_module():
    """Load the read-only registry auditor without packaging scripts as modules."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload(status_code: int, retry_after_seconds: object = None) -> dict:
    """Build one collection page with optional bounded retry guidance."""

    page = {
        "page": 1,
        "status_code": status_code,
        "has_next": False,
        "workflows": [],
    }
    if retry_after_seconds is not None:
        page["retry_after_seconds"] = retry_after_seconds
    return {
        "schema_version": 1,
        "expected_default_branch_sha": DEFAULT_SHA,
        "observed_default_branch_sha": DEFAULT_SHA,
        "observed_at": "2026-08-16T21:40:00Z",
        "reported_total_count": 0,
        "protected_workflow_paths": [".github/workflows/ci.yml"],
        "active_pr_workflow_paths": [],
        "registry_pages": [page],
    }


class WorkflowRegistryRetryAfterContractTests(unittest.TestCase):
    """Keep recollection guidance typed, bounded, and fail-closed."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_rate_limited_403_becomes_retryable_only_with_bounded_retry_after(self) -> None:
        """A collected 403 may be retried only when the collector retained Retry-After."""

        with self.assertRaises(self.audit.WorkflowAuditHttpStatusError) as raised:
            self.audit.audit_workflow_registry(_payload(403, 30))
        error = raised.exception
        self.assertTrue(error.retryable)
        self.assertEqual(error.retry_after_seconds, 30)
        self.assertEqual(str(error), "registry page 1 did not return HTTP 200")

        with self.assertRaises(self.audit.WorkflowAuditHttpStatusError) as raised_without_hint:
            self.audit.audit_workflow_registry(_payload(403))
        self.assertFalse(raised_without_hint.exception.retryable)
        self.assertIsNone(raised_without_hint.exception.retry_after_seconds)

    def test_retry_after_is_preserved_for_reviewed_transient_statuses(self) -> None:
        """429 and transient server failures expose the same bounded wait hint."""

        for status_code in (429, 500, 502, 503, 504):
            with self.subTest(status_code=status_code):
                with self.assertRaises(self.audit.WorkflowAuditHttpStatusError) as raised:
                    self.audit.audit_workflow_registry(_payload(status_code, 120))
                self.assertTrue(raised.exception.retryable)
                self.assertEqual(raised.exception.retry_after_seconds, 120)

    def test_retry_after_does_not_make_nontransient_statuses_retryable(self) -> None:
        """Missing resources and unsupported protocol responses stay non-retryable."""

        for status_code in (404, 501, 505):
            with self.subTest(status_code=status_code):
                with self.assertRaises(self.audit.WorkflowAuditHttpStatusError) as raised:
                    self.audit.audit_workflow_registry(_payload(status_code, 30))
                self.assertFalse(raised.exception.retryable)
                self.assertEqual(raised.exception.retry_after_seconds, 30)

    def test_retry_after_rejects_ambiguous_or_unbounded_values(self) -> None:
        """Retry guidance is a bounded integer, not a bool, negative, or huge wait."""

        for value in (True, False, -1, 3601, "30", 1.5):
            with self.subTest(value=value):
                with self.assertRaisesRegex(
                    self.audit.WorkflowAuditError,
                    "retry_after_seconds must be an integer from 0 through 3600",
                ):
                    self.audit.audit_workflow_registry(_payload(429, value))

    def test_success_page_rejects_retry_after_guidance(self) -> None:
        """Successful registry evidence must not silently retain retry-only metadata."""

        with self.assertRaisesRegex(
            self.audit.WorkflowAuditError,
            "retry_after_seconds is only valid for a failed registry page",
        ):
            self.audit.audit_workflow_registry(_payload(200, 30))


if __name__ == "__main__":
    unittest.main()
