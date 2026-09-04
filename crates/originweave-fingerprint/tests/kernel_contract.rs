//! Realistic presentation-kernel contracts for the fingerprint crate.
#![allow(clippy::expect_used)]

use originweave_fingerprint::{
    DevicePixelRatio, PresentationDigest, PresentationError, PresentationPlatform,
    PresentationProfile, PresentationSurface, PresentationTimeZone, ScreenMetrics, ViewportBounds,
    require_presentation_surfaces,
};

#[test]
fn presentation_error_display_covers_every_variant() {
    assert_eq!(
        PresentationError::InvalidDigest.to_string(),
        "digest must be sha256: plus 64 lowercase hex digits"
    );
    assert_eq!(
        PresentationError::DigestMismatch.to_string(),
        "stored presentation digest does not match profile fields"
    );
    assert_eq!(
        PresentationError::InvalidField.to_string(),
        "presentation field violates its bounded contract"
    );
    assert_eq!(
        PresentationError::InconsistentIdentity.to_string(),
        "presentation fields contradict each other"
    );
    assert_eq!(
        PresentationError::MissingSurface(PresentationSurface::HardwareConcurrency).to_string(),
        "adapter cannot override required HardwareConcurrency surface"
    );
}

#[test]
fn screen_metrics_reject_zero_and_oversized_edges() {
    assert_eq!(
        ScreenMetrics::new(0, 1080),
        Err(PresentationError::InvalidField)
    );
    assert_eq!(
        ScreenMetrics::new(1920, 0),
        Err(PresentationError::InvalidField)
    );
    assert_eq!(
        ScreenMetrics::new(7681, 1080),
        Err(PresentationError::InvalidField)
    );
    assert_eq!(
        ScreenMetrics::new(1920, 7681),
        Err(PresentationError::InvalidField)
    );
    let screen = ScreenMetrics::new(1920, 1080).expect("valid screen");
    assert_eq!(screen.color_depth_bits(), 24);
}

#[test]
fn viewport_bounds_reject_invalid_dimensions() {
    assert_eq!(
        ViewportBounds::new(0, 100),
        Err(PresentationError::InvalidField)
    );
    assert_eq!(
        ViewportBounds::new(100, 0),
        Err(PresentationError::InvalidField)
    );
    assert_eq!(
        ViewportBounds::new(7681, 100),
        Err(PresentationError::InvalidField)
    );
    assert_eq!(
        ViewportBounds::new(100, 7681),
        Err(PresentationError::InvalidField)
    );
    let viewport = ViewportBounds::new(1280, 720).expect("valid viewport");
    assert_eq!((viewport.width(), viewport.height()), (1280, 720));
}

#[test]
fn device_pixel_ratio_maps_exact_quantized_values() {
    assert_eq!(
        DevicePixelRatio::from_ratio(1.0),
        Some(DevicePixelRatio::Quantized1)
    );
    assert_eq!(
        DevicePixelRatio::from_ratio(1.5),
        Some(DevicePixelRatio::Quantized15)
    );
    assert_eq!(
        DevicePixelRatio::from_ratio(2.0),
        Some(DevicePixelRatio::Quantized2)
    );
    assert_eq!(DevicePixelRatio::from_ratio(1.25), None);
    for ratio in [
        DevicePixelRatio::Quantized1,
        DevicePixelRatio::Quantized15,
        DevicePixelRatio::Quantized2,
    ] {
        assert_eq!(
            ratio.value(),
            DevicePixelRatio::from_ratio(ratio.value())
                .expect("round trip")
                .value()
        );
    }
}

#[test]
fn platform_tokens_are_stable() {
    assert_eq!(PresentationPlatform::Windows.user_agent_token(), "Win32");
    assert_eq!(PresentationPlatform::MacOS.user_agent_token(), "MacIntel");
    assert_eq!(
        PresentationPlatform::Linux.user_agent_token(),
        "Linux x86_64"
    );
}

#[test]
fn digest_validation_rejects_each_malformation() {
    assert_eq!(
        PresentationDigest::new(""),
        Err(PresentationError::InvalidDigest)
    );
    assert_eq!(
        PresentationDigest::new(
            "sha257:0000000000000000000000000000000000000000000000000000000000000000"
        ),
        Err(PresentationError::InvalidDigest)
    );
    assert_eq!(
        PresentationDigest::new(
            "sha256:00000000000000000000000000000000000000000000000000000000000000"
        ),
        Err(PresentationError::InvalidDigest)
    );
    assert_eq!(
        PresentationDigest::new(
            "sha256:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
        ),
        Err(PresentationError::InvalidDigest)
    );
    assert_eq!(
        PresentationDigest::new(
            "sha256:A000000000000000000000000000000000000000000000000000000000000000"
        ),
        Err(PresentationError::InvalidDigest)
    );
    let valid = PresentationDigest::new(
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    )
    .expect("valid digest");
    assert_eq!(valid.to_string(), valid.as_str());
}

#[test]
fn standardized_timezone_has_one_consistent_identity() {
    assert_eq!(PresentationTimeZone::Utc.iana_name(), "UTC");
    assert_eq!(PresentationTimeZone::Utc.offset_minutes(), 0);
}

