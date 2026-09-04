"""Fail-closed contracts for repository-owned GitHub Actions workflow paths."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/ci/audit_workflow_registry.py"
DEFAULT_SHA = "0c376acf059be9ddddddfbde1d0189e4f39ef014"


def _load_module():
    """Load the read-only registry auditor without packaging scripts as modules."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload(path: str) -> dict:
    """Build one complete registry payload containing the supplied active record."""

    return {
        "schema_version": 1,
        "expected_default_branch_sha": DEFAULT_SHA,
        "observed_default_branch_sha": DEFAULT_SHA,
        "observed_at": "2026-08-14T00:00:00Z",
        "reported_total_count": 1,
        "protected_workflow_paths": [".github/workflows/ci.yml"],
        "active_pr_workflow_paths": [],
        "registry_pages": [
            {
                "page": 1,
                "status_code": 200,
                "has_next": False,
                "workflows": [
                    {"id": 7001, "name": "candidate", "path": path, "state": "active"}
                ],
            }
        ],
    }


class WorkflowRegistryRepositoryPathTests(unittest.TestCase):
    """Prevent impossible repository paths from becoming disable candidates."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_repository_workflow_candidates_require_direct_yaml_files(self) -> None:
        """Only direct .yml/.yaml files under .github/workflows are repository workflows."""

        for invalid_path in (
            ".github/workflows/no-extension",
            ".github/workflows/not-yaml.json",
            ".github/workflows/nested/child.yml",
            ".github/workflows/nested/child.yaml",
        ):
            with self.subTest(path=invalid_path):
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(_payload(invalid_path))

    def test_both_supported_yaml_extensions_remain_classifiable(self) -> None:
        """GitHub-supported .yml and .yaml workflow files remain valid evidence."""

        for path in (".github/workflows/legacy.yml", ".github/workflows/legacy.yaml"):
            with self.subTest(path=path):
                evidence = self.audit.audit_workflow_registry(_payload(path))
                record = evidence["workflow_records"][0]
                self.assertEqual(record["classification"], "active_orphan_repository_workflow")
                self.assertTrue(record["disable_candidate"])


if __name__ == "__main__":
    unittest.main()
