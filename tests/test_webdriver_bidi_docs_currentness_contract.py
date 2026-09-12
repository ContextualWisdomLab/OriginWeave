"""Repository contract for current WebDriver BiDi standards documentation."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class WebDriverBiDiDocsCurrentnessContractTests(unittest.TestCase):
    """Keep merged lineage, publication freshness, and runtime qualification distinct."""

    def test_adr_tracks_merged_adapter_lineage_and_publication_receipt(self) -> None:
        """ADR 0107 must not describe merged PR #293 as an active stacked slice."""
        adr = (ROOT / "docs/adr/0107-browser-protocol-adapter-strategy.md").read_text(
            encoding="utf-8"
        )

        self.assertNotIn(
            "PR #293 is a separate active, stacked browser-adapter slice",
            adr,
        )
        self.assertIn("PR #293 was merged into PR #229", adr)
        self.assertIn(
            "docs/traceability/webdriver-bidi-publication-current.md",
            adr,
        )
        self.assertIn("runtime-qualified 18 August 2026", adr)
        self.assertIn("latest published 18 August 2026", adr)

    def test_publication_freshness_is_single_sourced_from_runtime_qualification_docs(self) -> None:
        """Architecture and doctoring stay qualification records; the receipt owns latest-publication churn."""
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        doctoring = (ROOT / "docs/doctoring.md").read_text(encoding="utf-8")
        receipt = (
            ROOT / "docs/traceability/webdriver-bidi-publication-current.md"
        ).read_text(encoding="utf-8")

        runtime_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/"
        latest_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/"

        for path, text in {
            "ARCHITECTURE.md": architecture,
            "docs/doctoring.md": doctoring,
        }.items():
            with self.subTest(path=path):
                self.assertIn(runtime_uri, text)

        self.assertIn("Runtime-compatible pin: `2026-08-18`", receipt)
        self.assertIn("Latest published Working Draft: `2026-08-18`", receipt)
        self.assertIn(latest_uri, receipt)
        self.assertIn(
            "PR #229, which has inherited merged PR #293",
            receipt,
        )


if __name__ == "__main__":
    unittest.main()
