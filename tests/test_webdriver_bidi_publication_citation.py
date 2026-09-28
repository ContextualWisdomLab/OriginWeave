"""Keep WebDriver BiDi citations on the current dated W3C Working Draft."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CURRENT_WORKING_DRAFT = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/"
CITATION_PATHS = (
    "docs/doctoring.md",
    "docs/doctoring/browser-agent-protocols.md",
    "docs/doctoring/product-documentation-baseline.md",
    "docs/adr/0001-chromium-compatibility-kernel.md",
    "docs/adr/0102-typed-actions-and-arbitrary-js.md",
    "docs/adr/0103-semantic-observation-and-stale-node-identity.md",
    "docs/adr/0107-browser-protocol-adapter-strategy.md",
)
STALE_MARKERS = (
    "20260601",
    "20260629",
    "1 June 2026",
    "29 June 2026",
)


class WebDriverBidiPublicationCitationTests(unittest.TestCase):
    """Doctoring and architecture records cite one current dated Working Draft."""

    def test_records_cite_the_9_september_2026_working_draft(self) -> None:
        """A superseded June draft must not remain beside the current publication."""
        for relative_path in CITATION_PATHS:
            text = (ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn(CURRENT_WORKING_DRAFT, text, relative_path)
            for marker in STALE_MARKERS:
                self.assertNotIn(marker, text, f"{relative_path} still cites {marker}")


if __name__ == "__main__":
    unittest.main()
