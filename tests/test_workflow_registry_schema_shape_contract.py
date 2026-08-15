"""Reject forward-incompatible fields in versioned workflow registry evidence."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"
DEFAULT_SHA = "67af7c87589edc2039545af335c95064d9b8391c"


def _load_module():
    """Load the read-only workflow audit utility without packaging scripts."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload() -> dict:
    """Return one minimal complete schema-v1 registry evidence document."""

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
                        "state": "active",
                    }
                ],
            }
        ],
    }


class WorkflowRegistrySchemaShapeContractTests(unittest.TestCase):
    """Keep schema-v1 evidence closed against silent producer shape drift."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_unknown_members_fail_closed_at_each_versioned_object_boundary(self) -> None:
        """Unknown document, page, and workflow members cannot be silently ignored."""

        mutations = (
            lambda payload: payload.update(unexpected_document_field="shadow"),
            lambda payload: payload["registry_pages"][0].update(
                unexpected_page_field="shadow"
            ),
            lambda payload: payload["registry_pages"][0]["workflows"][0].update(
                unexpected_workflow_field="shadow"
            ),
        )
        for mutate in mutations:
            with self.subTest(mutate=mutate):
                payload = _payload()
                mutate(payload)
                with self.assertRaisesRegex(
                    self.audit.WorkflowAuditError, "unsupported field"
                ):
                    self.audit.audit_workflow_registry(payload)


if __name__ == "__main__":
    unittest.main()
