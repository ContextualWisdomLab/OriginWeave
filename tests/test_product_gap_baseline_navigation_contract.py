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


def single_markdown_table_row(text: str, row_label: str) -> str:
    """Return the unique buyer-matrix row identified by its stable row label."""
    prefix = f"| {row_label} |"
    rows = [line for line in text.splitlines() if line.startswith(prefix)]
    if len(rows) != 1:
        raise AssertionError(
            f"expected exactly one markdown table row for {row_label!r}, found {len(rows)}"
        )
    return rows[0]


class ProductGapBaselineNavigationContractTests(unittest.TestCase):
    """Keep current decisions reachable without discarding immutable history."""

    def test_current_baseline_is_bounded_and_reaches_buyer_matrix_early(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        lines = text.splitlines()

        self.assertLessEqual(len(lines), 220)
        self.assertLessEqual(len(text.encode("utf-8")), 24_000)
        self.assertIn("## Observed delivery cut — 2026-09-21 04:07 UTC", text)
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
        self.assertIn("135 open pull requests: 6 Ready/non-draft and 129 Draft", text)
        self.assertIn("19 open non-PR issues", text)
        self.assertIn("#309 is merged into this #238 documentation lineage", text)
        self.assertIn("#238 is Ready", text)
        self.assertNotIn("includes this #309 Draft successor", text)
        self.assertIn("Live GitHub state supersedes this cut after 2026-09-21 04:07 UTC.", text)

    def test_navigation_traceability_tracks_current_receipt_and_demotes_previous_cuts(self) -> None:
        text = TRACEABILITY.read_text(encoding="utf-8")

        self.assertIn("2026-09-21 04:07 UTC", text)
        self.assertIn("135 open PRs / 6 Ready/non-draft / 129 Draft / 19 open non-PR issues", text)
        self.assertIn("#309 remains merged into the #238 documentation lineage", text)
        self.assertIn("Historical 2026-09-21 03:59 receipt", text)
        self.assertIn("Historical 2026-09-20 receipt", text)
        self.assertIn("Historical 2026-09-15 receipt", text)
        self.assertIn("135 open PRs / 13 Ready/non-draft / 122 Draft / 19 open non-PR issues", text)
        self.assertIn("Historical 2026-09-09 receipt", text)
        self.assertIn("130 open PRs / 12 Ready / 118 Draft / 14 open non-PR issues", text)

    def test_current_unicode_row_tracks_stable_publication_and_exact_candidate(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        traceability = TRACEABILITY.read_text(encoding="utf-8")
        unicode_row_label = (
            "Controlled-benchmark Unicode evidence identity has stable publication provenance"
        )
        unicode_row = single_markdown_table_row(text, unicode_row_label)

        for marker in (
            "4e70d5ed9ce13f7b59012d39646e94ac41519c89",
            "unicode-18.0.0-default-ignorable-exclusion",
            "1,159,889 bytes",
            "09c928886a178fcafd93c29e4bd59073a058e5a100b716d425cb563ab50f68c9",
            "27 source entries / 4,174 scalars",
            "29,218 bytes",
            "673264e62183e35f6055a2ad4940403e706669e0750fcc5d56a99f158fb3bb93",
            "35629802868",
            "35629802889",
            "terminal runner-backed execution",
            "no blocking issue",
            "hostile expected / valid observed",
            "valid expected / hostile observed",
            "#325",
            "OPEN — LEAF GREEN, STACK BLOCKED",
        ):
            self.assertIn(marker, unicode_row)

        self.assertNotIn("2026-08-07 pre-release Unicode 18.0.0 UCD snapshot", unicode_row)
        self.assertNotIn("At or after final Unicode 18.0.0 UCD publication", unicode_row)

        traceability_lines = traceability.splitlines()
        decision_start = "## Decision"
        decision_end = "## Historical-evidence boundary"
        self.assertEqual(1, traceability_lines.count(decision_start))
        self.assertEqual(1, traceability_lines.count(decision_end))
        decision_start_index = traceability_lines.index(decision_start)
        decision_end_index = traceability_lines.index(decision_end)
        self.assertLess(decision_start_index, decision_end_index)
        decision_lines = traceability_lines[decision_start_index + 1 : decision_end_index]
        decision = "\n".join(decision_lines)

        for marker in (
            "4e70d5ed9ce13f7b59012d39646e94ac41519c89",
            "35629802868",
            "35629802889",
            "valid expected / hostile observed",
            "hostile expected / valid observed",
            "no blocking issue",
        ):
            self.assertIn(marker, decision)

        delivery_lines = [
            line for line in decision_lines if "Delivery remains" in line
        ]
        self.assertEqual(1, len(delivery_lines))
        delivery = delivery_lines[0]
        self.assertLess(delivery.index("#237"), delivery.index("#322"))
        self.assertLess(delivery.index("#322"), delivery.index("#324"))
        self.assertIn("ordinary/non-force", delivery)
        self.assertIn("fresh exact-head", delivery)
        self.assertIn("new exact head", delivery)
        self.assertIn("new evidence", delivery)

    def test_unicode_candidate_evidence_cannot_fall_back_to_other_sections(self) -> None:
        row_label = (
            "Controlled-benchmark Unicode evidence identity has stable publication provenance"
        )
        exact_head = "4e70d5ed9ce13f7b59012d39646e94ac41519c89"
        synthetic = (
            "## Buyer gap matrix\n"
            "| Buyer-visible gap | Evidence at this cut | Canonical owner / next acceptance | State |\n"
            "| --- | --- | --- | --- |\n"
            f"| {row_label} | stale candidate evidence | #325 | **OPEN** |\n\n"
            "## Evidence and history index\n"
            f"Historical predecessor evidence retained at {exact_head}.\n"
        )

        self.assertIn(exact_head, synthetic)
        self.assertNotIn(exact_head, single_markdown_table_row(synthetic, row_label))

    def test_changelog_preserves_the_previous_dated_inventory_receipt(self) -> None:
        baseline = BASELINE.read_text(encoding="utf-8")
        changelog = CHANGELOG.read_text(encoding="utf-8")
        inventory = [
            line
            for line in changelog.splitlines()
            if line.startswith("- Current delivery inventory:")
        ]

        self.assertIn("2026-09-21 04:07 UTC", baseline)
        self.assertEqual(1, len(inventory))
        self.assertIn("130 open pull requests (12 ready, 118 draft)", inventory[0])
        self.assertIn("14 open non-PR issues", inventory[0])
        self.assertIn("Observed 2026-09-09 12:57 UTC", inventory[0])
        self.assertNotIn("135 open pull requests (6 ready, 129 draft)", inventory[0])
