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
/// prove an acknowledgement, or establish page-observed presentation evidence.
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
    SetReducedMotion {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
        /// Whether `prefers-reduced-motion` is `reduce`.
        reduce: bool,
    },
    /// Restore the implementation-defined viewport and remove the persistent DPR override.
    ResetViewport {
        /// Exact target browsing context.
        context: WebDriverBidiBrowsingContext,
    },
}

/// Plan the three typed standard-BiDi commands covering the four admitted surfaces.
///
/// Screen, hardware concurrency, platform, and ordered languages are intentionally absent.
#[must_use]
pub fn plan_standard_presentation_commands(
    context: &WebDriverBidiBrowsingContext,
    profile: &PresentationProfile,
) -> [WebDriverBidiPresentationCommand; 3] {
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
        WebDriverBidiPresentationCommand::SetReducedMotion {
            context: context.clone(),
            reduce: profile.reduced_motion(),
        },
    ]
}

/// Plan explicit cleanup for viewport dimensions and device-pixel ratio.
///
/// WebDriver BiDi does not clear its DPR override when the final session ends. This command intent
/// sets both viewport and DPR to `null`; planning it does not prove transport, acknowledgement, or
/// page-observed cleanup.
#[must_use]
pub fn plan_standard_presentation_cleanup(
    context: &WebDriverBidiBrowsingContext,
) -> WebDriverBidiPresentationCommand {
    WebDriverBidiPresentationCommand::ResetViewport {
        context: context.clone(),
    }
}

/// Published WebDriver BiDi Working Draft revision used by this capability map.
pub const WEBDRIVER_BIDI_PRESENTATION_REVISION: &str = "2026-08-18";

/// Immutable upstream source commit used to doctor same-day emulation semantics.
///
/// The dated W3C Working Draft remains the publication identity. This commit records the exact
/// `w3c/webdriver-bidi` source snapshot used when interpreting same-day media-feature capability
/// details, including `prefers-reduced-motion`; it is not treated as a second protocol version.
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
/// Those remain version-pinned Chromium-adapter responsibilities rather than
/// ambient standard-BiDi authority.
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
    fn standard_commands_bind_complete_surfaces_to_one_context_without_claiming_success() {
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
                WebDriverBidiPresentationCommand::SetReducedMotion {
                    context,
                    reduce: true,
                },
            ]
        );
    }

    #[test]
    fn cleanup_plan_explicitly_resets_viewport_and_persistent_dpr_override() {
        let context =
            WebDriverBidiBrowsingContext::new("context-17").expect("bounded context identifier");

        assert_eq!(
            plan_standard_presentation_cleanup(&context),
            WebDriverBidiPresentationCommand::ResetViewport { context }
        );
    }
}
