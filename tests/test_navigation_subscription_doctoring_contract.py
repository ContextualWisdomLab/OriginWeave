"""Regression contracts for committed-navigation subscription doctoring."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-network/src/webdriver_bidi_navigation_committed_subscription.rs"
DOCTORING = ROOT / "docs/doctoring.md"
ADR = ROOT / "docs/adr/0103-semantic-observation-and-stale-node-identity.md"


class NavigationSubscriptionDoctoringContractTests(unittest.TestCase):
    """Keep durable browser-protocol evidence aligned with the implemented correlation boundary."""

    def test_doctoring_matches_provably_local_subscription_failure_retirement(self) -> None:
        """Doctoring must not retain correlation for a no-write failure that source can prove locally."""
        source = SOURCE.read_text(encoding="utf-8")
        doctoring = DOCTORING.read_text(encoding="utf-8")

        self.assertLess(
            source.index("validate_frame_timeout(frame_timeout)"),
            source.index(".register_command_for("),
        )
        self.assertIn("WebDriverBiDiWebSocketFrameError::MalformedFrame", source)
        self.assertIn("correlation.retire_command_for(", source)

        self.assertNotIn(
            "even known frame-owner preflight failures remain conservatively outstanding",
            doctoring,
        )
        self.assertNotIn(
            "including a rejected deadline",
            doctoring,
        )
        self.assertIn(
            "Provably local no-write failures retire only the exact typed subscription correlation",
            doctoring,
        )
        self.assertIn(
            "ambiguous frame-write failures retain correlation",
            doctoring,
        )

    def test_webdriver_bidi_reference_tracks_current_published_working_draft(self) -> None:
        """ADR and aggregate doctoring must cite the same current published WebDriver BiDi draft."""
        adr = ADR.read_text(encoding="utf-8")
        doctoring = DOCTORING.read_text(encoding="utf-8")
        current_url = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/"

        self.assertIn("September 3, 2026 Working Draft", adr)
        self.assertIn(current_url, adr)
        self.assertIn("3 September 2026 W3C Working Draft", doctoring)
        self.assertIn(current_url, doctoring)
        self.assertNotIn(
            "The 1 June 2026 W3C Working Draft remains the most recent dated published Working Draft",
            doctoring,
        )


if __name__ == "__main__":
    unittest.main()
