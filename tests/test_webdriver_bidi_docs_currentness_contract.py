"""Repository contract for current WebDriver BiDi standards documentation."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class WebDriverBiDiDocsCurrentnessContractTests(unittest.TestCase):
    """Keep runtime qualification and publication freshness as separate authorities."""

    def test_adr_tracks_merged_adapter_lineage_and_live_publication_history(self) -> None:
        """ADR 0107 must follow live W3C publication history without silently repinning runtime."""
        adr = (ROOT / "docs/adr/0107-browser-protocol-adapter-strategy.md").read_text(
            encoding="utf-8"
        )

        runtime_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/"
        latest_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260916/"
        previous_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260914/"

        self.assertNotIn(
            "PR #293 is a separate active, stacked browser-adapter slice",
            adr,
        )
        self.assertIn("PR #293 was merged into PR #229", adr)
        self.assertIn(
            "docs/traceability/webdriver-bidi-publication-current.md",
            adr,
        )
        self.assertIn("runtime-qualified 3 September 2026", adr)
        self.assertIn("fresh live W3C publication-history read on 2026-09-22", adr)
        self.assertIn(runtime_uri, adr)
        self.assertIn(latest_uri, adr)
        self.assertIn(previous_uri, adr)
        self.assertNotIn(
            "16 September / 14 September claim is invalid because those entries do not exist",
            adr,
        )

    def test_publication_freshness_is_single_sourced_from_runtime_qualification_docs(self) -> None:
        """Architecture and doctoring stay qualification records; the receipt owns publication churn."""
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        doctoring = (ROOT / "docs/doctoring.md").read_text(encoding="utf-8")
        receipt = (
            ROOT / "docs/traceability/webdriver-bidi-publication-current.md"
        ).read_text(encoding="utf-8")

        runtime_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/"
        latest_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260916/"
        previous_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260914/"
        prior_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/"
        history_uri = "https://www.w3.org/standards/history/webdriver-bidi/"
        editors_draft_uri = "https://w3c.github.io/webdriver-bidi/"

        for path, text in {
            "ARCHITECTURE.md": architecture,
            "docs/doctoring.md": doctoring,
        }.items():
            with self.subTest(path=path):
                self.assertIn(runtime_uri, text)
                self.assertNotIn(latest_uri, text)
                self.assertNotIn(previous_uri, text)

        self.assertIn("Observed: 2026-09-22", receipt)
        self.assertIn("Runtime-compatible pin: `2026-09-03`", receipt)
        self.assertIn("Latest published Working Draft: `2026-09-16`", receipt)
        self.assertIn("Previous published Working Draft: `2026-09-14`", receipt)
        self.assertIn("Earlier published Working Draft: `2026-09-09`", receipt)
        self.assertIn("Editor's Draft: `https://w3c.github.io/webdriver-bidi/`", receipt)
        self.assertIn(latest_uri, receipt)
        self.assertIn(previous_uri, receipt)
        self.assertIn(prior_uri, receipt)
        self.assertIn(history_uri, receipt)
        self.assertIn(editors_draft_uri, receipt)
        self.assertIn(
            "PR #229, which has inherited merged PR #293",
            receipt,
        )
        self.assertNotIn(
            "The canonical W3C publication history contains neither entry",
            receipt,
        )


if __name__ == "__main__":
    unittest.main()
