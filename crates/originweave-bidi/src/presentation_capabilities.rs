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

/// Proof that Browser Session owns the presentation-override lifecycle for one browsing context.
///
/// This type intentionally has no public constructor. WebDriver BiDi nullable viewport/DPR and
/// time-zone values remove an override or restore an implementation default; they do not restore a
/// predecessor override installed by another owner. A remote-issued context identifier is therefore
/// addressability, not mutation authority. Browser Session may mint this witness only after proving an
/// exclusive/disposable context or an equivalent lifecycle that preserves predecessor state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBidiPresentationOwnership {
    context: WebDriverBidiBrowsingContext,
}

impl WebDriverBidiPresentationOwnership {
    /// Return the exact browsing context covered by this ownership witness.
    #[must_use]
    pub const fn context(&self) -> &WebDriverBidiBrowsingContext {
        &self.context
    }
}

/// Coupled total-and-available screen-area fields representable by
/// `emulation.setScreenSettingsOverride`.
///
/// WebDriver BiDi applies one rectangle to both the web-exposed total screen area and available
/// screen area. Construction therefore remains an explicit partial capability: it projects width and
/// height from validated [`ScreenMetrics`] but does not claim that the presentation profile models the
/// resulting `screen.availWidth` / `screen.availHeight` observables or screen color depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebDriverBidiScreenArea {
    width_px: u32,
    height_px: u32,
}

impl WebDriverBidiScreenArea {
    /// Project the protocol-owned rectangle from validated presentation screen metrics.
    ///
    /// The returned value intentionally means that total and available screen areas will be coupled to
    /// the same rectangle. It must not be inserted into a profile-derived reusable plan unless the
    /// presentation schema has first modelled and authorized those available-area observables.
    #[must_use]
    pub const fn from_screen(screen: &ScreenMetrics) -> Self {
        Self {
            width_px: screen.width(),
            height_px: screen.height(),
        }
    }

    /// Return the width applied to both total and available web-exposed screen areas.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width_px
    }

    /// Return the height applied to both total and available web-exposed screen areas.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height_px
    }
}

/// Proof that Browser Session owns screen-settings mutation for one browsing context.
///
/// This type intentionally has no public constructor. A remote-issued context identifier is identity,
/// not authority: WebDriver BiDi replaces the current screen-area override when setting a rectangle and
/// removes it when `screenArea` is null. A Browser Session integration may create this witness only
/// after it has established an exclusive/disposable context or an equivalent lifecycle that proves no
/// unrelated owner state can be overwritten or cleared. Until that integration exists, external
/// callers have neither a mint path nor a callable screen-area planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBidiScreenAreaOwnership {
    context: WebDriverBidiBrowsingContext,
}

impl WebDriverBidiScreenAreaOwnership {
    /// Return the exact browsing context covered by this ownership witness.
    #[must_use]
    pub const fn context(&self) -> &WebDriverBidiBrowsingContext {
        &self.context
    }
}

