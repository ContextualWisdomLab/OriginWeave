"""Repository contract for Browser Session WebDriver BiDi publication provenance."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
ADR = ROOT / "docs/adr/0114-browser-session-disposable-context-authority.md"
TRACE = ROOT / "docs/traceability/browser-session-lifecycle-authority.md"


class BrowserSessionWebDriverBidiPublicationTraceTests(unittest.TestCase):
    """Keep dated W3C publication provenance separate from mutable editor/runtime state."""

    def test_current_and_previous_published_working_drafts_are_explicit(self) -> None:
        adr = ADR.read_text(encoding="utf-8")
        trace = TRACE.read_text(encoding="utf-8")

        for document in (adr, trace):
            self.assertIn("WD-webdriver-bidi-20260914", document)
            self.assertIn("WD-webdriver-bidi-20260909", document)
            self.assertIn("https://w3c.github.io/webdriver-bidi/", document)

    def test_current_publication_is_not_described_as_editor_draft(self) -> None:
        adr = ADR.read_text(encoding="utf-8")
        trace = TRACE.read_text(encoding="utf-8")

        self.assertIn("14 September 2026", adr)
        self.assertIn("14 September 2026", trace)
        self.assertIn("previous", adr.lower())
        self.assertIn("previous", trace.lower())
        self.assertIn("runtime", adr.lower())
        self.assertIn("runtime", trace.lower())


if __name__ == "__main__":
    unittest.main()
