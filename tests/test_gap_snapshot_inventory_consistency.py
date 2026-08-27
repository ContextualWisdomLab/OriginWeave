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
        """The current snapshot must use the exact 126/54/72 inventory observation."""
        current = self.baseline.split("### Open pull requests", 1)[1].split(
            "#### 2026-08-26 maintenance-loop record", 1
        )[0]
        for marker in (
            "126 open pull requests",
            "54 non-draft",
            "72 draft",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)

        for stale in (
            "128 open pull requests",
            "74 draft",
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

        expected = "126 open pull requests (54 ready, 72 draft)"
        self.assertIn(expected, preamble)
        self.assertIn(expected, added)
        self.assertNotIn("128 open pull requests (54 ready, 74 draft)", preamble)
        self.assertNotIn("153 open pull requests (39 ready, 114 draft)", added)


if __name__ == "__main__":
    unittest.main()
