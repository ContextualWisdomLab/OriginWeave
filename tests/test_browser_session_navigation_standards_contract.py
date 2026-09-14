"""Repository contract for Browser Session navigation standards provenance."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
DOCTORING = ROOT / "docs/doctoring/browser-session-navigation-lifecycle.md"
CHANGELOG = ROOT / "CHANGELOG.md"

LATEST_IMMUTABLE_URI = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/"
PREVIOUS_IMMUTABLE_URI = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/"


class BrowserSessionNavigationStandardsContractTests(unittest.TestCase):
    """Keep navigation acceptance tied to reproducible immutable W3C evidence."""

    def test_navigation_doctoring_uses_direct_immutable_w3c_publication(self) -> None:
        """Mutable publication-index lag must not erase a directly retrievable dated draft."""

        doctoring = DOCTORING.read_text(encoding="utf-8")

        self.assertIn(LATEST_IMMUTABLE_URI, doctoring)
        self.assertIn(PREVIOUS_IMMUTABLE_URI, doctoring)
        self.assertNotIn(
            "9 September / 3 September publication claims are not reproduced",
            doctoring,
        )

    def test_changelog_does_not_demote_retrievable_dated_publications(self) -> None:
        """Release notes must not call an authoritative dated W3C URI unreproducible."""

        changelog = CHANGELOG.read_text(encoding="utf-8")

        self.assertIn(LATEST_IMMUTABLE_URI, changelog)
        self.assertNotIn(
            "9 September / 3 September publication/runtime claims are not independently asserted",
            changelog,
        )
        self.assertNotIn(
            "does not reproduce a 3 September dated WebDriver BiDi Technical Report",
            changelog,
        )


if __name__ == "__main__":
    unittest.main()
