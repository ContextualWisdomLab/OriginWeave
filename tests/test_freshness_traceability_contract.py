"""Regression contracts for bounded freshness-authority documentation."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
TRACEABILITY = ROOT / "docs" / "traceability"


class FreshnessTraceabilityContractTests(unittest.TestCase):
    """Keep active freshness primitives discoverable without promoting them to shipped truth."""

    def test_traceability_index_discovers_each_active_freshness_authority(self) -> None:
        """Resolution and TLS freshness traces must be linked from the canonical index."""
        index = (TRACEABILITY / "README.md").read_text(encoding="utf-8")
        for filename in (
            "resolution-freshness-authority.md",
            "tls-revocation-freshness-authority.md",
        ):
            with self.subTest(filename=filename):
                self.assertTrue((TRACEABILITY / filename).is_file())
                self.assertIn(f"]({filename})", index)

    def test_active_freshness_traces_preserve_protected_main_maturity(self) -> None:
        """Active implementation evidence must remain explicitly non-shipped and partial overall."""
        for filename in (
            "resolution-freshness-authority.md",
            "tls-revocation-freshness-authority.md",
        ):
            text = (TRACEABILITY / filename).read_text(encoding="utf-8")
            with self.subTest(filename=filename):
                self.assertIn("Active-PR traceability", text)
                self.assertIn("Protected-main capability status:** **PARTIAL", text)
                self.assertIn("IMPLEMENTED_ON_ACTIVE_PR", text)
                self.assertIn("not protected-main truth", text)


if __name__ == "__main__":
    unittest.main()
