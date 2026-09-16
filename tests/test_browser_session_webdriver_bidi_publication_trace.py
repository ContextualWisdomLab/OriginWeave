"""Repository contract for Browser Session WebDriver BiDi publication provenance."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
ADR = ROOT / "docs/adr/0114-browser-session-disposable-context-authority.md"
TRACE = ROOT / "docs/traceability/browser-session-lifecycle-authority.md"


class BrowserSessionWebDriverBidiPublicationTraceTests(unittest.TestCase):
    """Keep dated W3C publication provenance separate from mutable editor/runtime state."""

    def test_current_previous_and_editor_provenance_are_distinct(self) -> None:
        adr = ADR.read_text(encoding="utf-8")
        trace = TRACE.read_text(encoding="utf-8")

        for document in (adr, trace):
            self.assertIn("WD-webdriver-bidi-20260914", document)
            self.assertIn("WD-webdriver-bidi-20260909", document)
            self.assertIn("https://w3c.github.io/webdriver-bidi/", document)
            self.assertRegex(
                document,
                r"14 September 2026.*current published WebDriver BiDi Working Draft",
            )
            self.assertRegex(
                document,
                r"WD-webdriver-bidi-20260909.*previous published version",
            )
            self.assertRegex(
                document,
                r"Editor(?:'s|’s) Draft.*https://w3c.github.io/webdriver-bidi/",
            )

    def test_publication_refresh_does_not_claim_runtime_repin(self) -> None:
        adr = ADR.read_text(encoding="utf-8")
        trace = TRACE.read_text(encoding="utf-8")

        self.assertIn("does not repin runtime behavior", adr)
        self.assertIn("runtime compatibility revision is a third", trace)


if __name__ == "__main__":
    unittest.main()
