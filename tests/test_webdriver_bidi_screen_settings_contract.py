"""Repository contract for reversible standard-BiDi screen-area planning."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"


class WebDriverBiDiScreenSettingsContractTests(unittest.TestCase):
    """Keep screen geometry typed and reversible without overstating color-depth control."""

    def test_standard_planner_uses_screen_settings_override(self) -> None:
        """The qualified BiDi adapter must plan the standard screen-area command."""
        text = SOURCE.read_text(encoding="utf-8")

        self.assertIn("ScreenMetrics", text)
        self.assertIn("WebDriverBidiScreenArea", text)
        self.assertIn("SetScreenArea", text)
        self.assertIn("screen_area: WebDriverBidiScreenArea", text)
        self.assertIn("screen: &ScreenMetrics", text)
        self.assertIn("profile.screen()", text)

    def test_standard_cleanup_removes_only_its_screen_area_override(self) -> None:
        """Reusable cleanup must use the command's nullable context-scoped reset."""
        text = SOURCE.read_text(encoding="utf-8")
        cleanup = text.split("pub fn plan_standard_presentation_cleanup", maxsplit=1)[1]
        cleanup = cleanup.split(
            "pub const WEBDRIVER_BIDI_PRESENTATION_REVISION", maxsplit=1
        )[0]

        self.assertIn("ResetScreenArea", cleanup)
        self.assertNotIn("ResetMediaFeatures", cleanup)

    def test_screen_surface_remains_fail_closed_until_color_depth_is_controlled(self) -> None:
        """Screen area alone cannot satisfy ScreenMetrics because color depth remains observable."""
        text = SOURCE.read_text(encoding="utf-8")
        surfaces = text.split(
            "const WEBDRIVER_BIDI_PRESENTATION_SURFACES", maxsplit=1
        )[1].split("];", maxsplit=1)[0]

        self.assertNotIn("PresentationSurface::Screen", surfaces)
        self.assertIn(
            "PresentationError::MissingSurface(PresentationSurface::Screen)",
            "".join(text.split()),
        )

    def test_screen_area_payload_does_not_carry_color_depth(self) -> None:
        """The command intent must not imply authority over an unapplied screen observable."""
        text = SOURCE.read_text(encoding="utf-8")
        screen_area = text.split("pub struct WebDriverBidiScreenArea", maxsplit=1)[1]
        screen_area = screen_area.split("pub enum WebDriverBidiPresentationCommand", maxsplit=1)[0]

        self.assertIn("width_px: u32", screen_area)
        self.assertIn("height_px: u32", screen_area)
        self.assertNotIn("color_depth", screen_area)


if __name__ == "__main__":
    unittest.main()
