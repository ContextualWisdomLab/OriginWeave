"""Repository contract for Browser Session's single-writer WebDriver BiDi provenance."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
ADR = ROOT / "docs/adr/0114-browser-session-disposable-context-authority.md"
TRACE = ROOT / "docs/traceability/browser-session-lifecycle-authority.md"
CANONICAL_RECEIPT = "docs/traceability/webdriver-bidi-publication-current.md"
DATED_TR = re.compile(r"WD-webdriver-bidi-\d{8}")
VOLATILE_CURRENTNESS = re.compile(
    r"(?:current|latest|previous) published WebDriver BiDi Working Draft",
    re.IGNORECASE,
)


class BrowserSessionWebDriverBidiPublicationTraceTests(unittest.TestCase):
    """Keep standards freshness with the canonical originweave-bidi owner."""

    def test_browser_session_references_canonical_publication_receipt(self) -> None:
        adr = ADR.read_text(encoding="utf-8")
        trace = TRACE.read_text(encoding="utf-8")

        for document in (adr, trace):
            self.assertIn(CANONICAL_RECEIPT, document)
            self.assertIsNone(DATED_TR.search(document))
            self.assertIsNone(VOLATILE_CURRENTNESS.search(document))

    def test_browser_session_keeps_only_lifecycle_relevant_standard_semantics(self) -> None:
        adr = ADR.read_text(encoding="utf-8")
        trace = TRACE.read_text(encoding="utf-8")

        self.assertIn("browser.UserContext", adr)
        self.assertIn("browser.createUserContext", adr)
        self.assertIn("browsingContext.create", adr)
        self.assertIn("browser.removeUserContext", adr)
        self.assertIn("command ACK alone is not destruction proof", adr)
        self.assertIn("command acknowledgement is insufficient proof", trace)
        self.assertIn("runtime compatibility", adr.lower())
        self.assertIn("runtime compatibility", trace.lower())


if __name__ == "__main__":
    unittest.main()
