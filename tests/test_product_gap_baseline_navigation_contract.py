"""Buyer-facing navigation contract for the live product-gap decision surface."""

from __future__ import annotations

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs/product-technical-gap-baseline.md"
ARCHIVE = ROOT / "docs/evidence/product-technical-gap-baseline-through-2026-09-09.md"


class ProductGapBaselineNavigationContractTests(unittest.TestCase):
    """Keep current decisions reachable without discarding immutable history."""

    def test_current_baseline_is_bounded_and_reaches_buyer_matrix_early(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        lines = text.splitlines()

        self.assertLessEqual(len(lines), 220)
        self.assertLessEqual(len(text.encode("utf-8")), 24_000)
        self.assertIn("## Observed delivery cut — 2026-09-09 12:57 UTC", text)
        self.assertIn("## Buyer gap matrix", text)
        self.assertIn("## Evidence and history index", text)
        self.assertLess(lines.index("## Buyer gap matrix"), 90)
        self.assertNotIn("### Previous verified cut:", text)
        self.assertIn(ARCHIVE.name, text)

    def test_observation_receipt_is_self_inclusive_and_not_continuously_live(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")

        self.assertNotIn("## Current exact observation", text)
        self.assertIn("This is a dated observation receipt, not a continuously live counter.", text)
        self.assertIn("130 open pull requests: 12 Ready/non-draft and 118 Draft", text)
        self.assertIn("includes this #309 Draft successor", text)
        self.assertIn("Live GitHub state supersedes this cut after 2026-09-09 12:57 UTC.", text)

    def test_historical_dossier_is_preserved_outside_the_decision_surface(self) -> None:
        self.assertTrue(ARCHIVE.is_file())
        archive = ARCHIVE.read_text(encoding="utf-8")

        self.assertGreater(len(archive.encode("utf-8")), 180_000)
        self.assertIn("### Previous verified cut: 2026-09-08", archive)
        self.assertIn("## Observed snapshot: 2026-08-29", archive)
        self.assertIn("#### Published session-end reply binding: 05:35 UTC", archive)


if __name__ == "__main__":
    unittest.main()