#[test]
fn profile_new_validates_each_field_independently() {
    let screen = ScreenMetrics::new(1920, 1080).expect("screen");
    let viewport = ViewportBounds::new(1920, 900).expect("viewport");

    // Viewport taller than the screen is impossible.
    let tall = ViewportBounds::new(1920, 1200).expect("viewport");
    assert_eq!(
        PresentationProfile::new(
            screen,
            tall,
            DevicePixelRatio::Quantized1,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::Linux,
            vec!["en".to_owned()],
            false
        ),
        Err(PresentationError::InconsistentIdentity)
    );
    let wide = ViewportBounds::new(2560, 1080).expect("viewport");
    assert_eq!(
        PresentationProfile::new(
            screen,
            wide,
            DevicePixelRatio::Quantized1,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::Linux,
            vec!["en".to_owned()],
            false
        ),
        Err(PresentationError::InconsistentIdentity)
    );

    // Trusted replay cannot reintroduce high-entropy arbitrary dimensions.
    let odd_screen = ScreenMetrics::new(1919, 1080).expect("bounded screen");
    assert_eq!(
        PresentationProfile::new(
            odd_screen,
            ViewportBounds::new(1024, 600).expect("viewport"),
            DevicePixelRatio::Quantized1,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::Linux,
            vec!["en".to_owned()],
            false
        ),
        Err(PresentationError::InvalidField)
    );
    let odd_viewport = ViewportBounds::new(1919, 900).expect("bounded viewport");
    assert_eq!(
        PresentationProfile::new(
            screen,
            odd_viewport,
            DevicePixelRatio::Quantized1,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::Linux,
            vec!["en".to_owned()],
            false
        ),
        Err(PresentationError::InvalidField)
    );
    let odd_viewport_height = ViewportBounds::new(1920, 899).expect("bounded viewport");
    assert_eq!(
        PresentationProfile::new(
            screen,
            odd_viewport_height,
            DevicePixelRatio::Quantized1,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::Linux,
            vec!["en".to_owned()],
            false
        ),
        Err(PresentationError::InvalidField)
    );

    // Processor count outside the enumerated set is rejected.
    assert_eq!(
        PresentationProfile::new(
            screen,
            viewport,
            DevicePixelRatio::Quantized1,
            3,
            PresentationTimeZone::Utc,
            PresentationPlatform::Linux,
            vec!["en".to_owned()],
            false
        ),
        Err(PresentationError::InvalidField)
    );

    // Language validation flows through.
    assert_eq!(
        PresentationProfile::new(
            screen,
            viewport,
            DevicePixelRatio::Quantized1,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::Linux,
            Vec::new(),
            false
        ),
        Err(PresentationError::InvalidField)
    );
    for languages in [
        vec!["cy-GB".to_owned()],
        vec!["cy-GB".to_owned(), "en".to_owned()],
        vec!["ko-KR".to_owned(), "fr-FR".to_owned()],
        vec!["ko-KR".to_owned(), "en".to_owned(), "en-GB".to_owned()],
    ] {
        assert_eq!(
            PresentationProfile::new(
                screen,
                viewport,
                DevicePixelRatio::Quantized1,
                8,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                languages,
                false
            ),
            Err(PresentationError::InvalidField)
        );
    }

    assert_eq!(
        PresentationProfile::new(
            screen,
            viewport,
            DevicePixelRatio::Quantized15,
            12,
            PresentationTimeZone::Utc,
            PresentationPlatform::MacOS,
            vec!["ko-KR".to_owned(), "en".to_owned()],
            true,
        ),
        Err(PresentationError::InconsistentIdentity)
    );
    let profile = PresentationProfile::new(
        screen,
        viewport,
        DevicePixelRatio::Quantized1,
        12,
        PresentationTimeZone::Utc,
        PresentationPlatform::MacOS,
        vec!["ko-KR".to_owned(), "en".to_owned()],
        true,
    )
    .expect("valid profile");
    assert_eq!(profile.screen().width(), 1920);
    assert_eq!(profile.screen().height(), 1080);
    assert_eq!(profile.device_pixel_ratio().value(), 1.0);
    assert_eq!(profile.hardware_concurrency(), 12);
    assert_eq!(profile.timezone_offset_minutes(), 0);
    assert_eq!(profile.timezone(), PresentationTimeZone::Utc);
    assert_eq!(profile.platform(), PresentationPlatform::MacOS);
    assert_eq!(profile.languages().len(), 2);
    assert!(profile.reduced_motion());
}

#[test]
fn surface_admission_checks_all_required_surfaces() {
    let surfaces = [
        PresentationSurface::Screen,
        PresentationSurface::Viewport,
        PresentationSurface::DevicePixelRatio,
        PresentationSurface::HardwareConcurrency,
        PresentationSurface::TimeZone,
        PresentationSurface::Platform,
        PresentationSurface::Languages,
        PresentationSurface::ReducedMotion,
    ];
    assert!(require_presentation_surfaces(&surfaces).is_ok());
}
