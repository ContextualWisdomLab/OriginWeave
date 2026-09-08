"""Repository contract for the versioned WebDriver BiDi presentation adapter."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class WebDriverBiDiPresentationAdapterContractTests(unittest.TestCase):
    """Keep browser emulation authority typed, versioned, and inward-dependent."""

    def test_versioned_bidi_adapter_exists_as_its_own_bounded_context(self) -> None:
        """The adapter must not be hidden in the pure fingerprint kernel."""
        manifest = ROOT / "crates/originweave-bidi/Cargo.toml"
        source = ROOT / "crates/originweave-bidi/src/lib.rs"
        self.assertTrue(
            manifest.is_file(),
            "RED: #292 has no originweave-bidi adapter crate on this exact parent",
        )
        self.assertTrue(source.is_file())
        manifest_text = manifest.read_text(encoding="utf-8")
        self.assertIn(
            'originweave-fingerprint = { path = "../originweave-fingerprint" }',
            manifest_text,
        )

    def test_2026_09_03_bidi_capabilities_fail_closed_for_complete_profile(self) -> None:
        """Standard BiDi must not pretend to own Chromium-only presentation surfaces."""
        source = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"
        self.assertTrue(
            source.is_file(),
            "RED: #292 has no version-pinned BiDi presentation capability map",
        )
        text = source.read_text(encoding="utf-8")
        self.assertIn('"2026-09-03"', text)
        self.assertIn(
            '"https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/"',
            text,
        )
        self.assertIn("PresentationSurface::Screen", text)
        self.assertIn("PresentationSurface::Viewport", text)
        self.assertIn("PresentationSurface::DevicePixelRatio", text)
        self.assertIn("PresentationSurface::TimeZone", text)
        self.assertIn("PresentationSurface::Languages", text)
        self.assertIn("PresentationSurface::ReducedMotion", text)
        self.assertIn("PresentationSurface::HardwareConcurrency", text)
        self.assertIn("PresentationError::MissingSurface", text)
        self.assertIn("WebDriverBidiBrowsingContext", text)
        self.assertIn("plan_standard_presentation_commands", text)
        self.assertIn("plan_standard_presentation_cleanup", text)
        self.assertIn("SetViewport", text)
        self.assertIn("ResetViewport", text)
        self.assertIn("SetTimezone", text)
        self.assertIn("SetReducedMotion", text)


if __name__ == "__main__":
    unittest.main()
