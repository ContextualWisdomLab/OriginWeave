"""Buyer-facing contract for the active stacked HTTP interoperability gap."""

from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs/product-technical-gap-baseline.md"
ROW_LABEL = "Stacked HTTP `Content-Encoding` interoperability without budget bypass"


def single_markdown_table_row(text: str, row_label: str) -> str:
    """Return the unique buyer-matrix row identified by its stable row label."""
    prefix = f"| {row_label} |"
    rows = [line for line in text.splitlines() if line.startswith(prefix)]
    if len(rows) != 1:
        raise AssertionError(
            f"expected exactly one markdown table row for {row_label!r}, found {len(rows)}"
        )
    return rows[0]


class ProductGapHttpBaselineContractTests(unittest.TestCase):
    """Keep the active HTTP successor visible without promoting pending evidence."""

    def test_active_http_gap_is_explicit_and_not_promoted_to_acceptance(self) -> None:
        row = single_markdown_table_row(
            BASELINE.read_text(encoding="utf-8"),
            ROW_LABEL,
        )

        for marker in (
            "#37",
            "#326/#327",
            "2b44db8392e2c3de4dd4f15b98f77dddc2a2c5ea",
            "38 ahead / 0 behind",
            "p95 **20,000 µs**",
            "TCP/TLS transport setup",
            "not an accepted performance receipt",
            "ORIGINWEAVE_PERFORMANCE_SOURCE_REVISION",
            "ORIGINWEAVE_PERFORMANCE_ENVIRONMENT_ID",
            "budget_status",
            "source_acceptance_status",
            "network_acceptance_status",
            "evidence_acceptance_status",
            "acceptance_status",
            "UNACCEPTED_SOURCE_FALLBACK",
            "parent_untimed_connection_plan",
            "UNACCEPTED_PARENT_NETWORK_AUTHORITY",
            "caller_produced_unattested_receipt",
            "UNACCEPTED_UNATTESTED_RECEIPT",
            "#50",
            "ad87cfea59db711cb29ef90559790ba77e22029f",
            "FreshConnectionPlan",
            "released/pinned",
            "ContextualWisdomLab/.github#2162",
            "#2166",
            "authenticated attestation",
            "origin/integrity",
            "OPEN — ACTIVE SUCCESSOR",
        ):
            self.assertIn(marker, row)

        self.assertIn("CI `36003567161` is queued", row)
        self.assertIn("no ≤20 ms claim is made", row)
        self.assertNotIn("**CLOSED**", row)
        self.assertNotIn("shipped", row.lower())


if __name__ == "__main__":
    unittest.main()
