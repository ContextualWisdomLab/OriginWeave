use std::{error::Error, fmt};

use originweave_fingerprint::{
    DevicePixelRatio, PresentationError, PresentationSurface, PresentationTimeZone, ScreenMetrics,
    ViewportBounds, require_presentation_surfaces,
};

const MAX_BROWSING_CONTEXT_BYTES: usize = 256;

/// Failure to construct a bounded typed WebDriver BiDi command input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBidiCommandError {
    /// The remote-provided browsing-context identifier is empty, oversized, or contains control text.
    InvalidBrowsingContext,
}

impl fmt::Display for WebDriverBidiCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid WebDriver BiDi browsing context")
    }
}

impl Error for WebDriverBidiCommandError {}

/// One bounded opaque browsing-context identifier issued by the WebDriver BiDi remote end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBidiBrowsingContext(String);

impl WebDriverBidiBrowsingContext {
    /// Validate an opaque identifier without interpreting it as page or model authority.
    pub fn new(value: &str) -> Result<Self, WebDriverBidiCommandError> {
        if value.is_empty()
            || value.len() > MAX_BROWSING_CONTEXT_BYTES
            || value.chars().any(char::is_control)
        {
            return Err(WebDriverBidiCommandError::InvalidBrowsingContext);
        }
        Ok(Self(value.to_owned()))
    }

    /// Return the validated opaque identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Coupled total-and-available screen-area fields representable by
/// `emulation.setScreenSettingsOverride`.
///
/// WebDriver BiDi applies one rectangle to both the web-exposed total screen area and available
/// screen area. This value deliberately represents geometry only: a browsing-context identifier does
/// not prove that OriginWeave owns the existing override and therefore cannot authorize replacing or
/// clearing it. A Browser Session owner must establish an exclusive/disposable context or equivalent
/// ownership witness before a transport adapter may materialize the mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebDriverBidiScreenArea {
    width_px: u32,
    height_px: u32,
}

impl WebDriverBidiScreenArea {
    /// Project the protocol-owned rectangle from validated presentation screen metrics.
    ///
    /// The returned value intentionally means that total and available screen areas would be coupled
    /// to the same rectangle if an authorized Browser Session later applies it. Constructing this
    /// value grants no mutation or cleanup authority and does not claim that the presentation profile
    /// models `screen.availWidth`, `screen.availHeight`, or screen color depth.
    #[must_use]
    pub const fn from_screen(screen: &ScreenMetrics) -> Self {
        Self {
            width_px: screen.width(),
            height_px: screen.height(),
        }
    }

    /// Return the width represented for both total and available web-exposed screen areas.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width_px
    }

    /// Return the height represented for both total and available web-exposed screen areas.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height_px
    }
}

/// Typed standard-BiDi presentation command intent for one explicit browsing context.
///
/// These values are inputs to a later transport owner. Constructing them does not send a command,
/// prove an acknowledgement, establish Browser Session ownership, or establish page-observed state.
/// Presentation payloads retain validated value objects so a transport adapter cannot reopen raw
/// viewport, DPR, or time-zone validation. Screen-area mutation is intentionally absent: the standard
/// operation replaces or removes context state, while this adapter has no ownership or snapshot
/// witness proving that such state belongs to OriginWeave. This reusable-boundary enum deliberately
/// exposes no media-feature mutation command for the same non-destructive-cleanup reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebDriverBidiPresentationCommand {
    /// Set viewport dimensions and device-pixel ratio together.
    SetViewport {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
        /// Validated viewport bounds from the presentation-identity kernel.
        viewport: ViewportBounds,
        /// Validated quantized device-pixel ratio from the presentation-identity kernel.
        device_pixel_ratio: DevicePixelRatio,
    },
    /// Set the named time zone.
    SetTimezone {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
        /// Validated presentation time-zone identity.
        timezone: PresentationTimeZone,
    },
    /// Restore the implementation-defined viewport and remove the device-pixel-ratio override.
    ResetViewport {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
    },
    /// Remove the time-zone override.
    ResetTimezone {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
    },
}

