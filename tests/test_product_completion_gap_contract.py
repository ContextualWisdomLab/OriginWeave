"""Regression contract for the code-current commercial gap baseline."""

from __future__ import annotations

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs" / "product-technical-gap-baseline.md"


class ProductCompletionGapContractTests(unittest.TestCase):
    """Keep buyer gaps tied to the current repository snapshot and authority model."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.text = BASELINE.read_text(encoding="utf-8")

    def test_baseline_records_current_inventory_and_protected_head(self) -> None:
        """The current snapshot must not retain the superseded August inventory as current."""
        for phrase in (
            "## Observed snapshot: 2026-09-06",
            "87c4daa1830bac5a5228b6036752ad5633232085",
            "125 open pull requests",
            "12 non-draft",
            "113 draft",
            "13 open non-PR issues",
            "0 GitHub Releases",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, self.text)

        for stale_phrase in (
            "126 open pull requests",
            "54 non-draft and 72 draft",
            "Protected `main` is at `b05d5acca82b9d916ada2c8e82f59f92a89817e1`",
        ):
            with self.subTest(stale_phrase=stale_phrase):
                self.assertNotIn(stale_phrase, self.text)

    def test_current_blockers_and_owner_paths_are_explicit(self) -> None:
        """Commercial completion must retain the exercised browser and control-plane gaps."""
        for phrase in (
            "89708cf5e474f7701513b84a1356a8ce1699bef5",
            "#242",
            "0135984f1bc1f68d89d7777f49c4999474105a12",
            "failure_stage=session_create",
            "Issue #212",
            "Issue #279",
            "ContextualWisdomLab/.github#712",
            "No predecessor GREEN transfers",
            "Command ACK is never sufficient",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, self.text)

    def test_release_and_dependency_boundaries_remain_fail_closed(self) -> None:
        """The baseline must require an immutable release and versioned owner contracts."""
        for phrase in (
            "signed cross-platform artifacts",
            "SBOM/provenance",
            "reproducibility",
            "rollback",
            "released/versioned contracts or ACLs",
            "cross-service SQL",
            "mutable sibling heads",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, self.text)


if __name__ == "__main__":
    unittest.main()
