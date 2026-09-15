use originweave_fingerprint::{
    PresentationError, PresentationSurface, require_presentation_surfaces,
};

const COMPLETE_SURFACES: [PresentationSurface; 8] = [
    PresentationSurface::Screen,
    PresentationSurface::Viewport,
    PresentationSurface::DevicePixelRatio,
    PresentationSurface::HardwareConcurrency,
    PresentationSurface::TimeZone,
    PresentationSurface::Platform,
    PresentationSurface::Languages,
    PresentationSurface::ReducedMotion,
];

#[test]
fn incomplete_adapter_support_fails_on_the_first_missing_surface() {
    let supported = COMPLETE_SURFACES
        .into_iter()
        .filter(|surface| *surface != PresentationSurface::HardwareConcurrency)
        .collect::<Vec<_>>();

    assert_eq!(
        require_presentation_surfaces(&supported),
        Err(PresentationError::MissingSurface(
            PresentationSurface::HardwareConcurrency
        ))
    );
}

#[test]
fn complete_adapter_support_is_order_and_duplicate_independent() {
    let mut supported = COMPLETE_SURFACES.to_vec();
    supported.reverse();
    supported.push(PresentationSurface::Screen);

    assert_eq!(require_presentation_surfaces(&supported), Ok(()));
}
