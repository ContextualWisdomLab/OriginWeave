"""Regression contracts for the current dated product-gap inventory snapshot."""

from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs" / "product-technical-gap-baseline.md"
CHANGELOG = ROOT / "CHANGELOG.md"


class GapSnapshotInventoryConsistencyTests(unittest.TestCase):
    """Prevent one dated snapshot from carrying contradictory live PR totals."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.baseline = BASELINE.read_text(encoding="utf-8")
        cls.changelog = CHANGELOG.read_text(encoding="utf-8")

    def test_current_live_state_is_distinct_from_dated_snapshot(self) -> None:
        """The live section must bind evidence structurally without freezing transient stack heads."""
        current = self.baseline.split("## Current live delivery state", 1)[1].split(
            "## Observed snapshot: 2026-08-29", 1
        )[0]
        for marker in (
            "141 open pull requests",
            "26 Ready/non-draft",
            "115 Draft",
            "11 open non-PR issues",
            "542ca1e9c0a863595b8b6697790005d2471f5413",
            "18156473",
            "Live GitHub PR/base/head/check APIs are authoritative over PR bodies",
            "Issue #28 remains the P0 buyer-visible integration target",
            "PR #271 is Draft at exact head",
            "on exact #270 head",
            "Exact native CI run",
            "queued/non-passing",
            "Ready root #82 remains at exact head",
            "DDD ownership correction #272 is Draft at exact head",
            "#273 is its Draft context-map/ubiquitous-language child",
            "active-PR ancestry observations only",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)
        for non_passing in (
            "queued reviewer evidence is non-passing",
            "discard queued, skipped, cancelled, absent, predecessor, synthetic, status-only, and model-only evidence",
        ):
            with self.subTest(non_passing=non_passing):
                self.assertIn(non_passing.casefold(), current.casefold())

    def test_current_baseline_inventory_matches_the_verified_snapshot(self) -> None:
        """The dated 2026-08-29 snapshot must keep its exact historical inventory."""
        current = self.baseline.split("### Open pull requests", 1)[1].split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[0]
        for marker in (
            "108 open pull requests",
            "24 non-draft",
            "84 draft",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)

        for stale in (
            "128 open pull requests",
            "126 open pull requests",
            "54 non-draft",
            "72 draft",
            "115 open pull requests",
            "31 non-draft",
            "116 open pull requests",
            "32 non-draft",
            "111 open pull requests",
            "27 non-draft",
            "153 open pull requests",
            "114 draft PRs",
        ):
            with self.subTest(stale=stale):
                self.assertNotIn(stale, current)

    def test_unreleased_changelog_uses_one_current_inventory(self) -> None:
        """The historical Unreleased entry remains internally self-consistent."""
        unreleased = self.changelog.split("## [Unreleased]", 1)[1]
        preamble, remainder = unreleased.split("### Added", 1)
        added = remainder.split("### Changed", 1)[0]

        expected = "108 open pull requests (24 ready, 84 draft)"
        self.assertIn(expected, preamble)
        self.assertIn("on 2026-08-29", preamble)
        self.assertNotIn("on 2026-08-28", preamble)
        self.assertIn(expected, added)
        self.assertNotIn("126 open pull requests (54 ready, 72 draft)", preamble)
        self.assertNotIn("115 open pull requests (31 ready, 84 draft)", preamble)
        self.assertNotIn("126 open pull requests (54 ready, 72 draft)", added)
        self.assertNotIn("110 open pull requests (26 ready, 84 draft)", preamble)

    def test_current_snapshot_records_recent_stack_merges_and_revalidation(self) -> None:
        """A stacked merge must update the dated queue and parent exact-head evidence."""
        current = self.baseline.split("### Open pull requests", 1)[1].split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[0]
        record = self.baseline.split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[1].split("#### Current exact-head active PR evidence", 1)[0]
        self.assertIn("108 open pull requests", current)
        self.assertIn("24 non-draft", current)
        self.assertIn("#217 was squash-merged", record)
        self.assertIn("66f360ccac5cec60c72222cc79d58e39f6f00088", record)
        self.assertIn("#67 was squash-merged", record)
        self.assertIn("5021d142583cb5a8e393248048bb824762a98056", record)
        self.assertIn("PR #64 consequently advanced", record)
        self.assertIn(
            "| #210 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | "
            "`7946dce9a3dd074047d93fca299d48c7aef40e47` |",
            self.baseline,
        )
        self.assertNotIn("| #217 |", current)

    def test_current_documentation_head_records_security_and_review_state(self) -> None:
        """The dated self-referential record must preserve its historical gate truth."""
        record = self.baseline.split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[1].split("#### Current exact-head active PR evidence", 1)[0]
        for marker in (
            "The immediately preceding PR #238 head `d0b0d1ed92f891f14646fc673b8e1c0d912586fd` remains historical",
            "automatic OpenCode run `33193822920` / job `98926243116` failed closed",
            "current Strix run `33193822929` / job `98925769697` succeeded",
            "central dispatch run `33194506918` / job `98928580387` also failed closed at OpenCode",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, record)


if __name__ == "__main__":
    unittest.main()
