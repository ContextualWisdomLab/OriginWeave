"""Bind active-PR workflow deferrals to exact auditable pull-request identities."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"
DEFAULT_SHA = "0841d2ab3d8b5e60a03c0a8e818cf438e2716829"
ACTIVE_PR_HEAD = "b" * 40
ACTIVE_PATH = ".github/workflows/current-bounded-diagnostic.yml"


def _load_module():
    """Load the read-only registry auditor without packaging scripts as modules."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload() -> dict:
    """Build evidence in which one workflow is deferred to an exact active PR owner."""

    return {
        "schema_version": 1,
        "expected_default_branch_sha": DEFAULT_SHA,
        "observed_default_branch_sha": DEFAULT_SHA,
        "observed_at": "2026-08-17T07:30:00Z",
        "reported_total_count": 1,
        "protected_workflow_paths": [".github/workflows/ci.yml"],
        "active_pr_workflow_paths": [ACTIVE_PATH],
        "active_pr_workflow_owners": [
            {
                "path": ACTIVE_PATH,
                "pull_request_number": 321,
                "head_sha": ACTIVE_PR_HEAD,
            }
        ],
        "registry_pages": [
            {
                "page": 1,
                "status_code": 200,
                "has_next": False,
                "workflows": [
                    {
                        "id": 17,
                        "name": "Current bounded diagnostic",
                        "path": ACTIVE_PATH,
                        "state": "active",
                    }
                ],
            }
        ],
    }


class WorkflowRegistryActivePrOwnerContractTests(unittest.TestCase):
    """Prevent an unbound or stale-looking path assertion from hiding an orphan."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_active_pr_deferral_retains_exact_owner_identity(self) -> None:
        """A deferred workflow records the PR number and exact contributor head."""

        evidence = self.audit.audit_workflow_registry(_payload())
        record = evidence["workflow_records"][0]
        self.assertEqual(record["classification"], "active_pr_owned_workflow")
        self.assertFalse(record["disable_candidate"])
        self.assertEqual(
            record["active_pr_owner"],
            {
                "pull_request_number": 321,
                "head_sha": ACTIVE_PR_HEAD,
            },
        )

    def test_disabled_active_pr_workflow_is_reported_as_operational_drift(self) -> None:
        """PR ownership must not hide that its registry identity is disabled."""

        payload = _payload()
        payload["registry_pages"][0]["workflows"][0]["state"] = "disabled_manually"
        evidence = self.audit.audit_workflow_registry(payload)
        record = evidence["workflow_records"][0]
        self.assertEqual(
            record["classification"], "disabled_active_pr_owned_workflow"
        )
        self.assertFalse(record["disable_candidate"])
        self.assertEqual(
            record["active_pr_owner"],
            {
                "pull_request_number": 321,
                "head_sha": ACTIVE_PR_HEAD,
            },
        )

    def test_nonempty_active_pr_paths_require_exact_owner_evidence(self) -> None:
        """A path string alone cannot suppress an orphan disable candidate."""

        payload = _payload()
        payload.pop("active_pr_workflow_owners")
        with self.assertRaisesRegex(
            self.audit.WorkflowAuditError,
            "active PR workflow ownership must be bound to exact PR heads",
        ):
            self.audit.audit_workflow_registry(payload)

    def test_owner_paths_must_exactly_match_deferred_paths(self) -> None:
        """Missing, extra, or mismatched owner paths fail closed."""

        for owners in (
            [],
            [
                {
                    "path": ".github/workflows/other.yml",
                    "pull_request_number": 321,
                    "head_sha": ACTIVE_PR_HEAD,
                }
            ],
        ):
            with self.subTest(owners=owners):
                payload = _payload()
                payload["active_pr_workflow_owners"] = owners
                with self.assertRaisesRegex(
                    self.audit.WorkflowAuditError,
                    "active PR workflow ownership must be bound to exact PR heads",
                ):
                    self.audit.audit_workflow_registry(payload)

    def test_owner_identity_rejects_ambiguous_pr_numbers_and_heads(self) -> None:
        """Only positive integer PR numbers and exact lowercase commit SHAs are accepted."""

        invalid_identities = (
            (True, ACTIVE_PR_HEAD),
            (0, ACTIVE_PR_HEAD),
            (-1, ACTIVE_PR_HEAD),
            (321, "B" * 40),
            (321, "b" * 39),
        )
        for pull_request_number, head_sha in invalid_identities:
            with self.subTest(
                pull_request_number=pull_request_number, head_sha=head_sha
            ):
                payload = _payload()
                payload["active_pr_workflow_owners"][0][
                    "pull_request_number"
                ] = pull_request_number
                payload["active_pr_workflow_owners"][0]["head_sha"] = head_sha
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(payload)

    def test_owner_head_movement_during_collection_fails_closed(self) -> None:
        """A moved contributor head cannot keep suppressing an orphan candidate."""

        payload = _payload()
        payload["active_pr_workflow_owners"][0]["observed_head_sha"] = "c" * 40
        with self.assertRaisesRegex(
            self.audit.WorkflowAuditError,
            "active PR workflow head moved during collection",
        ):
            self.audit.audit_workflow_registry(payload)


if __name__ == "__main__":
    unittest.main()
