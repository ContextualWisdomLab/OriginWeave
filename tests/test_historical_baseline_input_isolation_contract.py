"""Regression contracts for immutable historical baseline inputs and navigation."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from types import ModuleType
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


def load_contract_loader() -> ModuleType:
    """Import the historical loader so its returned bindings can be verified."""
    spec = importlib.util.spec_from_file_location(
        "_historical_baseline_contract_loader_contract",
        LOADER,
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load historical baseline contract loader")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class HistoricalBaselineInputIsolationContractTests(unittest.TestCase):
    """Historical receipts must not silently depend on mutable current controls."""

    def test_loader_redirects_every_historical_control_input(self) -> None:
        loader = load_contract_loader()
        for constant, path in ARCHIVED_INPUTS.items():
            with self.subTest(constant=constant):
                self.assertTrue(path.is_file(), f"missing immutable input: {path}")
                self.assertEqual(path, getattr(loader, constant))

        documentation = loader.load_historical_contract(
            "legacy_documentation_active_pr_evidence_contract.py",
            "_probe_historical_documentation_contract",
        )
        self.assertEqual(loader.ARCHIVE, documentation.BASELINE)
        self.assertEqual(loader.ARCHIVE_ROOT, documentation.ROOT)
        self.assertEqual(loader.ARCHIVE_FITNESS, documentation.FITNESS)
        self.assertEqual(loader.ARCHIVE_MATURITY, documentation.MATURITY)

        evidence = loader.load_historical_contract(
            "legacy_live_gap_evidence_integrity_contract.py",
            "_probe_historical_evidence_contract",
        )
        self.assertEqual(loader.ARCHIVE, evidence.BASELINE)
        self.assertEqual(loader.ARCHIVE_ROOT, evidence.ROOT)
        self.assertEqual(loader.ARCHIVE_AGENTS, evidence.AGENTS)
        self.assertEqual(loader.ARCHIVE_EVIDENCE_SCRIPT, evidence.EVIDENCE_SCRIPT)

        snapshot = loader.load_historical_contract(
            "legacy_gap_snapshot_inventory_consistency.py",
            "_probe_historical_snapshot_contract",
        )
        self.assertEqual(loader.ARCHIVE, snapshot.BASELINE)
        self.assertEqual(loader.ARCHIVE_ROOT, snapshot.ROOT)
        self.assertEqual(loader.ARCHIVE_CHANGELOG, snapshot.CHANGELOG)

        completion = loader.load_historical_contract(
            "legacy_product_completion_gap_contract.py",
            "_probe_historical_completion_contract",
        )
        self.assertEqual(loader.ARCHIVE, completion.BASELINE)
        self.assertEqual(loader.ARCHIVE_ROOT, completion.ROOT)

    def test_direct_root_reads_are_redirected_without_rewriting_legacy_contracts(self) -> None:
        loader = load_contract_loader()
        self.assertEqual(ARCHIVE_ROOT, loader.ARCHIVE_ROOT)
        self.assertTrue((ARCHIVE_ROOT / "CHANGELOG.md").is_file())
        self.assertTrue((ARCHIVE_ROOT / "scripts/ci/collect_live_merge_evidence.sh").is_file())

    def test_relocated_archive_has_an_original_location_navigation_receipt(self) -> None:
        self.assertTrue(NAVIGATION.is_file())
        receipt = NAVIGATION.read_text(encoding="utf-8")
        for marker in (
            "Original relative-link base: `docs/`",
            "| `../scripts/ci/collect_live_merge_evidence.sh` | `scripts/ci/collect_live_merge_evidence.sh` |",
            "| `doctoring.md` | `docs/doctoring.md` |",
            "| `doctoring/browser-agent-protocols.md` | `docs/doctoring/browser-agent-protocols.md` |",
            "| `product-roadmap.md` | `docs/product-roadmap.md` |",
            "| `PRD.md` | `docs/PRD.md` |",
            "| `TRD.md` | `docs/TRD.md` |",
            "Do not rewrite the archived baseline blob",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, receipt)


if __name__ == "__main__":
    unittest.main()
