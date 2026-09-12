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

    def test_latest_published_bidi_is_tracked_without_silently_repinning_adapter(self) -> None:
        """Publication freshness and the qualified runtime pin must remain distinct evidence."""
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

        publication_receipt = (
            ROOT / "docs/traceability/webdriver-bidi-publication-current.md"
        )
        self.assertTrue(
            publication_receipt.is_file(),
            "RED: latest WebDriver BiDi publication is not traceable beside the qualified runtime pin",
        )
        receipt = publication_receipt.read_text(encoding="utf-8")
        self.assertIn("2026-09-09", receipt)
        self.assertIn(
            "https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/",
            receipt,
        )
        self.assertIn("Runtime-compatible pin: `2026-09-03`", receipt)
        self.assertIn("Latest published Working Draft: `2026-09-09`", receipt)

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
        self.assertIn("PresentationSurface::ReducedMotion", text)
        self.assertNotIn("SetReducedMotion", text)

    def test_presentation_documentation_tracks_qualified_wd_and_cleanup_symmetry(self) -> None:
        """Architecture, changelog, and doctoring must describe the qualified pinned adapter contract."""
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

    def test_reusable_apply_and_cleanup_require_browser_session_ownership(self) -> None:
        """Reset-to-default must not erase predecessor overrides in an unowned reused context."""
        source = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"
        text = source.read_text(encoding="utf-8")

        self.assertIn("pub struct WebDriverBidiPresentationOwnership", text)
        ownership = text.split(
            "pub struct WebDriverBidiPresentationOwnership", maxsplit=1
        )[1].split("pub enum WebDriverBidiPresentationCommand", maxsplit=1)[0]
        self.assertIn("context: WebDriverBidiBrowsingContext", ownership)
        self.assertNotIn("pub context:", ownership)
        self.assertNotIn("pub fn new(", ownership)
        self.assertNotIn("pub fn from_", ownership)

        standard_apply = text.split("pub fn plan_standard_presentation_commands", maxsplit=1)[1]
        standard_apply_signature = standard_apply.split(") ->", maxsplit=1)[0]
        self.assertIn(
            "ownership: &WebDriverBidiPresentationOwnership",
            standard_apply_signature,
        )
        self.assertNotIn(
            "context: &WebDriverBidiBrowsingContext",
            standard_apply_signature,
        )

        standard_cleanup = text.split("pub fn plan_standard_presentation_cleanup", maxsplit=1)[1]
        standard_cleanup_signature = standard_cleanup.split(") ->", maxsplit=1)[0]
        self.assertIn(
            "ownership: &WebDriverBidiPresentationOwnership",
            standard_cleanup_signature,
        )
        self.assertNotIn(
            "context: &WebDriverBidiBrowsingContext",
            standard_cleanup_signature,
        )

        for variant in ["SetViewport {", "SetTimezone {", "ResetViewport {", "ResetTimezone {"]:
            body = text.split(variant, maxsplit=1)[1].split("},", maxsplit=1)[0]
            self.assertIn("ownership: WebDriverBidiPresentationOwnership", body)
            self.assertNotIn("context: WebDriverBidiBrowsingContext", body)

    def test_reusable_apply_and_cleanup_do_not_mutate_unrestorable_media_state(self) -> None:
        """A reusable default plan must not install media state that generic cleanup cannot undo."""
        source = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"
        text = source.read_text(encoding="utf-8")

        self.assertNotIn("ExclusivePresentationContext", text)
        self.assertNotIn("plan_exclusive_presentation_media_cleanup", text)
        self.assertIn("plan_standard_presentation_commands", text)
        self.assertIn("plan_standard_presentation_cleanup", text)
        self.assertIn("PresentationSurface::ReducedMotion", text)
        self.assertNotIn("SetReducedMotion", text)

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

    def test_reusable_plan_cannot_be_mistaken_for_complete_profile_application(self) -> None:
        """The reusable planner must require the explicitly admitted fields only."""

        source = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"
        text = source.read_text(encoding="utf-8")
        standard_apply = text.split("pub fn plan_standard_presentation_commands", maxsplit=1)[1]
        standard_apply = standard_apply.split(") ->", maxsplit=1)[0]

        self.assertNotIn("profile: &PresentationProfile", standard_apply)
        self.assertIn("viewport: &ViewportBounds", standard_apply)
        self.assertIn("device_pixel_ratio: DevicePixelRatio", standard_apply)
        self.assertIn("timezone: PresentationTimeZone", standard_apply)

    def test_public_command_intents_carry_validated_presentation_value_objects(self) -> None:
        """Public command construction must not reopen validation already owned by the kernel."""

        source = ROOT / "crates/originweave-bidi/src/presentation_capabilities.rs"
        text = source.read_text(encoding="utf-8")
        command_enum = text.split("pub enum WebDriverBidiPresentationCommand", maxsplit=1)[1]
        command_enum = command_enum.split(
            "pub fn plan_standard_presentation_commands", maxsplit=1
        )[0]

        self.assertIn("viewport: ViewportBounds", command_enum)
        self.assertIn("device_pixel_ratio: DevicePixelRatio", command_enum)
        self.assertIn("timezone: PresentationTimeZone", command_enum)
        self.assertNotIn("width: u32", command_enum)
        self.assertNotIn("height: u32", command_enum)
        self.assertNotIn("device_pixel_ratio: f64", command_enum)
        self.assertNotIn("timezone: String", command_enum)

    def test_top_level_docs_distinguish_planning_boundary_from_live_bidi_transport(self) -> None:
        """Active-branch planning code must not be documented as either absent or live transport."""

        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        roadmap = (ROOT / "docs/product-roadmap.md").read_text(encoding="utf-8")

        self.assertIn("`originweave-bidi` capability and command-planning boundary", readme)
        self.assertIn("live WebDriver BiDi transport remains planned", readme)
        self.assertNotIn(
            "Chromium, WebDriver BiDi, CDP, complete MCP, HTTP, proxy, WARC, and persistent provenance adapters are planned but not yet shipped",
            readme,
        )
        self.assertIn("live WebDriver BiDi transport", roadmap)
        self.assertIn("version-pinned capability and command-planning boundary", roadmap)
        self.assertNotIn("- WebDriver BiDi adapter behind a versioned interface;", roadmap)


if __name__ == "__main__":
    unittest.main()