/// Typed standard-BiDi presentation command intent for one explicitly owned browsing context.
///
/// These values are inputs to a later transport owner. Constructing them does not send a command,
/// prove an acknowledgement, establish Browser Session ownership, or establish page-observed state.
/// Presentation payloads retain validated value objects so a transport adapter cannot reopen raw
/// screen, viewport, DPR, or time-zone validation. Viewport/DPR and time-zone intents retain an opaque
/// Browser Session ownership witness because nullable reset clears predecessor overrides rather than
/// restoring them. Screen-area command vocabulary keeps its narrower witness because setting or
/// clearing that override also mutates unmodelled available-screen state. No media-feature mutation is
/// exposed because this crate has no predecessor snapshot or ownership contract for that state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebDriverBidiPresentationCommand {
    /// Set total and available web-exposed screen width and height together.
    SetScreenArea {
        /// Browser Session proof that this context's screen-settings lifecycle is exclusively owned.
        ownership: WebDriverBidiScreenAreaOwnership,
        /// Exact coupled standard-BiDi screen-area payload derived from validated screen metrics.
        screen_area: WebDriverBidiScreenArea,
    },
    /// Set viewport dimensions and device-pixel ratio together.
    SetViewport {
        /// Browser Session proof that replacing viewport/DPR state cannot destroy another owner's state.
        ownership: WebDriverBidiPresentationOwnership,
        /// Validated viewport bounds from the presentation-identity kernel.
        viewport: ViewportBounds,
        /// Validated quantized device-pixel ratio from the presentation-identity kernel.
        device_pixel_ratio: DevicePixelRatio,
    },
    /// Set the named time zone.
    SetTimezone {
        /// Browser Session proof that replacing time-zone state cannot destroy another owner's state.
        ownership: WebDriverBidiPresentationOwnership,
        /// Validated presentation time-zone identity.
        timezone: PresentationTimeZone,
    },
    /// Remove the coupled total-and-available screen-area override for the owned browsing context.
    ResetScreenArea {
        /// Browser Session proof that clearing this context cannot remove another owner's override.
        ownership: WebDriverBidiScreenAreaOwnership,
    },
    /// Remove owned viewport and device-pixel-ratio overrides.
    ResetViewport {
        /// Browser Session proof that default-reset is valid for this owned lifecycle.
        ownership: WebDriverBidiPresentationOwnership,
    },
    /// Remove the owned time-zone override.
    ResetTimezone {
        /// Browser Session proof that default-reset is valid for this owned lifecycle.
        ownership: WebDriverBidiPresentationOwnership,
    },
}

/// Plan standard-BiDi presentation commands only for a Browser Session-owned lifecycle.
///
/// The pinned Working Draft can set viewport/device-pixel-ratio and time-zone state, but its nullable
/// reset semantics do not restore a predecessor override. The ownership witness therefore replaces the
/// former raw browsing-context argument: callers that can merely name a reused context cannot overwrite
/// another owner's state and later clear it to an implementation default. Screen settings remain outside
/// this profile-derived plan because they additionally change unmodelled available-screen geometry.
/// Reduced motion remains an expressible protocol capability, but this boundary installs no media state
/// because it lacks a restorable predecessor contract. The explicit values keep this a partial-plan API
/// rather than complete [`originweave_fingerprint::PresentationProfile`] application.
#[must_use]
pub fn plan_standard_presentation_commands(
    ownership: &WebDriverBidiPresentationOwnership,
    viewport: &ViewportBounds,
    device_pixel_ratio: DevicePixelRatio,
    timezone: PresentationTimeZone,
) -> [WebDriverBidiPresentationCommand; 2] {
    [
        WebDriverBidiPresentationCommand::SetViewport {
            ownership: ownership.clone(),
            viewport: *viewport,
            device_pixel_ratio,
        },
        WebDriverBidiPresentationCommand::SetTimezone {
            ownership: ownership.clone(),
            timezone,
        },
    ]
}

/// Plan default-reset cleanup only for the same Browser Session-owned lifecycle.
///
/// A reset removes OriginWeave-owned viewport/DPR and time-zone overrides only when Browser Session has
/// already proved that no unrelated predecessor state can be lost. This function therefore accepts the
/// non-caller-mintable ownership witness, not a raw context identifier. Screen-area cleanup remains
/// separately ownership-gated and has no callable planner while its lifecycle mint path is absent.
/// Media cleanup is absent because `features: null` clears the complete media-feature configuration
/// rather than selectively restoring OriginWeave's prior `prefers-reduced-motion` value.
#[must_use]
pub fn plan_standard_presentation_cleanup(
    ownership: &WebDriverBidiPresentationOwnership,
) -> [WebDriverBidiPresentationCommand; 2] {
    [
        WebDriverBidiPresentationCommand::ResetViewport {
            ownership: ownership.clone(),
        },
        WebDriverBidiPresentationCommand::ResetTimezone {
            ownership: ownership.clone(),
        },
    ]
}

