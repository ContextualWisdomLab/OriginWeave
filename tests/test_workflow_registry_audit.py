"""Regression contracts for read-only GitHub Actions workflow registry auditing."""

from __future__ import annotations

import unittest

from scripts.ci.workflow_registry_audit import (
    WorkflowRegistryAuditError,
    audit_workflow_registry_pages,
)


class WorkflowRegistryAuditTests(unittest.TestCase):
    """Keep orphan-workflow evidence complete, exact, and fail closed."""

    def test_classifies_active_orphan_without_disabling_supported_workflow(self) -> None:
        """Absent active paths are orphan evidence while present supported paths are not."""

        records = audit_workflow_registry_pages(
            pages=[
                {
                    "page_number": 1,
                    "workflows": [
                        {
                            "id": 101,
                            "path": ".github/workflows/ci.yml",
                            "state": "active",
                        },
                        {
                            "id": 202,
                            "path": ".github/workflows/format-pr1-once.yml",
                            "state": "active",
                        },
                    ],
                    "next_page": None,
                }
            ],
            present_workflow_paths={".github/workflows/ci.yml"},
            default_branch_sha="67af7c87589edc2039545af335c95064d9b8391c",
            observed_at="2026-08-12T12:30:00Z",
        )

        self.assertEqual(records[0].classification, "active_present")
        self.assertEqual(records[1].classification, "active_orphan")
        self.assertEqual(records[1].workflow_id, 202)
        self.assertEqual(
            records[1].default_branch_sha,
            "67af7c87589edc2039545af335c95064d9b8391c",
        )
        self.assertEqual(records[1].observed_at, "2026-08-12T12:30:00Z")
        self.assertEqual(records[1].page_number, 1)

    def test_disabled_or_non_repository_workflow_is_not_active_orphan(self) -> None:
        """Disabled records and non-repository paths must not be mutation candidates."""

        records = audit_workflow_registry_pages(
            pages=[
                {
                    "page_number": 1,
                    "workflows": [
                        {
                            "id": 303,
                            "path": ".github/workflows/old.yml",
                            "state": "disabled_manually",
                        },
                        {"id": 404, "path": "dynamic/copilot", "state": "active"},
                    ],
                    "next_page": None,
                }
            ],
            present_workflow_paths=set(),
            default_branch_sha="a" * 40,
            observed_at="2026-08-12T12:30:00Z",
        )

        self.assertEqual(records[0].classification, "disabled")
        self.assertEqual(records[1].classification, "non_repository_path")

    def test_requires_complete_monotonic_pagination_receipts(self) -> None:
        """A truncated or reordered registry inventory must fail closed."""

        with self.assertRaisesRegex(
            WorkflowRegistryAuditError, "incomplete workflow registry pagination"
        ):
            audit_workflow_registry_pages(
                pages=[
                    {
                        "page_number": 1,
                        "workflows": [],
                        "next_page": 2,
                    }
                ],
                present_workflow_paths=set(),
                default_branch_sha="b" * 40,
                observed_at="2026-08-12T12:30:00Z",
            )

        with self.assertRaisesRegex(
            WorkflowRegistryAuditError, "non-contiguous workflow registry pagination"
        ):
            audit_workflow_registry_pages(
                pages=[
                    {"page_number": 1, "workflows": [], "next_page": 3},
                    {"page_number": 3, "workflows": [], "next_page": None},
                ],
                present_workflow_paths=set(),
                default_branch_sha="c" * 40,
                observed_at="2026-08-12T12:30:00Z",
            )

    def test_rejects_ambiguous_path_identity_and_duplicate_workflow_id(self) -> None:
        """Case/encoding ambiguity or reused IDs must not be normalized into safe evidence."""

        with self.assertRaisesRegex(
            WorkflowRegistryAuditError, "ambiguous workflow path"
        ):
            audit_workflow_registry_pages(
                pages=[
                    {
                        "page_number": 1,
                        "workflows": [
                            {
                                "id": 505,
                                "path": ".GITHUB/workflows/ci.yml",
                                "state": "active",
                            }
                        ],
                        "next_page": None,
                    }
                ],
                present_workflow_paths={".github/workflows/ci.yml"},
                default_branch_sha="d" * 40,
                observed_at="2026-08-12T12:30:00Z",
            )

        with self.assertRaisesRegex(
            WorkflowRegistryAuditError, "duplicate workflow id"
        ):
            audit_workflow_registry_pages(
                pages=[
                    {
                        "page_number": 1,
                        "workflows": [
                            {
                                "id": 606,
                                "path": ".github/workflows/ci.yml",
                                "state": "active",
                            },
                            {
                                "id": 606,
                                "path": ".github/workflows/hourly-product-development.yml",
                                "state": "active",
                            },
                        ],
                        "next_page": None,
                    }
                ],
                present_workflow_paths={
                    ".github/workflows/ci.yml",
                    ".github/workflows/hourly-product-development.yml",
                },
                default_branch_sha="e" * 40,
                observed_at="2026-08-12T12:30:00Z",
            )


if __name__ == "__main__":
    unittest.main()
