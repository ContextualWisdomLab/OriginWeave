"""Repository contract for current WebDriver BiDi standards documentation."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class WebDriverBiDiDocsCurrentnessContractTests(unittest.TestCase):
    """Keep runtime qualification and contradictory publication surfaces distinct."""

    def test_adr_tracks_merged_adapter_lineage_and_publication_surface_inconsistency(self) -> None:
        """ADR 0107 must not turn publication metadata disagreement into a runtime repin."""
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
        self.assertIn("runtime-qualified 3 September 2026", adr)
        self.assertIn("W3C publication surfaces currently disagree", adr)
        self.assertIn("Technical Report alias still presents 9 September 2026", adr)
        self.assertIn("publication history lists 16 September 2026", adr)
        self.assertIn("14 September 2026", adr)
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
        canonical_alias_uri = "https://www.w3.org/TR/webdriver-bidi/"
        alias_current_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/"
        history_newest_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260916/"
        history_previous_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260914/"
        editors_draft_uri = "https://w3c.github.io/webdriver-bidi/"

        for path, text in {
            "ARCHITECTURE.md": architecture,
            "docs/doctoring.md": doctoring,
        }.items():
            with self.subTest(path=path):
                self.assertIn(runtime_uri, text)
                self.assertNotIn(history_newest_uri, text)
                self.assertNotIn(history_previous_uri, text)

        self.assertIn("Observed: 2026-09-22", receipt)
        self.assertIn("Runtime-compatible pin: `2026-09-03`", receipt)
        self.assertIn("Canonical Technical Report alias observed: `2026-09-09`", receipt)
        self.assertIn("Publication-history newest listed Working Draft: `2026-09-16`", receipt)
        self.assertIn("Publication-history next listed Working Draft: `2026-09-14`", receipt)
        self.assertIn("Publication state: `PRIMARY-SURFACE INCONSISTENCY — FAIL CLOSED FOR RUNTIME REPIN`", receipt)
        self.assertIn("Editor's Draft: `https://w3c.github.io/webdriver-bidi/`", receipt)
        self.assertIn(canonical_alias_uri, receipt)
        self.assertIn(alias_current_uri, receipt)
        self.assertIn(history_newest_uri, receipt)
        self.assertIn(history_previous_uri, receipt)
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
