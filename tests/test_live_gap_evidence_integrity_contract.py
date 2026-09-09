"""Fail-closed contracts for the current product-gap evidence procedure."""

from pathlib import Path
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs" / "product-technical-gap-baseline.md"
CHANGELOG = ROOT / "CHANGELOG.md"
AGENTS = ROOT / "AGENTS.md"
EVIDENCE_SCRIPT = ROOT / "scripts" / "ci" / "collect_live_merge_evidence.sh"


def bounded(text: str, start: str, end: str) -> str:
    """Return one current section without accepting a historical substitute."""
    if start not in text:
        raise AssertionError(f"missing start marker: {start}")
    remainder = text.split(start, 1)[1]
    if end not in remainder:
        raise AssertionError(f"missing end marker after {start}: {end}")
    return remainder.split(end, 1)[0]


class LiveGapEvidenceIntegrityContractTests(unittest.TestCase):
    """Keep merge evidence bound to the exact current PR state."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.baseline = BASELINE.read_text(encoding="utf-8")
        cls.changelog = CHANGELOG.read_text(encoding="utf-8")
        cls.agents = AGENTS.read_text(encoding="utf-8")
        cls.evidence = EVIDENCE_SCRIPT.read_text(encoding="utf-8")
        cls.latest = bounded(
            cls.baseline,
            "### Latest verified cut: 2026-09-09",
            "### Previous verified cut: 2026-09-08",
        )

    def test_latest_inventory_and_changelog_use_the_september_9_cut(self) -> None:
        marker = (
            "131 open pull requests: 14 Ready/non-draft and "
            "117 Draft; 14 open non-PR issues"
        )
        self.assertIn(marker, " ".join(self.latest.split()))
        inventory = [
            line
            for line in self.changelog.splitlines()
            if line.startswith("- Current delivery inventory:")
        ]
        self.assertEqual(1, len(inventory))
        self.assertIn("131 open pull requests (14 ready, 117 draft)", inventory[0])
        self.assertIn("14 open non-PR issues", inventory[0])
        self.assertIn("Observed 2026-09-09", inventory[0])

    def test_presentation_snapshot_uses_full_exact_sha(self) -> None:
        full_sha = "6855e2578ae94279cc9ab4a14527b016e8c049ee"
        self.assertIn(full_sha, self.latest)
        self.assertNotIn("to exact head\n`6855e257`", self.latest)

    def test_active_presentation_harness_is_not_described_as_unimplemented(self) -> None:
        normalized = " ".join(self.latest.split())
        self.assertIn("active controlled evidence harness is implemented", normalized)
        self.assertIn("failed 0/3 at session creation before navigation", normalized)
        self.assertIn("product Browser Session adapter", normalized)

    def test_documented_current_evidence_collector_is_executable(self) -> None:
        current_evidence = self.baseline.split("## Evidence commands", 1)[1]
        self.assertIn("scripts/ci/collect_live_merge_evidence.sh", current_evidence)
        self.assertIn("scripts/ci/collect_live_merge_evidence.sh", self.agents)
        self.assertNotEqual(0, EVIDENCE_SCRIPT.stat().st_mode & 0o111)

    def test_evidence_collector_is_valid_bash(self) -> None:
        result = subprocess.run(
            ["bash", "-n", str(EVIDENCE_SCRIPT)],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(0, result.returncode, result.stderr)

    def test_current_change_requests_block_the_approval_verdict(self) -> None:
        self.assertIn("as $current_change_requests", self.evidence)
        self.assertIn("blocking_change_requests: $current_change_requests", self.evidence)
        self.assertIn("($current_change_requests | length) == 0", self.evidence)

    def test_every_unresolved_thread_remains_blocking_when_outdated(self) -> None:
        self.assertIn("select(.isResolved == false)", self.evidence)
        self.assertNotIn(
            "select(.isResolved == false and .isOutdated == false)",
            self.evidence,
        )

    def test_workflow_evidence_is_bound_to_pr_head_and_base(self) -> None:
        for marker in (
            ".number == ($pr[0].number)",
            ".head.sha == $head",
            ".base.sha == $base",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.evidence)
        self.assertIn("workflow_runs_without_exact_pr_base_provenance", self.evidence)

    def test_rules_are_evaluated_for_each_pull_requests_actual_base(self) -> None:
        for marker in (
            "BASE_REF=$(jq -r '.base.ref' \"$PR_JSON\")",
            'BASE_REF_ENCODED=$(jq -rn --arg value "$BASE_REF" \'$value | @uri\')',
            '"repos/$REPOSITORY/rules/branches/$BASE_REF_ENCODED?per_page=100"',
            '--slurpfile rules "$EVIDENCE_DIR/pr-${PR}-branch-rules.json"',
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.evidence)
        self.assertNotIn(
            '--slurpfile rules "$EVIDENCE_DIR/main-branch-rules.json"',
            self.evidence,
        )

    def test_verdict_materializes_only_after_inventory_and_final_state_stabilize(self) -> None:
        for marker in (
            "INVENTORY_DRAFT=$(jq -r --argjson pr \"$PR\"",
            "PR_STATE=$(jq -r '.state' \"$PR_JSON\")",
            "PR_DRAFT=$(jq -r '.draft' \"$PR_JSON\")",
            "RECHECKED_STATE=$(jq -r '.state' \"$RECHECKED_PR_JSON\")",
            "RECHECKED_DRAFT=$(jq -r '.draft' \"$RECHECKED_PR_JSON\")",
            '"$PR_STATE" == "open"',
            '"$RECHECKED_STATE" == "$PR_STATE"',
            '"$RECHECKED_DRAFT" == "$PR_DRAFT"',
            '"$PR_DRAFT" == "$INVENTORY_DRAFT"',
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.evidence)

    def test_whole_generation_closes_against_fresh_pr_and_issue_inventories(self) -> None:
        for marker in (
            'open-pr-pages-rechecked.json',
            'open-prs-rechecked.json',
            'open-issue-pages-rechecked.json',
            'open-issues-rechecked.json',
            'INITIAL_PR_INVENTORY_PROJECTION=',
            'FINAL_PR_INVENTORY_PROJECTION=',
            'INITIAL_ISSUE_INVENTORY_PROJECTION=',
            'FINAL_ISSUE_INVENTORY_PROJECTION=',
            'Live inventory changed during evidence collection.',
            'rm -f "$EVIDENCE_DIR"/pr-*-merge-verdict.json',
            'evidence-generation.json',
            'complete: true',
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.evidence)

    def test_explicit_evidence_directory_must_start_empty(self) -> None:
        for marker in (
            'if [[ $# -gt 0 ]]; then',
            'find "$EVIDENCE_DIR" -mindepth 1 -print -quit',
            'Evidence directory must be empty:',
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.evidence)


if __name__ == "__main__":
    unittest.main()
