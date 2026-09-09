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
        self.assertIn("runtime-qualified 3 September 2026", adr)
        self.assertIn("latest published 9 September 2026", adr)

    def test_architecture_and_doctoring_separate_latest_publication_from_runtime_pin(self) -> None:
        """Top-level architecture and doctoring must state both dates without implying a repin."""
        documents = {
            "ARCHITECTURE.md": (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8"),
            "docs/doctoring.md": (ROOT / "docs/doctoring.md").read_text(
                encoding="utf-8"
            ),
        }

        for path, text in documents.items():
            with self.subTest(path=path):
                self.assertIn(
                    "docs/traceability/webdriver-bidi-publication-current.md",
                    text,
                )
                self.assertIn("runtime-qualified 3 September 2026", text)
                self.assertIn("latest published 9 September 2026", text)
                self.assertNotIn("PR #293 is a separate active, stacked", text)


if __name__ == "__main__":
    unittest.main()
