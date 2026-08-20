"""Regression contracts for read-only orphaned workflow registry evidence."""

from __future__ import annotations

import importlib.util
import io
import json
import pathlib
import types
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"
DEFAULT_SHA = "67af7c87589edc2039545af335c95064d9b8391c"
OBSERVED_AT = "2026-08-12T00:00:00Z"
ACTIVE_PR_HEAD = "b" * 40


def _load_module():
    """Load the repository utility without making scripts a Python package."""

    spec = importlib.util.spec_from_file_location("audit_workflow_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError("workflow registry audit module is not loadable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _workflow(workflow_id: int, path: str, state: str, name: str = "fixture") -> dict:
    """Return one bounded GitHub Actions registry fixture record."""

    return {
        "id": workflow_id,
        "name": name,
        "path": path,
        "state": state,
    }


def _payload(*workflows: dict) -> dict:
    """Return one complete two-page exact-protected-main audit fixture."""

    split = max(1, len(workflows) // 2)
    active_pr_path = ".github/workflows/current-bounded-diagnostic.yml"
    return {
        "schema_version": 1,
        "expected_default_branch_sha": DEFAULT_SHA,
        "observed_default_branch_sha": DEFAULT_SHA,
        "observed_at": OBSERVED_AT,
        "reported_total_count": len(workflows),
        "protected_workflow_paths": [
            ".github/workflows/ci.yml",
            ".github/workflows/hourly-product-development.yml",
        ],
        "active_pr_workflow_paths": [active_pr_path],
        "active_pr_workflow_owners": [
            {
                "path": active_pr_path,
                "pull_request_number": 321,
                "head_sha": ACTIVE_PR_HEAD,
                "observed_head_sha": ACTIVE_PR_HEAD,
            }
        ],
        "registry_pages": [
            {
                "page": 1,
                "status_code": 200,
                "has_next": True,
                "workflows": list(workflows[:split]),
            },
            {
                "page": 2,
                "status_code": 200,
                "has_next": False,
                "workflows": list(workflows[split:]),
            },
        ],
    }


class _GrowingAuditInput:
    """Model an input that grows after a stale metadata check."""

    def __init__(self, maximum_input_bytes: int) -> None:
        self._content = json.dumps(
            {"padding": "x" * maximum_input_bytes}, separators=(",", ":")
        ).encode("utf-8")

    def stat(self):
        """Return deliberately stale metadata claiming one byte."""

        return types.SimpleNamespace(st_size=1)

    def read_text(self, encoding: str):
        """Expose the post-check oversized content to the legacy reader."""

        if encoding != "utf-8":
            raise AssertionError("unexpected encoding")
        return self._content.decode("utf-8")

    def open(self, mode: str):
        """Expose the same content through a bounded binary reader."""

        if mode != "rb":
            raise AssertionError("unexpected mode")
        return io.BytesIO(self._content)


class WorkflowRegistryAuditTests(unittest.TestCase):
    """Keep workflow-lifecycle evidence exhaustive, immutable, and non-mutating."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = _load_module()

    def test_classifies_registry_records_by_exact_path_and_preserves_receipts(self) -> None:
        """Only exact protected-tree/active-PR paths may avoid orphan classification."""

        payload = _payload(
            _workflow(1, ".github/workflows/ci.yml", "active", "CI"),
            _workflow(
                2,
                ".github/workflows/hourly-product-development.yml",
                "active",
                "Hourly Product Development",
            ),
            _workflow(3, ".github/workflows/format-pr1-once.yml", "active", "CI"),
            _workflow(4, ".github/workflows/old-http.yml", "disabled_manually"),
            _workflow(5, "dynamic/dependabot/dependabot-updates", "active"),
            _workflow(
                6,
                ".github/workflows/current-bounded-diagnostic.yml",
                "active",
            ),
        )

        evidence = self.audit.audit_workflow_registry(payload)

        self.assertEqual(evidence["schema_version"], 1)
        self.assertEqual(evidence["default_branch_sha"], DEFAULT_SHA)
        self.assertEqual(evidence["observed_at"], OBSERVED_AT)
        self.assertEqual(evidence["reported_total_count"], 6)
        self.assertFalse(evidence["mutation_performed"])
        self.assertEqual(
            evidence["pagination_receipts"],
            [
                {
                    "page": 1,
                    "status_code": 200,
                    "item_count": 3,
                    "has_next": True,
                },
                {
                    "page": 2,
                    "status_code": 200,
                    "item_count": 3,
                    "has_next": False,
                },
            ],
        )
        classifications = {
            record["workflow_id"]: record["classification"]
            for record in evidence["workflow_records"]
        }
        self.assertEqual(
            classifications,
            {
                1: "present_repository_workflow",
                2: "present_repository_workflow",
                3: "active_orphan_repository_workflow",
                4: "disabled_orphan_repository_workflow",
                5: "github_dynamic_workflow",
                6: "active_pr_owned_workflow",
            },
        )
        orphan = next(
            record
            for record in evidence["workflow_records"]
            if record["workflow_id"] == 3
        )
        self.assertEqual(orphan["path"], ".github/workflows/format-pr1-once.yml")
        self.assertEqual(orphan["state"], "active")
        self.assertEqual(orphan["default_branch_sha"], DEFAULT_SHA)
        self.assertEqual(orphan["observed_at"], OBSERVED_AT)

    def test_name_collision_never_protects_an_absent_workflow_path(self) -> None:
        """A historical workflow named CI is not current CI without exact path ownership."""

        evidence = self.audit.audit_workflow_registry(
            _payload(_workflow(77, ".github/workflows/legacy-ci.yml", "active", "CI"))
        )
        self.assertEqual(
            evidence["workflow_records"][0]["classification"],
            "active_orphan_repository_workflow",
        )

    def test_incomplete_or_noncontiguous_pagination_fails_closed(self) -> None:
        """A truncated first page must never be treated as a complete registry inventory."""

        for mutate in (
            lambda payload: payload["registry_pages"].pop(),
            lambda payload: payload["registry_pages"][1].update(page=3),
            lambda payload: payload["registry_pages"][0].update(has_next=False),
        ):
            with self.subTest(mutate=mutate):
                payload = _payload(_workflow(1, ".github/workflows/ci.yml", "active"))
                mutate(payload)
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(payload)

    def test_reported_total_count_must_match_the_complete_unique_inventory(self) -> None:
        """A falsely terminated export cannot hide records behind a complete-looking page."""

        for reported_total_count in (-1, True, 0, 2):
            with self.subTest(reported_total_count=reported_total_count):
                payload = _payload(_workflow(1, ".github/workflows/ci.yml", "active"))
                payload["reported_total_count"] = reported_total_count
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(payload)

        payload = _payload(
            _workflow(1, ".github/workflows/ci.yml", "active"),
            _workflow(2, ".github/workflows/orphan.yml", "active"),
        )
        payload["registry_pages"][1]["workflows"].clear()
        with self.assertRaises(self.audit.WorkflowAuditError):
            self.audit.audit_workflow_registry(payload)

    def test_input_size_bound_applies_to_bytes_read_not_stale_metadata(self) -> None:
        """A growing or replaced input cannot bypass the four-mebibyte read bound."""

        source = _GrowingAuditInput(self.audit._MAX_INPUT_BYTES)
        with self.assertRaises(self.audit.WorkflowAuditError):
            self.audit._read_payload(source)

    def test_permission_and_transient_http_failures_fail_closed(self) -> None:
        """403/404/5xx exports are evidence gaps, not empty successful pages."""

        for status_code in (403, 404, 429, 500, 503):
            with self.subTest(status_code=status_code):
                payload = _payload(_workflow(1, ".github/workflows/ci.yml", "active"))
                payload["registry_pages"][0]["status_code"] = status_code
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(payload)

    def test_default_branch_movement_invalidates_the_inventory(self) -> None:
        """Evidence bound to a moved protected branch must be recollected before use."""

        payload = _payload(_workflow(1, ".github/workflows/ci.yml", "active"))
        payload["observed_default_branch_sha"] = "a" * 40
        with self.assertRaises(self.audit.WorkflowAuditError):
            self.audit.audit_workflow_registry(payload)

    def test_impossible_observation_timestamps_fail_closed(self) -> None:
        """Syntactically shaped but impossible UTC times are not valid audit evidence."""

        for observed_at in (
            "2026-02-30T00:00:00Z",
            "2026-08-12T24:00:00Z",
            "2026-08-12T23:60:00Z",
            "2026-08-12T23:59:60Z",
        ):
            with self.subTest(observed_at=observed_at):
                payload = _payload(_workflow(1, ".github/workflows/ci.yml", "active"))
                payload["observed_at"] = observed_at
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(payload)

    def test_deleted_and_disabled_fork_states_are_inactive_non_candidates(self) -> None:
        """Every documented inactive REST state remains valid but never actionable."""

        for state in ("deleted", "disabled_fork"):
            with self.subTest(state=state):
                evidence = self.audit.audit_workflow_registry(
                    _payload(_workflow(20, ".github/workflows/legacy.yml", state))
                )
                record = evidence["workflow_records"][0]
                self.assertEqual(record["classification"], "disabled_orphan_repository_workflow")
                self.assertFalse(record["disable_candidate"])

    def test_malformed_case_encoded_and_traversal_paths_fail_closed(self) -> None:
        """Ambiguous workflow paths must never become disable recommendations."""

        for path in (
            ".GITHUB/workflows/ci.yml",
            ".github/WORKFLOWS/ci.yml",
            ".github/workflows/%2e%2e/ci.yml",
            ".github/workflows/../ci.yml",
            ".github\\workflows\\ci.yml",
            ".github/workflows/ci.yml\u0000",
        ):
            with self.subTest(path=path):
                payload = _payload(_workflow(1, path, "active"))
                with self.assertRaises(self.audit.WorkflowAuditError):
                    self.audit.audit_workflow_registry(payload)

    def test_reused_or_duplicate_workflow_identifiers_fail_closed(self) -> None:
        """A workflow ID reused for another path cannot be classified safely."""

        payload = _payload(
            _workflow(9, ".github/workflows/ci.yml", "active"),
            _workflow(9, ".github/workflows/legacy.yml", "active"),
        )
        with self.assertRaises(self.audit.WorkflowAuditError):
            self.audit.audit_workflow_registry(payload)

    def test_distinct_workflow_ids_cannot_reuse_the_same_registry_path(self) -> None:
        """Duplicate paths cannot create ambiguous disable-candidate evidence."""

        payload = _payload(
            _workflow(31, ".github/workflows/orphan.yml", "active"),
            _workflow(32, ".github/workflows/orphan.yml", "active"),
        )
        with self.assertRaises(self.audit.WorkflowAuditError):
            self.audit.audit_workflow_registry(payload)

    def test_active_pr_owned_diagnostic_is_not_reported_as_an_orphan(self) -> None:
        """A bounded workflow owned by an active PR remains deferred, never disabled."""

        evidence = self.audit.audit_workflow_registry(
            _payload(
                _workflow(
                    12,
                    ".github/workflows/current-bounded-diagnostic.yml",
                    "active",
                )
            )
        )
        record = evidence["workflow_records"][0]
        self.assertEqual(record["classification"], "active_pr_owned_workflow")
        self.assertFalse(record["disable_candidate"])
        self.assertEqual(
            record["active_pr_owner"],
            {"pull_request_number": 321, "head_sha": ACTIVE_PR_HEAD},
        )

    def test_only_reviewed_active_orphans_are_disable_candidates(self) -> None:
        """Dynamic, disabled, present, and active-PR records remain non-candidates."""

        evidence = self.audit.audit_workflow_registry(
            _payload(
                _workflow(1, ".github/workflows/ci.yml", "active"),
                _workflow(2, ".github/workflows/orphan.yml", "active"),
                _workflow(3, ".github/workflows/disabled.yml", "disabled_manually"),
                _workflow(4, "dynamic/codeql/code-scanning", "active"),
                _workflow(
                    5,
                    ".github/workflows/current-bounded-diagnostic.yml",
                    "active",
                ),
            )
        )
        candidates = [
            record["workflow_id"]
            for record in evidence["workflow_records"]
            if record["disable_candidate"]
        ]
        self.assertEqual(candidates, [2])


if __name__ == "__main__":
    unittest.main()