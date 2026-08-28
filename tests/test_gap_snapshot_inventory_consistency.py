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

    def test_current_baseline_inventory_matches_the_verified_snapshot(self) -> None:
        """The current snapshot must use the exact 109/25/84 inventory observation."""
        current = self.baseline.split("### Open pull requests", 1)[1].split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[0]
        for marker in (
            "109 open pull requests",
            "25 non-draft",
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
            "114 draft",
        ):
            with self.subTest(stale=stale):
                self.assertNotIn(stale, current)

    def test_unreleased_changelog_uses_one_current_inventory(self) -> None:
        """The Unreleased current snapshot must agree before and inside Added."""
        unreleased = self.changelog.split("## [Unreleased]", 1)[1]
        preamble, remainder = unreleased.split("### Added", 1)
        added = remainder.split("### Changed", 1)[0]

        expected = "109 open pull requests (25 ready, 84 draft)"
        self.assertIn(expected, preamble)
        self.assertIn("on 2026-08-29", preamble)
        self.assertNotIn("on 2026-08-28", preamble)
        self.assertIn(expected, added)
        self.assertNotIn("126 open pull requests (54 ready, 72 draft)", preamble)
        self.assertNotIn("115 open pull requests (31 ready, 84 draft)", preamble)
        self.assertNotIn("126 open pull requests (54 ready, 72 draft)", added)
        self.assertNotIn("110 open pull requests (26 ready, 84 draft)", preamble)

    def test_current_snapshot_records_pr217_merge_and_pr210_revalidation(self) -> None:
        """A stacked merge must update the live queue and parent exact-head evidence."""
        current = self.baseline.split("### Open pull requests", 1)[1].split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[0]
        record = self.baseline.split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[1].split("#### Current exact-head active PR evidence", 1)[0]
        self.assertIn("109 open pull requests", current)
        self.assertIn("25 non-draft", current)
        self.assertIn("#217 was squash-merged", record)
        self.assertIn("66f360ccac5cec60c72222cc79d58e39f6f00088", record)
        self.assertIn(
            "| #210 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | "
            "`66f360ccac5cec60c72222cc79d58e39f6f00088` |",
            self.baseline,
        )
        self.assertNotIn("| #217 |", current)

    def test_current_documentation_head_records_security_and_review_state(self) -> None:
        """The self-referential documentation PR must preserve current-head gate truth."""
        record = self.baseline.split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[1].split("#### Current exact-head active PR evidence", 1)[0]
        for marker in (
            "PR #238 current exact head is `d0b0d1ed92f891f14646fc673b8e1c0d912586fd`",
            "automatic OpenCode run `33193822920` / job `98926243116` failed closed",
            "current Strix run `33193822929` / job `98925769697` succeeded",
            "Central dispatch run `33194506918` / job `98928580387` also failed closed at OpenCode",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, record)


if __name__ == "__main__":
    unittest.main()
