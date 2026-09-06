"""Regression contracts for discoverability of the commercial gap baseline."""

from __future__ import annotations

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs" / "product-technical-gap-baseline.md"


class ProductGapDiscoverabilityContractTests(unittest.TestCase):
    """Keep the code-current buyer gap baseline reachable from canonical indexes."""

    def test_gap_baseline_exists_and_is_linked_from_canonical_indexes(self) -> None:
        """Architecture and documentation readers must not reconstruct the baseline from PR history."""
        self.assertTrue(BASELINE.is_file())
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        documentation_index = (ROOT / "docs" / "README.md").read_text(encoding="utf-8")
        self.assertIn("docs/product-technical-gap-baseline.md", architecture)
        self.assertIn("product-technical-gap-baseline.md", documentation_index)

    def test_bap_lifecycle_adr_is_indexed_without_premature_acceptance(self) -> None:
        """Restoring the ADR file must restore discoverability without changing its lifecycle."""
        adr = (ROOT / "docs" / "adr" / "0016-bap-task-lifecycle-authority.md").read_text(
            encoding="utf-8"
        )
        index = (ROOT / "docs" / "adr" / "README.md").read_text(encoding="utf-8")
        self.assertIn("- Status: Proposed", adr)
        self.assertIn("0016-bap-task-lifecycle-authority.md", index)
        self.assertIn("| Proposed |", index)


if __name__ == "__main__":
    unittest.main()
