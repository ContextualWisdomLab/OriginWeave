use originweave_fingerprint::{
    PresentationError, PresentationSurface, require_presentation_surfaces,
};

/// Published WebDriver BiDi Working Draft revision used by this capability map.
pub const WEBDRIVER_BIDI_PRESENTATION_REVISION: &str = "2026-08-18";

const WEBDRIVER_BIDI_PRESENTATION_SURFACES: [PresentationSurface; 6] = [
    PresentationSurface::Screen,
    PresentationSurface::Viewport,
    PresentationSurface::DevicePixelRatio,
    PresentationSurface::TimeZone,
    PresentationSurface::Languages,
    PresentationSurface::ReducedMotion,
];

/// Return presentation surfaces expressible through the pinned standard BiDi contract.
///
/// Hardware concurrency and the complete Chromium platform/User-Agent Client Hints
/// surface are intentionally absent. Those remain version-pinned Chromium-adapter
/// responsibilities rather than ambient standard-BiDi authority.
#[must_use]
pub const fn webdriver_bidi_presentation_surfaces() -> &'static [PresentationSurface] {
    &WEBDRIVER_BIDI_PRESENTATION_SURFACES
}

/// Require the pinned standard BiDi capability set to satisfy the complete profile.
///
/// The current result is fail-closed with
/// `PresentationError::MissingSurface(PresentationSurface::HardwareConcurrency)`.
/// Callers must not translate that result into ambient-host fallback.
pub fn require_complete_presentation_profile() -> Result<(), PresentationError> {
    require_presentation_surfaces(webdriver_bidi_presentation_surfaces())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_revision_tracks_current_published_working_draft() {
        assert_eq!(WEBDRIVER_BIDI_PRESENTATION_REVISION, "2026-09-03");
    }

    #[test]
    fn standard_bidi_claims_only_complete_canonical_surfaces() {
        let surfaces = webdriver_bidi_presentation_surfaces();

        assert_eq!(
            require_complete_presentation_profile(),
            Err(PresentationError::MissingSurface(PresentationSurface::Screen))
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
