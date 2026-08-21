"""Preserve bounded Retry-After guidance without turning failed collection into evidence."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import pathlib
import tempfile
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

    def _run_cli(self, payload: dict) -> tuple[int, str]:
        """Run the operator CLI against one bounded local evidence document."""

        with tempfile.TemporaryDirectory(prefix="originweave-workflow-audit-") as directory:
            input_path = pathlib.Path(directory) / "registry.json"
            input_path.write_text(json.dumps(payload), encoding="utf-8")
            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                exit_code = self.audit.main([str(input_path)])
        return exit_code, stderr.getvalue()

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
        """Request timeout, rate limiting, and transient server failures expose bounded wait hints."""

        for status_code in (408, 429, 500, 502, 503, 504):
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

    def test_cli_preserves_retryable_recollection_guidance(self) -> None:
        """Operator stderr must retain the typed bounded wait decision on failure."""

        exit_code, diagnostic = self._run_cli(_payload(403, 30))

        self.assertEqual(exit_code, 1)
        self.assertIn("registry page 1 did not return HTTP 200", diagnostic)
        self.assertIn("retryable=true", diagnostic)
        self.assertIn("retry_after_seconds=30", diagnostic)

    def test_cli_preserves_nonretryable_decision_without_promoting_hint(self) -> None:
        """A retained delay on a non-reviewed status must stay explicitly non-retryable."""

        exit_code, diagnostic = self._run_cli(_payload(404, 30))

        self.assertEqual(exit_code, 1)
        self.assertIn("registry page 1 did not return HTTP 200", diagnostic)
        self.assertIn("retryable=false", diagnostic)
        self.assertIn("retry_after_seconds=30", diagnostic)

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
