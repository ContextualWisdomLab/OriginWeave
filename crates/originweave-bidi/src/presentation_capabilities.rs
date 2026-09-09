use std::{error::Error, fmt};

use originweave_fingerprint::{
    PresentationError, PresentationProfile, PresentationSurface, require_presentation_surfaces,
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

/// Typed standard-BiDi presentation command intent for one explicit browsing context.
///
/// These values are inputs to a later transport owner. Constructing them does not send a command,
/// prove an acknowledgement, establish Browser Session ownership, or establish page-observed state.
#[derive(Debug, Clone, PartialEq)]
pub enum WebDriverBidiPresentationCommand {
    /// Set viewport dimensions and device-pixel ratio together.
    SetViewport {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
        /// CSS-pixel viewport width.
        width: u32,
        /// CSS-pixel viewport height.
        height: u32,
        /// Positive device-pixel ratio.
        device_pixel_ratio: f64,
    },
    /// Set the named time zone.
    SetTimezone {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
        /// IANA time-zone identifier.
        timezone: String,
    },
    /// Set the reduced-motion media feature.
    ///
    /// The pinned standard can express this command, but it is intentionally excluded from the
    /// reusable default plan because standard media cleanup cannot selectively restore prior state.
    SetReducedMotion {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
        /// Whether `prefers-reduced-motion` is `reduce`.
        reduce: bool,
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
/// pinned Working Draft. Reduced motion remains an expressible protocol capability, but the default
/// reusable plan does not install it because `features: null` clears the complete media-feature
/// configuration rather than restoring only OriginWeave's prior `prefers-reduced-motion` value.
/// A Browser Session owner must first bind media mutation to a genuinely disposable lifecycle or a
/// complete snapshot/restore path before constructing and sending `SetReducedMotion`.
#[must_use]
pub fn plan_standard_presentation_commands(
    context: &WebDriverBidiBrowsingContext,
    profile: &PresentationProfile,
) -> [WebDriverBidiPresentationCommand; 2] {
    [
        WebDriverBidiPresentationCommand::SetViewport {
            context: context.clone(),
            width: profile.viewport().width(),
            height: profile.viewport().height(),
            device_pixel_ratio: profile.device_pixel_ratio().value(),
        },
        WebDriverBidiPresentationCommand::SetTimezone {
            context: context.clone(),
            timezone: profile.timezone().iana_name().to_owned(),
        },
    ]
}

/// Plan cleanup that is non-destructive to unrelated media-feature overrides.
///
/// The pinned Working Draft provides independently nullable reset paths for viewport/DPR and
/// time-zone state, so these two resets are safe to plan for a reusable browsing context. Media
/// cleanup is deliberately absent because `features: null` clears the complete media-feature
/// override configuration rather than selectively undoing `prefers-reduced-motion`.
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

/// Return presentation surfaces expressible through the pinned standard BiDi contract.
///
/// Complete screen and ordered-language surfaces, hardware concurrency, and the
/// Chromium platform/User-Agent Client Hints surface are intentionally absent.
/// Reduced motion is listed as protocol capability even though reusable default application leaves
/// media state untouched until a Browser Session owner supplies a restorable lifecycle.
#[must_use]
pub const fn webdriver_bidi_presentation_surfaces() -> &'static [PresentationSurface] {
    &WEBDRIVER_BIDI_PRESENTATION_SURFACES
}

/// Require the pinned standard BiDi capability set to satisfy the complete profile.
///
/// The current result is fail-closed with
/// `PresentationError::MissingSurface(PresentationSurface::Screen)`.
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
    fn reusable_standard_commands_bind_only_symmetrically_restorable_state() {
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
            plan_standard_presentation_commands(&context, &profile),
            [
                WebDriverBidiPresentationCommand::SetViewport {
                    context: context.clone(),
                    width: 1440,
                    height: 900,
                    device_pixel_ratio: 2.0,
                },
                WebDriverBidiPresentationCommand::SetTimezone {
                    context: context.clone(),
                    timezone: "UTC".to_owned(),
                },
            ]
        );
        assert_eq!(
            WebDriverBidiPresentationCommand::SetReducedMotion {
                context: context.clone(),
                reduce: profile.reduced_motion(),
            },
            WebDriverBidiPresentationCommand::SetReducedMotion {
                context,
                reduce: true,
            }
        );
    }

    #[test]
    fn reusable_cleanup_does_not_clear_unrelated_media_feature_state() {
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
