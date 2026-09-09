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
        self.assertIn("## Current exact observation", text)
        self.assertIn("## Buyer gap matrix", text)
        self.assertIn("## Evidence and history index", text)
        self.assertLess(lines.index("## Buyer gap matrix"), 90)
        self.assertNotIn("### Previous verified cut:", text)
        self.assertIn(ARCHIVE.name, text)

    def test_historical_dossier_is_preserved_outside_the_decision_surface(self) -> None:
        self.assertTrue(ARCHIVE.is_file())
        archive = ARCHIVE.read_text(encoding="utf-8")

        self.assertGreater(len(archive.encode("utf-8")), 180_000)
        self.assertIn("### Previous verified cut: 2026-09-08", archive)
        self.assertIn("## Observed snapshot: 2026-08-29", archive)
        self.assertIn("#### Published session-end reply binding: 05:35 UTC", archive)


if __name__ == "__main__":
    unittest.main()
