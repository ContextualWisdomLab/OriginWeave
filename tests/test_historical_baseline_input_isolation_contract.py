"""Regression contracts for immutable historical baseline inputs and navigation."""

from __future__ import annotations

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs" / "evidence"
LOADER = ROOT / "tests" / "_historical_baseline_contract_loader.py"
NAVIGATION = EVIDENCE / "product-technical-gap-baseline-through-2026-09-09-navigation.md"
ARCHIVE_ROOT = EVIDENCE / "historical-contract-root-through-2026-09-09"

ARCHIVED_INPUTS = {
    "ARCHIVE_FITNESS": EVIDENCE / "DOCUMENTATION_FITNESS-through-2026-09-09.md",
    "ARCHIVE_MATURITY": EVIDENCE / "active-pr-maturity-through-2026-09-09.md",
    "ARCHIVE_AGENTS": EVIDENCE / "AGENTS-through-2026-09-09.md",
    "ARCHIVE_EVIDENCE_SCRIPT": EVIDENCE / "collect-live-merge-evidence-through-2026-09-09.sh",
}


class HistoricalBaselineInputIsolationContractTests(unittest.TestCase):
    """Historical receipts must not silently depend on mutable current controls."""

    def test_loader_redirects_every_historical_control_input(self) -> None:
        loader = LOADER.read_text(encoding="utf-8")
        for constant, path in ARCHIVED_INPUTS.items():
            with self.subTest(constant=constant):
                self.assertTrue(path.is_file(), f"missing immutable input: {path}")
                self.assertIn(constant, loader)

        for assignment in (
            "module.FITNESS = ARCHIVE_FITNESS",
            "module.MATURITY = ARCHIVE_MATURITY",
            "module.AGENTS = ARCHIVE_AGENTS",
            "module.EVIDENCE_SCRIPT = ARCHIVE_EVIDENCE_SCRIPT",
        ):
            with self.subTest(assignment=assignment):
                self.assertIn(assignment, loader)

    def test_direct_root_reads_are_redirected_without_rewriting_legacy_contracts(self) -> None:
        loader = LOADER.read_text(encoding="utf-8")
        self.assertIn("ARCHIVE_ROOT", loader)
        self.assertIn("module.ROOT = ARCHIVE_ROOT", loader)
        self.assertTrue((ARCHIVE_ROOT / "CHANGELOG.md").is_file())
        self.assertTrue((ARCHIVE_ROOT / "scripts/ci/collect_live_merge_evidence.sh").is_file())

    def test_relocated_archive_has_an_original_location_navigation_receipt(self) -> None:
        self.assertTrue(NAVIGATION.is_file())
        receipt = NAVIGATION.read_text(encoding="utf-8")
        for marker in (
            "Original relative-link base: `docs/`",
            "`../scripts/ci/collect_live_merge_evidence.sh` → `scripts/ci/collect_live_merge_evidence.sh`",
            "`doctoring.md` → `docs/doctoring.md`",
            "`PRD.md` → `docs/PRD.md`",
            "`TRD.md` → `docs/TRD.md`",
            "Do not rewrite the archived baseline blob",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, receipt)


if __name__ == "__main__":
    unittest.main()
