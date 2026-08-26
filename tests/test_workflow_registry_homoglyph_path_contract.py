"""Reject Unicode homoglyph path confusion from workflow audit evidence.

These regression tests cover the Strix-reported finding ``vuln-0001``: printable
Unicode confusables such as FULLWIDTH SOLIDUS must never enter workflow path
evidence where they could pass validation yet fail exact-match classification.
"""

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


class WorkflowRegistryHomoglyphPathContractTests(unittest.TestCase):
    """Keep every audited workflow path inside the canonical ASCII alphabet."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def _record(self, path: str, workflow_id: int = 4101) -> dict:
        """Return one active registry record carrying the supplied path."""

        return {
            "id": workflow_id,
            "name": "twin",
            "path": path,
            "state": "active",
        }

    def _payload(self, *records: dict) -> dict:
        """Build one complete payload around the supplied registry records."""

        return {
            "schema_version": 1,
            "expected_default_branch_sha": DEFAULT_SHA,
            "observed_default_branch_sha": DEFAULT_SHA,
            "observed_at": "2026-08-26T03:00:00Z",
            "reported_total_count": len(records),
            "protected_workflow_paths": [],
            "active_pr_workflow_paths": [],
            "registry_pages": [
                {
                    "page": 1,
                    "status_code": 200,
                    "has_next": False,
                    "workflows": list(records),
                }
            ],
        }

    def test_homoglyph_path_separators_fail_closed(self) -> None:
        """Printable slash look-alikes must be rejected before classification."""

        for separator in ("\uff0f", "\u2044", "\u2215", "\u29f8"):
            with self.subTest(separator=hex(ord(separator))):
                twin = f"github{separator}workflows{separator}real.yml"
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(
                        self._payload(self._record(twin))
                    )

    def test_mixed_ascii_and_homoglyph_separators_fail_closed(self) -> None:
        """Partially homoglyphic paths cannot bypass the canonical alphabet gate."""

        for path in (
            ".github/workflows\uff0freal.yml",
            ".github\uff0fworkflows/real.yml",
            ".github/workflows/real.yml\uff0e",
        ):
            with self.subTest(path=path):
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(self._payload(self._record(path)))

    def test_fullwidth_and_accented_path_letters_fail_closed(self) -> None:
        """Confusable letter forms outside the canonical alphabet are rejected."""

        for path in (
            ".github/workflows/r\u0131al.yml",
            ".github/workflows/\uff52eal.yml",
        ):
            with self.subTest(path=path):
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(self._payload(self._record(path)))

    def test_canonical_ascii_paths_remain_valid(self) -> None:
        """The strict alphabet keeps every legitimate repository workflow usable."""

        evidence = self.audit.audit_workflow_registry(
            self._payload(
                self._record(".github/workflows/ci.yml", workflow_id=4101),
                self._record("dynamic/external-workflow.yml", workflow_id=4102),
            )
        )
        classifications = [
            record["classification"] for record in evidence["workflow_records"]
        ]
        self.assertEqual(classifications[0], "active_orphan_repository_workflow")
        self.assertEqual(classifications[1], "github_dynamic_workflow")


if __name__ == "__main__":
    unittest.main()