/// Plan the reversible standard-BiDi presentation commands safe for a reusable browsing context.
///
/// Viewport/device-pixel-ratio and time-zone state each have a non-destructive nullable reset in the
/// pinned Working Draft. Screen-area mutation is excluded even as an explicit context-only command:
/// setting a rectangle can replace another owner's override and `screenArea: null` removes the current
/// override rather than restoring a prior value. Reduced motion remains an expressible protocol
/// capability, but this reusable planning boundary neither installs nor exposes a media-mutation
/// command because `features: null` clears the complete media-feature configuration rather than
/// restoring only OriginWeave's prior `prefers-reduced-motion` value. The explicit arguments make this
/// a partial-plan API: it cannot be mistaken for application of a complete
/// [`originweave_fingerprint::PresentationProfile`].
#[must_use]
pub fn plan_standard_presentation_commands(
    context: &WebDriverBidiBrowsingContext,
    viewport: &ViewportBounds,
    device_pixel_ratio: DevicePixelRatio,
    timezone: PresentationTimeZone,
) -> [WebDriverBidiPresentationCommand; 2] {
    [
        WebDriverBidiPresentationCommand::SetViewport {
            context: context.clone(),
            viewport: *viewport,
            device_pixel_ratio,
        },
        WebDriverBidiPresentationCommand::SetTimezone {
            context: context.clone(),
            timezone,
        },
    ]
}

/// Plan cleanup that is non-destructive to unrelated presentation or media overrides.
///
/// The pinned Working Draft provides independently nullable context-scoped reset paths for viewport/DPR
/// and time-zone state, so these two resets are safe to plan for a reusable browsing context. Screen-area
/// cleanup is absent because this boundary cannot prove ownership of the current screen override or
/// restore a predecessor value. Media cleanup is absent because `features: null` clears the complete
/// media-feature override configuration rather than selectively undoing `prefers-reduced-motion`.
#[must_use]
pub fn plan_standard_presentation_cleanup(
    context: &WebDriverBidiBrowsingContext,
) -> [WebDriverBidiPresentationCommand; 2] {
    [
        WebDriverBidiPresentationCommand::ResetViewport {
            context: context.clone(),
        },
        WebDriverBidiPresentationCommand::ResetTimezone {
            context: context.clone(),
        },
    ]
}

/// Published WebDriver BiDi Working Draft revision used by this capability map.
/// The immutable dated-TR identity is
/// `https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/`.
pub const WEBDRIVER_BIDI_PRESENTATION_REVISION: &str = "2026-09-03";

/// Auxiliary upstream source commit retained as historical doctoring evidence.
///
/// The dated W3C Working Draft remains the publication identity. This older commit records
/// supporting `w3c/webdriver-bidi` history for media-feature semantics; it is not treated as a
/// same-day source snapshot or a second protocol version.
pub const WEBDRIVER_BIDI_PRESENTATION_DOCTORING_SOURCE_COMMIT: &str =
    "1e5e36c43adbe24f2a4052c2ec091635c006c352";

const WEBDRIVER_BIDI_PRESENTATION_SURFACES: [PresentationSurface; 4] = [
    PresentationSurface::Viewport,
    PresentationSurface::DevicePixelRatio,
    PresentationSurface::TimeZone,
    PresentationSurface::ReducedMotion,
];

/// Return complete presentation surfaces expressible through the pinned standard BiDi contract.
///
/// The protocol can explicitly couple total and available screen width/height through
/// `emulation.setScreenSettingsOverride`, but OriginWeave's `Screen` surface also includes color depth
/// and the current profile does not model the available screen rectangle. `Screen` therefore remains
/// intentionally absent. This adapter additionally withholds screen-area mutation until Browser
/// Session proves ownership of the affected override lifecycle. Ordered-language surfaces, hardware
/// concurrency, and the Chromium platform/User-Agent Client Hints surface are also absent. Reduced
/// motion is listed as protocol capability even though reusable application leaves media state
/// untouched until a Browser Session owner supplies a restorable lifecycle and corresponding command
/// authority.
#[must_use]
pub const fn webdriver_bidi_presentation_surfaces() -> &'static [PresentationSurface] {
    &WEBDRIVER_BIDI_PRESENTATION_SURFACES
}

