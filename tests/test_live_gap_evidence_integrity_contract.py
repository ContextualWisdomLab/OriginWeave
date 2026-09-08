"""Fail-closed contracts for the current product-gap evidence procedure."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs" / "product-technical-gap-baseline.md"
CHANGELOG = ROOT / "CHANGELOG.md"
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
        cls.evidence = EVIDENCE_SCRIPT.read_text(encoding="utf-8")
        cls.latest = bounded(
            cls.baseline,
            "### Latest verified cut: 2026-09-08",
            "### Historical verified cut: 2026-09-07",
        )

    def test_latest_inventory_and_changelog_use_the_september_8_cut(self) -> None:
        marker = (
            "126 open pull requests: 12 Ready/non-draft and "
            "114 Draft; 14 open non-PR issues"
        )
        self.assertIn(marker, " ".join(self.latest.split()))
        inventory = [
            line
            for line in self.changelog.splitlines()
            if line.startswith("- Current delivery inventory:")
        ]
        self.assertEqual(1, len(inventory))
        self.assertIn("126 open pull requests (12 ready, 114 draft)", inventory[0])
        self.assertIn("14 open non-PR issues", inventory[0])
        self.assertIn("Observed 2026-09-08", inventory[0])

    def test_presentation_snapshot_uses_full_exact_sha(self) -> None:
        full_sha = "0c077445d73640a6299ea4d379faa4b0ab0226c2"
        self.assertIn(full_sha, self.latest)
        self.assertNotIn("to exact head\n`0c077445`", self.latest)

    def test_baseline_names_the_executable_current_evidence_collector(self) -> None:
        current_evidence = self.baseline.split("## Evidence commands", 1)[1]
        self.assertIn("scripts/ci/collect_live_merge_evidence.sh", current_evidence)

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


if __name__ == "__main__":
    unittest.main()
