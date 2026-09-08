use originweave_fingerprint::{
    PresentationError, PresentationSurface, require_presentation_surfaces,
};

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
mod tests {
    use super::*;

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
}
