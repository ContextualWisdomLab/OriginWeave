"""Buyer-facing navigation contract for the live product-gap decision surface."""

from __future__ import annotations

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs/product-technical-gap-baseline.md"
TRACEABILITY = ROOT / "docs/traceability/product-gap-baseline-navigation.md"
ARCHIVE = ROOT / "docs/evidence/product-technical-gap-baseline-through-2026-09-09.md"
ARCHIVE_NAVIGATION = ROOT / "docs/evidence/product-technical-gap-baseline-through-2026-09-09-navigation.md"
ARCHIVE_CHANGELOG = ROOT / "docs/evidence/CHANGELOG-through-2026-09-09.md"
CHECKPOINT = ROOT / "docs/evidence/2026-09-06-delivery-checkpoint.md"
CHANGELOG = ROOT / "CHANGELOG.md"
LOADER = ROOT / "tests/_historical_baseline_contract_loader.py"


class ProductGapBaselineNavigationContractTests(unittest.TestCase):
    """Keep current decisions reachable without discarding immutable history."""

    def test_current_baseline_is_bounded_and_reaches_buyer_matrix_early(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        lines = text.splitlines()

        self.assertLessEqual(len(lines), 220)
        self.assertLessEqual(len(text.encode("utf-8")), 24_000)
        self.assertIn("## Observed delivery cut — 2026-09-20 22:00 UTC", text)
        self.assertIn("## Buyer gap matrix", text)
        self.assertIn("## Evidence and history index", text)
        self.assertLess(lines.index("## Buyer gap matrix"), 90)
        self.assertNotIn("### Previous verified cut:", text)
        self.assertIn(ARCHIVE.name, text)
        self.assertIn(ARCHIVE_NAVIGATION.name, text)

    def test_observation_receipt_is_self_inclusive_and_not_continuously_live(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")

        self.assertNotIn("## Current exact observation", text)
        self.assertIn("This is a dated observation receipt, not a continuously live counter.", text)
        self.assertIn("135 open pull requests: 5 Ready/non-draft and 130 Draft", text)
        self.assertIn("19 open non-PR issues", text)
        self.assertIn("#309 is merged into this #238 documentation lineage", text)
        self.assertNotIn("includes this #309 Draft successor", text)
        self.assertIn("Live GitHub state supersedes this cut after 2026-09-20 22:00 UTC.", text)

    def test_navigation_traceability_tracks_current_receipt_and_demotes_previous_cuts(self) -> None:
        text = TRACEABILITY.read_text(encoding="utf-8")

        self.assertIn("2026-09-20 22:00 UTC", text)
        self.assertIn("135 open PRs / 5 Ready/non-draft / 130 Draft / 19 open non-PR issues", text)
        self.assertIn("#309 is merged into the #238 documentation lineage", text)
        self.assertIn("Historical 2026-09-15 receipt", text)
        self.assertIn("135 open PRs / 13 Ready/non-draft / 122 Draft / 19 open non-PR issues", text)
        self.assertIn("Historical 2026-09-09 receipt", text)
        self.assertIn("130 open PRs / 12 Ready / 118 Draft / 14 open non-PR issues", text)

    def test_current_unicode_row_tracks_stable_publication_and_exact_candidate(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")

        self.assertIn("accdd2d194f21ae1444ccca5297ce6590bc5384e", text)
        self.assertIn("unicode-18.0.0-default-ignorable-exclusion", text)
        self.assertIn("1,159,889 bytes", text)
        self.assertIn(
            "09c928886a178fcafd93c29e4bd59073a058e5a100b716d425cb563ab50f68c9",
            text,
        )
        self.assertIn("27 source entries / 4,174 scalars", text)
        self.assertIn("29,218 bytes", text)
        self.assertIn(
            "673264e62183e35f6055a2ad4940403e706669e0750fcc5d56a99f158fb3bb93",
            text,
        )
        self.assertNotIn("2026-08-07 pre-release Unicode 18.0.0 UCD snapshot", text)
        self.assertNotIn("At or after final Unicode 18.0.0 UCD publication", text)

    def test_changelog_preserves_the_previous_dated_inventory_receipt(self) -> None:
        baseline = BASELINE.read_text(encoding="utf-8")
        changelog = CHANGELOG.read_text(encoding="utf-8")
        inventory = [
            line
            for line in changelog.splitlines()
            if line.startswith("- Current delivery inventory:")
        ]

        self.assertIn("2026-09-20 22:00 UTC", baseline)
        self.assertEqual(1, len(inventory))
        self.assertIn("130 open pull requests (12 ready, 118 draft)", inventory[0])
        self.assertIn("14 open non-PR issues", inventory[0])
        self.assertIn("Observed 2026-09-09 12:57 UTC", inventory[0])
        self.assertNotIn("135 open pull requests (5 ready, 130 draft)", inventory[0])

    def test_historical_dossier_is_preserved_outside_the_decision_surface(self) -> None:
        self.assertTrue(ARCHIVE.is_file())
        self.assertTrue(ARCHIVE_NAVIGATION.is_file())
        archive = ARCHIVE.read_text(encoding="utf-8")

        self.assertGreater(len(archive.encode("utf-8")), 180_000)
        self.assertIn("### Previous verified cut: 2026-09-08", archive)
        self.assertIn("### Latest verified cut: 2026-09-06", archive)
        self.assertIn("## Observed snapshot: 2026-08-29", archive)
        self.assertIn("#### Published session-end reply binding: 05:35 UTC", archive)

    def test_historical_checkpoint_links_to_the_preserved_dossier_anchor(self) -> None:
        checkpoint = CHECKPOINT.read_text(encoding="utf-8")

        self.assertIn(
            "product-technical-gap-baseline-through-2026-09-09.md#latest-verified-cut-2026-09-06",
            checkpoint,
        )
        self.assertNotIn(
            "../product-technical-gap-baseline.md#latest-verified-cut-2026-09-06",
            checkpoint,
        )

    def test_archive_navigation_receipt_maps_original_relative_links_in_two_columns(self) -> None:
        navigation = ARCHIVE_NAVIGATION.read_text(encoding="utf-8")
        for row in (
            "| `../scripts/ci/collect_live_merge_evidence.sh` | `scripts/ci/collect_live_merge_evidence.sh` |",
            "| `doctoring.md` | `docs/doctoring.md` |",
            "| `doctoring/browser-agent-protocols.md` | `docs/doctoring/browser-agent-protocols.md` |",
            "| `product-roadmap.md` | `docs/product-roadmap.md` |",
            "| `PRD.md` | `docs/PRD.md` |",
            "| `TRD.md` | `docs/TRD.md` |",
        ):
            with self.subTest(row=row):
                self.assertIn(row, navigation)

    def test_legacy_changelog_input_is_immutable_and_separate_from_live_changelog(self) -> None:
        self.assertTrue(ARCHIVE_CHANGELOG.is_file())
        archived = ARCHIVE_CHANGELOG.read_text(encoding="utf-8")
        live = CHANGELOG.read_text(encoding="utf-8")
        loader = LOADER.read_text(encoding="utf-8")

        self.assertIn("131 open pull requests (14 ready, 117 draft)", archived)
        self.assertIn("130 open pull requests (12 ready, 118 draft)", live)
        self.assertIn("ARCHIVE_CHANGELOG", loader)
        self.assertIn("module.CHANGELOG = ARCHIVE_CHANGELOG", loader)


if __name__ == "__main__":
    unittest.main()
