"""Regression contracts for bounded freshness-authority documentation."""

from __future__ import annotations

import pathlib
import unittest
from unittest import mock

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
                self.assertIn("Protected-main capability status:** **PARTIAL", text)
                if filename == "resolution-freshness-authority.md":
                    self.assertIn("PR #50 remains open", text)
                    self.assertIn(
                        "first-party planning remains on open PR #50; socket-use remains closed, unmerged PR #54 branch evidence",
                        text,
                    )
                else:
                    self.assertIn("Active-PR traceability", text)
                    self.assertTrue(
                        "not protected-main truth" in text
                        or "protected-main primitive evidence" in text
                    )

    def test_resolution_trace_requires_socket_use_freshness_not_only_plan_time(self) -> None:
        """The DNS freshness trace must retain the delayed-use boundary added by PR #54."""
        text = (TRACEABILITY / "resolution-freshness-authority.md").read_text(encoding="utf-8")
        self.assertIn("Socket-use freshness lane:** PR #54", text)
        self.assertIn("connect_at(current_time)", text)
        self.assertIn("rechecks the retained freshness authority immediately before socket I/O", text)
        self.assertIn("delayed call cannot reuse plan-time freshness", text)

    def test_resolution_trace_matches_live_pr_maturity(self) -> None:
        """Merged primitive and unmerged consumers must not share one maturity label."""
        text = (TRACEABILITY / "resolution-freshness-authority.md").read_text(encoding="utf-8")
        self.assertIn("merged PR #47", text)
        self.assertIn("IMPLEMENTED_ON_PROTECTED_MAIN", text)
        self.assertIn("PR #50 remains open", text)
        self.assertIn("#54 is closed without merge", text)
        self.assertIn("overall protected-main resolution-to-socket interval remains **PARTIAL**", text)
        self.assertNotIn("PR #47, #50 and #54 remain **IMPLEMENTED_ON_ACTIVE_PR**", text)

    def test_traceability_index_matches_merged_resolution_primitive(self) -> None:
        """The canonical index must not leave merged PR #47 in active-only status."""
        index = (TRACEABILITY / "README.md").read_text(encoding="utf-8")
        self.assertIn("merged PR #47 bounds the lifetime", index)
        self.assertIn("| Bounded resolution freshness is explicit before destination authority is consumed | IMPLEMENTED_ON_PROTECTED_MAIN |", index)
        self.assertIn("merged `originweave-destination` primitive from PR #47", index)
        self.assertIn("ADR 0004 (Accepted); merged PR #47 tightens the existing boundary", index)
        self.assertIn("the merged resolution-freshness primitive remains distinct from the unshipped socket consumer", index)
        self.assertNotIn("active `originweave-destination` work in PR #47", index)
        self.assertNotIn("active PR #47 tightens the existing boundary", index)
        self.assertNotIn("resolution freshness remains an active lower-layer primitive", index)

    def test_merged_revocation_primitive_matches_live_maturity(self) -> None:
        """Current TLS freshness docs must distinguish merged #48 from the incomplete path."""
        trace = (TRACEABILITY / "tls-revocation-freshness-authority.md").read_text(encoding="utf-8")
        index = (TRACEABILITY / "README.md").read_text(encoding="utf-8")
        fitness = (ROOT / "docs" / "DOCUMENTATION_FITNESS.md").read_text(encoding="utf-8")
        self.assertIn("merged PR #48", trace)
        self.assertIn("IMPLEMENTED_ON_PROTECTED_MAIN", trace)
        self.assertIn("Merged PR #48 adds a freshness classifier only", index)
        self.assertIn("IMPLEMENTED_ON_PROTECTED_MAIN", index)
        self.assertIn("Merged #48 provides a protected-main bounded freshness primitive", fitness)
        self.assertNotIn("active PR #48", index)
        self.assertNotIn("Active #48 provides", fitness)

    def test_sensitive_requirement_and_open_work_match_merged_lifecycle_evidence(self) -> None:
        """Each sensitive-data trace must separate merged evidence from open reservation work."""
        index = (TRACEABILITY / "README.md").read_text(encoding="utf-8")
        rows = [
            line
            for line in index.splitlines()
            if line.startswith("| Purpose-bound sensitive-data policy/evidence |")
        ]
        self.assertEqual(len(rows), 1)
        self.assertIn("merged PR #45 records credential-free handle-lifecycle evidence", rows[0])
        self.assertIn("open PR #46 adds authoritative use reservation", rows[0])
        self.assertIn("PARTIAL", rows[0])
        self.assertNotIn("active lifecycle/reservation work #45/#46", rows[0])
        open_items = [
            line
            for line in index.splitlines()
            if line.startswith("- **Open:**") and "trusted-broker" in line
        ]
        self.assertEqual(len(open_items), 1)
        self.assertIn("PR #45 lifecycle evidence is on protected main", open_items[0])
        self.assertIn("PR #46 reservation remains open", open_items[0])
        self.assertIn("trusted-broker storage/revocation/value-resolution/model-disclosure", open_items[0])
        self.assertNotIn("after #45/#46 integrate", open_items[0])

    def test_sensitive_disclosure_contract_rejects_relocated_or_duplicate_rows(self) -> None:
        """Unrelated historical prose cannot satisfy the current decision-row contract."""
        index = (TRACEABILITY / "README.md").read_text(encoding="utf-8")
        rows = [
            line
            for line in index.splitlines()
            if line.startswith("| Sensitive disclosure is purpose- and classification-bound |")
        ]
        self.assertEqual(len(rows), 1)
        row = rows[0]
        unrelated = "\nHistorical quotation only: " + row + "\n"
        incorrect = (
            "| Sensitive disclosure is purpose- and classification-bound | "
            "IMPLEMENTED_ON_PROTECTED_MAIN | ADR 0007 | "
            "Authoritative revocation and trusted broker are complete |"
        )
        variants = {
            "false_shipment_with_correct_text_elsewhere": index.replace(row, incorrect) + unrelated,
            "missing_row_with_correct_text_elsewhere": index.replace(row, "") + unrelated,
            "duplicate_row": index.replace(row, row + "\n" + row),
        }
        for name, text in variants.items():
            with self.subTest(variant=name):
                with mock.patch.object(pathlib.Path, "read_text", return_value=text):
                    with self.assertRaises(AssertionError):
                        self.test_sensitive_disclosure_row_matches_live_maturity()

    def test_sensitive_disclosure_row_matches_live_maturity(self) -> None:
        """The current index must distinguish merged evidence from open handle lanes."""
        index = (TRACEABILITY / "README.md").read_text(encoding="utf-8")
        rows = [
            line
            for line in index.splitlines()
            if line.startswith("| Sensitive disclosure is purpose- and classification-bound |")
        ]
        self.assertEqual(len(rows), 1)
        row = rows[0]
        self.assertIn("| PARTIAL |", row)
        self.assertIn("merged PR #45 adds credential-free handle-lifecycle evidence", row)
        self.assertIn("PR #53 merged into the #46 stack, not protected main", row)
        self.assertIn("authoritative revocation remains unshipped", row)
        self.assertNotIn("merged PR #53 adds bounded in-process revocation state", row)
        self.assertIn("Open PR #46 adds authoritative use reservation", row)
        self.assertIn("#55 adds audience binding", row)
        self.assertIn("trusted storage/value resolution/cross-process lifecycle/model-disclosure remain open", row)
        self.assertNotIn("active PR #45 adds", row)


if __name__ == "__main__":
    unittest.main()
