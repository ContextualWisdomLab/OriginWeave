"""Regression contracts for release-manifest documentation truth and source citations."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
ADR_PATH = ROOT / "docs/adr/0015-release-manifest-identity.md"
DOCTORING_PATH = ROOT / "docs/doctoring.md"


class ReleaseManifestDocumentationContractTests(unittest.TestCase):
    """Keep release-manifest portability and Git-source claims narrower than the code proves."""

    def test_sync_compatibility_claim_matches_the_actual_filename_validator(self) -> None:
        """Reserved-device guards must not be presented as full OneDrive/SharePoint compatibility."""
        adr = ADR_PATH.read_text(encoding="utf-8")
        doctoring = DOCTORING_PATH.read_text(encoding="utf-8")

        for text in (adr, doctoring):
            with self.subTest(document=text[:40]):
                self.assertIn("desktop.ini", text)
                self.assertIn(
                    "not a complete OneDrive or SharePoint synchronization-compatibility guarantee",
                    text,
                )

        self.assertNotIn("common buyer synchronization paths", adr)
        self.assertNotIn("does not become a device or synchronization conflict", adr)

    def test_git_protocol_references_pin_the_verified_manual_revisions(self) -> None:
        """Protocol evidence must resolve to the manual revisions that contain the cited rules."""
        for path in (ADR_PATH, DOCTORING_PATH):
            text = path.read_text(encoding="utf-8")
            with self.subTest(path=path):
                self.assertIn("gitprotocol-common/2.50.0", text)
                self.assertIn("gitprotocol-pack/2.54.0", text)
                self.assertNotIn("gitprotocol-common/2.55.0", text)
                self.assertNotIn("gitprotocol-pack/2.55.0", text)


if __name__ == "__main__":
    unittest.main()
