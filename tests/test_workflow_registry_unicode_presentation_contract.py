"""Reject invisible Unicode controls from workflow audit presentation evidence."""

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


def _payload(*, name: str = "candidate", path: str = ".github/workflows/orphan.yml") -> dict:
    """Build one complete registry payload containing one active orphan record."""

    return {
        "schema_version": 1,
        "expected_default_branch_sha": DEFAULT_SHA,
        "observed_default_branch_sha": DEFAULT_SHA,
        "observed_at": "2026-08-15T02:00:00Z",
        "reported_total_count": 1,
        "protected_workflow_paths": [".github/workflows/ci.yml"],
        "active_pr_workflow_paths": [],
        "registry_pages": [
            {
                "page": 1,
                "status_code": 200,
                "has_next": False,
                "workflows": [
                    {"id": 9001, "name": name, "path": path, "state": "active"}
                ],
            }
        ],
    }


class WorkflowRegistryUnicodePresentationTests(unittest.TestCase):
    """Keep operator-facing workflow evidence free of invisible format controls."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_bidi_and_zero_width_controls_fail_closed_in_names_and_paths(self) -> None:
        """Invisible Unicode formatting must not survive into canonical audit evidence."""

        for field, value in (
            ("name", "legacy\u202eworkflow"),
            ("name", "legacy\u200bworkflow"),
            ("path", ".github/workflows/legacy\u202e.yml"),
            ("path", ".github/workflows/legacy\u200b.yml"),
        ):
            with self.subTest(field=field, value=value):
                payload = _payload(**{field: value})
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(payload)

    def test_printable_unicode_workflow_name_remains_valid(self) -> None:
        """Ordinary printable Unicode labels remain usable as non-authoritative display text."""

        evidence = self.audit.audit_workflow_registry(_payload(name="릴리즈 점검 🚦"))
        self.assertEqual(evidence["workflow_records"][0]["name"], "릴리즈 점검 🚦")


if __name__ == "__main__":
    unittest.main()
