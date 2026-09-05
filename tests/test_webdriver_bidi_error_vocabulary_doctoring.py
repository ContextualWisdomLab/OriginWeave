"""Lock the reviewed WebDriver BiDi error-vocabulary evidence to the shipped adapter contract."""

from __future__ import annotations

import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
DOCTORING = ROOT / "docs" / "doctoring" / "browser-agent-protocols.md"
CHANGELOG = ROOT / "CHANGELOG.md"


class WebDriverBiDiErrorVocabularyDoctoringTests(unittest.TestCase):
    """Prevent the W3C prose/CDDL discrepancy from being rewritten as false conformance evidence."""

    def test_doctoring_records_the_reviewed_cddl_exception(self) -> None:
        """The adapter's 30+1 admission rule must stay explicit and fail-closed."""

        doctoring = DOCTORING.read_text(encoding="utf-8")
        self.assertIn(
            "rendered local-end CDDL enumerates 30 values and omits `no such client window`",
            doctoring,
        )
        self.assertIn(
            "§3.5 separately defines `no such client window` and normative client-window algorithms return that error code",
            doctoring,
        )
        self.assertIn(
            "explicit interoperability exception for a specification-internal inconsistency",
            doctoring,
        )

    def test_changelog_does_not_misstate_the_exception_as_cddl_membership(self) -> None:
        """Release evidence must distinguish rendered CDDL values from the normative exception."""

        changelog = CHANGELOG.read_text(encoding="utf-8")
        self.assertIn(
            "30-value rendered W3C `ErrorCode` CDDL plus the separately defined normative `no such client window` error",
            changelog,
        )
        self.assertIn("arbitrary strings still fail closed", changelog)


if __name__ == "__main__":
    unittest.main()
