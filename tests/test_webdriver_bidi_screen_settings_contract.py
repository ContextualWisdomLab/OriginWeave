"""Repository contract for reversible standard-BiDi screen settings planning."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"


class WebDriverBiDiScreenSettingsContractTests(unittest.TestCase):
    """Keep the screen presentation surface typed, scoped, and reversible."""

    def test_standard_planner_uses_screen_settings_override(self) -> None:
        """The qualified BiDi adapter must plan the standard screen-area command."""
        text = SOURCE.read_text(encoding="utf-8")

        self.assertIn("ScreenMetrics", text)
        self.assertIn("SetScreenSettings", text)
        self.assertIn("screen: ScreenMetrics", text)
        self.assertIn("profile.screen()", text)

    def test_standard_cleanup_removes_only_its_screen_override(self) -> None:
        """Reusable cleanup must use the command's nullable context-scoped reset."""
        text = SOURCE.read_text(encoding="utf-8")
        cleanup = text.split("pub fn plan_standard_presentation_cleanup", maxsplit=1)[1]
        cleanup = cleanup.split(
            "pub const WEBDRIVER_BIDI_PRESENTATION_REVISION", maxsplit=1
        )[0]

        self.assertIn("ResetScreenSettings", cleanup)
        self.assertNotIn("ResetMediaFeatures", cleanup)

    def test_screen_surface_is_admitted_by_the_standard_capability_map(self) -> None:
        """A standard command that OriginWeave can reversibly own must be advertised."""
        text = SOURCE.read_text(encoding="utf-8")
        surfaces = text.split(
            "const WEBDRIVER_BIDI_PRESENTATION_SURFACES", maxsplit=1
        )[1].split("];", maxsplit=1)[0]

        self.assertIn("PresentationSurface::Screen", surfaces)


if __name__ == "__main__":
    unittest.main()