/// Require the pinned standard BiDi capability set to satisfy the complete profile.
///
/// The current result remains fail-closed with
/// `PresentationError::MissingSurface(PresentationSurface::Screen)` because the standard screen-area
/// value does not control color depth, the profile does not model available-screen geometry, and this
/// adapter has no Browser Session ownership witness for mutating existing screen-settings state.
/// Callers must not translate that result into ambient-host fallback.
pub fn require_complete_presentation_profile() -> Result<(), PresentationError> {
    require_presentation_surfaces(webdriver_bidi_presentation_surfaces())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use originweave_fingerprint::{
        DevicePixelRatio, PresentationPlatform, PresentationProfile, PresentationTimeZone,
        ScreenMetrics, ViewportBounds,
    };

    #[test]
    fn pinned_revision_tracks_current_published_working_draft() {
        assert_eq!(WEBDRIVER_BIDI_PRESENTATION_REVISION, "2026-09-03");
        assert_eq!(
            WEBDRIVER_BIDI_PRESENTATION_DOCTORING_SOURCE_COMMIT,
            "1e5e36c43adbe24f2a4052c2ec091635c006c352"
        );
    }

    #[test]
    fn standard_bidi_claims_only_complete_canonical_surfaces() {
        let surfaces = webdriver_bidi_presentation_surfaces();

        assert_eq!(
            require_complete_presentation_profile(),
            Err(PresentationError::MissingSurface(
                PresentationSurface::Screen
            ))
        );
        assert!(!surfaces.contains(&PresentationSurface::Screen));
        assert!(surfaces.contains(&PresentationSurface::Viewport));
        assert!(surfaces.contains(&PresentationSurface::DevicePixelRatio));
        assert!(!surfaces.contains(&PresentationSurface::HardwareConcurrency));
        assert!(surfaces.contains(&PresentationSurface::TimeZone));
        assert!(!surfaces.contains(&PresentationSurface::Platform));
        assert!(!surfaces.contains(&PresentationSurface::Languages));
        assert!(surfaces.contains(&PresentationSurface::ReducedMotion));
    }

    #[test]
    fn screen_area_value_preserves_protocol_coupling_without_mutation_authority() {
        let profile = PresentationProfile::new(
            ScreenMetrics::new(1920, 1080).expect("valid screen"),
            ViewportBounds::new(1440, 900).expect("valid viewport"),
            DevicePixelRatio::Quantized2,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::MacOS,
            vec!["en-US".to_owned()],
            true,
        )
        .expect("consistent profile");
        let screen_area = WebDriverBidiScreenArea::from_screen(profile.screen());

        assert_eq!(screen_area.width(), 1920);
        assert_eq!(screen_area.height(), 1080);
    }

    #[test]
    fn reusable_standard_commands_bind_only_modelled_symmetrically_restorable_state() {
        let error = WebDriverBidiCommandError::InvalidBrowsingContext;
        assert_eq!(error.to_string(), "invalid WebDriver BiDi browsing context");
        assert!(Error::source(&error).is_none());
        for invalid in ["", "context\n17"] {
            assert_eq!(
                WebDriverBidiBrowsingContext::new(invalid),
                Err(WebDriverBidiCommandError::InvalidBrowsingContext)
            );
        }
        assert_eq!(
            WebDriverBidiBrowsingContext::new(&"x".repeat(257)),
            Err(WebDriverBidiCommandError::InvalidBrowsingContext)
        );
        let profile = PresentationProfile::new(
            ScreenMetrics::new(1920, 1080).expect("valid screen"),
            ViewportBounds::new(1440, 900).expect("valid viewport"),
            DevicePixelRatio::Quantized2,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::MacOS,
            vec!["en-US".to_owned()],
            true,
        )
        .expect("consistent profile");
        let context =
            WebDriverBidiBrowsingContext::new("context-17").expect("bounded context identifier");
        assert_eq!(context.as_str(), "context-17");

        assert_eq!(
            plan_standard_presentation_commands(
                &context,
                profile.viewport(),
                profile.device_pixel_ratio(),
                profile.timezone(),
            ),
            [
                WebDriverBidiPresentationCommand::SetViewport {
                    context: context.clone(),
                    viewport: *profile.viewport(),
                    device_pixel_ratio: profile.device_pixel_ratio(),
                },
                WebDriverBidiPresentationCommand::SetTimezone {
                    context,
                    timezone: profile.timezone(),
                },
            ]
        );
    }

    #[test]
    fn reusable_cleanup_does_not_clear_unrelated_screen_or_media_state() {
        let context =
            WebDriverBidiBrowsingContext::new("context-17").expect("bounded context identifier");

        assert_eq!(
            plan_standard_presentation_cleanup(&context),
            [
                WebDriverBidiPresentationCommand::ResetViewport {
                    context: context.clone(),
                },
                WebDriverBidiPresentationCommand::ResetTimezone { context },
            ]
        );
    }
}