/// Published WebDriver BiDi Working Draft revision used by this capability map.
/// The immutable dated-TR identity is
/// `https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/`.
pub const WEBDRIVER_BIDI_PRESENTATION_REVISION: &str = "2026-08-18";

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
/// intentionally absent. Ordered-language surfaces, hardware concurrency, and the Chromium
/// platform/User-Agent Client Hints surface are also absent. Reduced motion is listed as protocol
/// capability even though application leaves media state untouched until a Browser Session owner
/// supplies a restorable lifecycle and corresponding command authority.
#[must_use]
pub const fn webdriver_bidi_presentation_surfaces() -> &'static [PresentationSurface] {
    &WEBDRIVER_BIDI_PRESENTATION_SURFACES
}

/// Require the pinned standard BiDi capability set to satisfy the complete profile.
///
/// The current result remains fail-closed with
/// `PresentationError::MissingSurface(PresentationSurface::Screen)` because the dormant screen-area
/// command does not control color depth, additionally couples an available-screen observable absent
/// from the current profile, and cannot be materialized until Browser Session supplies ownership of the
/// screen-settings lifecycle. Callers must not translate that result into ambient-host fallback.
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
        assert_eq!(WEBDRIVER_BIDI_PRESENTATION_REVISION, "2026-08-18");
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
    fn screen_area_command_shape_requires_the_same_ownership_witness() {
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
        let ownership = WebDriverBidiScreenAreaOwnership {
            context: context.clone(),
        };
        let screen_area = WebDriverBidiScreenArea::from_screen(profile.screen());
        let set_command = WebDriverBidiPresentationCommand::SetScreenArea {
            ownership: ownership.clone(),
            screen_area,
        };
        let reset_command = WebDriverBidiPresentationCommand::ResetScreenArea {
            ownership: ownership.clone(),
        };

        assert_eq!(ownership.context(), &context);
        assert_eq!(screen_area.width(), 1920);
        assert_eq!(screen_area.height(), 1080);
        assert_eq!(
            set_command,
            WebDriverBidiPresentationCommand::SetScreenArea {
                ownership: ownership.clone(),
                screen_area,
            }
        );
        assert_eq!(
            reset_command,
            WebDriverBidiPresentationCommand::ResetScreenArea { ownership }
        );
    }

    #[test]
    fn standard_commands_require_the_same_presentation_ownership_witness() {
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
        let ownership = WebDriverBidiPresentationOwnership {
            context: context.clone(),
        };
        assert_eq!(context.as_str(), "context-17");
        assert_eq!(ownership.context(), &context);

        assert_eq!(
            plan_standard_presentation_commands(
                &ownership,
                profile.viewport(),
                profile.device_pixel_ratio(),
                profile.timezone(),
            ),
            [
                WebDriverBidiPresentationCommand::SetViewport {
                    ownership: ownership.clone(),
                    viewport: *profile.viewport(),
                    device_pixel_ratio: profile.device_pixel_ratio(),
                },
                WebDriverBidiPresentationCommand::SetTimezone {
                    ownership: ownership.clone(),
                    timezone: profile.timezone(),
                },
            ]
        );
    }

    #[test]
    fn standard_cleanup_requires_owned_lifecycle_before_default_reset() {
        let context =
            WebDriverBidiBrowsingContext::new("context-17").expect("bounded context identifier");
        let ownership = WebDriverBidiPresentationOwnership { context };

        assert_eq!(
            plan_standard_presentation_cleanup(&ownership),
            [
                WebDriverBidiPresentationCommand::ResetViewport {
                    ownership: ownership.clone(),
                },
                WebDriverBidiPresentationCommand::ResetTimezone {
                    ownership: ownership.clone(),
                },
            ]
        );
    }
}
