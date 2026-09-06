"""Regression contracts for the current product-gap inventory snapshot."""

from __future__ import annotations

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs" / "product-technical-gap-baseline.md"


class GapSnapshotInventoryConsistencyTests(unittest.TestCase):
    """Prevent the canonical snapshot from carrying contradictory live PR totals."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.baseline = BASELINE.read_text(encoding="utf-8")

    def test_current_inventory_is_internally_consistent(self) -> None:
        """Ready plus Draft counts must equal the recorded open-PR total."""
        match = re.search(
            r"\*\*(\d+) open pull requests: (\d+) non-draft and (\d+) draft\*\*",
            self.baseline,
        )
        self.assertIsNotNone(match)
        total, non_draft, draft = (int(value) for value in match.groups())
        self.assertEqual((total, non_draft, draft), (125, 12, 113))
        self.assertEqual(non_draft + draft, total)

    def test_current_snapshot_has_one_protected_main_identity(self) -> None:
        """The delivery boundary must name the current signed protected head."""
        current = self.baseline.split("## Observed snapshot: 2026-09-06", 1)[1]
        self.assertIn("87c4daa1830bac5a5228b6036752ad5633232085", current)
        self.assertNotIn("b05d5acca82b9d916ada2c8e82f59f92a89817e1", current)

    def test_evidence_procedure_requires_fresh_head_recheck(self) -> None:
        """A stored snapshot must never authorize stale-head promotion."""
        self.assertIn("Re-fetch the head and base immediately before any merge/readiness decision", self.baseline)
        self.assertIn("--paginate", self.baseline)
        self.assertIn("head_sha=89708cf5e474f7701513b84a1356a8ce1699bef5", self.baseline)


if __name__ == "__main__":
    unittest.main()
