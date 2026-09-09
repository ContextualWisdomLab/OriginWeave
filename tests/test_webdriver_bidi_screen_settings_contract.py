"""Repository contract for bounded standard-BiDi screen-area planning."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"
FINGERPRINT_SOURCE = ROOT / "crates/originweave-fingerprint/src/lib.rs"


class WebDriverBiDiScreenSettingsContractTests(unittest.TestCase):
    """Keep screen geometry typed without silently widening page-observable authority."""

    def test_adapter_exposes_screen_area_value_without_unowned_mutation_intent(self) -> None:
        """Geometry may be typed before Browser Session proves authority to mutate it."""
        text = SOURCE.read_text(encoding="utf-8")

        self.assertIn("ScreenMetrics", text)
        self.assertIn("WebDriverBidiScreenArea", text)
        self.assertNotIn("SetScreenArea", text)
        self.assertNotIn("ResetScreenArea", text)
        self.assertNotIn("plan_explicit_screen_area_override", text)
        self.assertNotIn("plan_explicit_screen_area_cleanup", text)

    def test_profile_derived_plan_cannot_silently_mutate_available_screen_area(self) -> None:
        """A profile-derived reusable plan must not change an unmodelled page observable."""
        source = SOURCE.read_text(encoding="utf-8")
        fingerprint = FINGERPRINT_SOURCE.read_text(encoding="utf-8")
        screen_metrics = fingerprint.split("pub struct ScreenMetrics", maxsplit=1)[1]
        screen_metrics = screen_metrics.split("impl ScreenMetrics", maxsplit=1)[0]
        planner = source.split("pub fn plan_standard_presentation_commands", maxsplit=1)[1]
        planner = planner.split("pub fn plan_standard_presentation_cleanup", maxsplit=1)[0]
        cleanup = source.split("pub fn plan_standard_presentation_cleanup", maxsplit=1)[1]
        cleanup = cleanup.split(
            "pub const WEBDRIVER_BIDI_PRESENTATION_REVISION", maxsplit=1
        )[0]

        models_available_screen_area = (
            "available_width" in screen_metrics
            and "available_height" in screen_metrics
        )
        if models_available_screen_area:
            return

        self.assertNotIn(
            "SetScreenArea",
            planner,
            "WebDriver BiDi screen settings override also changes screen.availWidth/availHeight; "
            "the reusable profile-derived plan must model those observables or keep the override "
            "behind Browser Session ownership",
        )
        self.assertNotIn(
            "ResetScreenArea",
            cleanup,
            "generic reusable cleanup must not clear a screen override that the generic plan did "
            "not own or install",
        )

    def test_screen_area_mutation_requires_browser_session_ownership(self) -> None:
        """A context identifier alone cannot authorize replacing or clearing another owner's override."""
        text = SOURCE.read_text(encoding="utf-8")

        self.assertNotIn("SetScreenArea", text)
        self.assertNotIn("ResetScreenArea", text)
        self.assertNotIn("plan_explicit_screen_area_override", text)
        self.assertNotIn("plan_explicit_screen_area_cleanup", text)

    def test_screen_surface_remains_fail_closed_until_complete_observables_are_controlled(self) -> None:
        """Screen-area representation cannot satisfy the complete page-observable Screen contract."""
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
        """The protocol value must not imply authority over an unapplied screen observable."""
        text = SOURCE.read_text(encoding="utf-8")
        screen_area = text.split("pub struct WebDriverBidiScreenArea", maxsplit=1)[1]
        screen_area = screen_area.split("pub enum WebDriverBidiPresentationCommand", maxsplit=1)[0]

        self.assertIn("width_px: u32", screen_area)
        self.assertIn("height_px: u32", screen_area)
        self.assertNotIn("color_depth", screen_area)


if __name__ == "__main__":
    unittest.main()
