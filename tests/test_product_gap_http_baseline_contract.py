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
            "0c256a9739222fceea000f75193cdda8270e2a33",
            "24 ahead / 0 behind",
            "p95 **20,000 µs**",
            "not an accepted performance receipt",
            "identified-environment performance receipt",
            "OPEN — ACTIVE SUCCESSOR",
        ):
            self.assertIn(marker, row)

        self.assertIn("CI `35936872235` is still pre-step queued", row)
        self.assertIn("no ≤20 ms claim is made", row)
        self.assertNotIn("**CLOSED**", row)
        self.assertNotIn("shipped", row.lower())


if __name__ == "__main__":
    unittest.main()
