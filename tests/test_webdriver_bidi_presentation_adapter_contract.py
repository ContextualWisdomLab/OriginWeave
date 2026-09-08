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
            "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/",
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
        self.assertIn("ResetTimezone", text)
        self.assertIn("SetReducedMotion", text)

    def test_presentation_documentation_tracks_published_wd_and_cleanup_symmetry(self) -> None:
        """Architecture, changelog, and doctoring must describe the same pinned adapter contract."""
        documents = {
            "ARCHITECTURE.md": (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8"),
            "CHANGELOG.md": (ROOT / "CHANGELOG.md").read_text(encoding="utf-8"),
            "docs/doctoring.md": (ROOT / "docs/doctoring.md").read_text(encoding="utf-8"),
        }
        dated_uri = "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/"
        stale_publication = "18 August 2026 published W3C Working Draft"
        for path, text in documents.items():
            with self.subTest(path=path):
                self.assertNotIn(stale_publication, text)
                self.assertIn(dated_uri, text)
                self.assertIn("timezone", text.lower())
                self.assertIn("media", text.lower())
                self.assertIn("cleanup", text.lower())

    def test_reusable_apply_and_cleanup_do_not_mutate_unrestorable_media_state(self) -> None:
        """A reusable default plan must not install media state that generic cleanup cannot undo."""
        source = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"
        text = source.read_text(encoding="utf-8")

        self.assertNotIn("ExclusivePresentationContext", text)
        self.assertNotIn("plan_exclusive_presentation_media_cleanup", text)
        self.assertIn("plan_standard_presentation_commands", text)
        self.assertIn("plan_standard_presentation_cleanup", text)
        self.assertIn("SetReducedMotion", text)

        standard_apply = text.split("pub fn plan_standard_presentation_commands", maxsplit=1)[1]
        standard_apply = standard_apply.split(
            "pub fn plan_standard_presentation_cleanup", maxsplit=1
        )[0]
        self.assertNotIn("SetReducedMotion", standard_apply)

        standard_cleanup = text.split("pub fn plan_standard_presentation_cleanup", maxsplit=1)[1]
        standard_cleanup = standard_cleanup.split(
            "pub const WEBDRIVER_BIDI_PRESENTATION_REVISION", maxsplit=1
        )[0]
        self.assertNotIn("ResetMediaFeatures", standard_cleanup)


if __name__ == "__main__":
    unittest.main()
