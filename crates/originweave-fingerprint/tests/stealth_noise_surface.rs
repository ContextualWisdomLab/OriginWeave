//! Realistic stealth-normalization contracts for the fingerprint kernel.
//!
//! These tests exercise the bounded render and media surfaces an adapter
//! must prove before it may claim a complete stealth presentation: canvas
//! noise quantization, WebGL renderer tokens, WebAudio sample-rate
//! normalization, WebRTC interface policy, and the surface admission
//! contract that forces fail-closed completeness.
#![allow(clippy::expect_used)]

use originweave_fingerprint::{
    CanvasNoise, StealthError, StealthSurface, WebAudioRate, WebGlRendererToken, WebRtcInterface,
    require_stealth_surfaces,
};

const COMPLETE_STEALTH_SURFACES: [StealthSurface; 4] = [
    StealthSurface::Canvas,
    StealthSurface::WebGL,
    StealthSurface::WebAudio,
    StealthSurface::WebRtc,
];

#[test]
fn incomplete_adapter_support_fails_on_the_first_missing_surface() {
    let supported = COMPLETE_STEALTH_SURFACES
        .into_iter()
        .filter(|surface| *surface != StealthSurface::WebGL)
        .collect::<Vec<_>>();

    assert_eq!(
        require_stealth_surfaces(&supported),
        Err(StealthError::MissingSurface(StealthSurface::WebGL))
    );
}

#[test]
fn complete_adapter_surface_support_is_order_and_duplicate_independent() {
    let mut supported = COMPLETE_STEALTH_SURFACES.to_vec();
    supported.reverse();
    supported.push(StealthSurface::Canvas);

    assert_eq!(require_stealth_surfaces(&supported), Ok(()));
}

#[test]
fn empty_adapter_surface_support_reports_canvas_first() {
    assert_eq!(
        require_stealth_surfaces(&[]),
        Err(StealthError::MissingSurface(StealthSurface::Canvas))
    );
}

#[test]
fn canvas_noise_quantizes_only_supported_classes() {
    assert_eq!(CanvasNoise::quantize(0), Ok(CanvasNoise::Crisp));
    assert_eq!(CanvasNoise::quantize(1), Ok(CanvasNoise::Smooth));
    assert_eq!(CanvasNoise::quantize(2), Ok(CanvasNoise::Diffuse));
    assert_eq!(
        CanvasNoise::quantize(3),
        Err(StealthError::InvalidCanvasNoise)
    );
}

#[test]
fn canvas_noise_class_bit_shift_is_bound_to_the_declared_class() {
    assert_eq!(CanvasNoise::Crisp.bit_shift(), 0);
    assert_eq!(CanvasNoise::Smooth.bit_shift(), 1);
    assert_eq!(CanvasNoise::Diffuse.bit_shift(), 2);
}

#[test]
fn web_gl_renderer_tokens_are_bounded_and_standardized() {
    assert_eq!(
        WebGlRendererToken::canonical("ANGLE (NVIDIA GeForce RTX 4090)"),
        Some(WebGlRendererToken::Angle)
    );
    assert_eq!(
        WebGlRendererToken::canonical("WebKit Software Rendering"),
        Some(WebGlRendererToken::Standard)
    );
    assert_eq!(WebGlRendererToken::canonical("Mozilla/5.0"), None);
}

#[test]
fn web_audio_rates_normalize_only_standard_rates() {
    assert_eq!(WebAudioRate::normalize(44_100), Ok(WebAudioRate::Rate44100));
    assert_eq!(WebAudioRate::normalize(48_000), Ok(WebAudioRate::Rate48000));
    assert_eq!(
        WebAudioRate::normalize(22_050),
        Err(StealthError::InvalidSampleRate)
    );
}

#[test]
fn web_rtc_interface_policy_names_direct_candidate_disclosure_explicitly() {
    assert!(WebRtcInterface::DirectCandidates.exposes_candidates());
    assert!(!WebRtcInterface::MDnsOnly.exposes_candidates());
}

#[test]
fn stealth_errors_implement_display_for_adapters() {
    assert_eq!(
        StealthError::InvalidCanvasNoise.to_string(),
        "canvas noise class must be one of the enumerated supported values"
    );
    assert_eq!(
        StealthError::InvalidSampleRate.to_string(),
        "web audio sample rate must be a supported standard rate"
    );
}

#[test]
fn web_audio_rate_accessors_expose_exact_hertz() {
    assert_eq!(WebAudioRate::Rate44100.rate_hz(), 44_100);
    assert_eq!(WebAudioRate::Rate48000.rate_hz(), 48_000);
}

#[test]
fn missing_surface_error_formats_cleanly_for_each_surface() {
    let surfaces = [
        StealthSurface::Canvas,
        StealthSurface::WebGL,
        StealthSurface::WebAudio,
        StealthSurface::WebRtc,
    ];
    for surface in surfaces {
        let err = StealthError::MissingSurface(surface);
        assert!(err.to_string().contains("adapter cannot override required"));
    }
}
