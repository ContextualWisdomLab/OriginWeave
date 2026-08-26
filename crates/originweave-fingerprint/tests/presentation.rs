//! Realistic presentation-profile contracts for the fingerprint kernel.
//!
//! These tests exercise the public surface a Chromium adapter would consume:
//! explicit construction, stable digest binding, cross-field consistency, and
//! fail-closed rejection of inconsistent identities.
#![allow(clippy::expect_used)]

use originweave_fingerprint::{
    DevicePixelRatio, PresentationDigest, PresentationError, PresentationPlatform,
    PresentationProfile, PresentationTimeZone, ScreenMetrics, ViewportBounds,
};

fn profile() -> PresentationProfile {
    PresentationProfile::new(
        ScreenMetrics::new(1920, 1080).expect("valid screen"),
        ViewportBounds::new(1440, 900).expect("valid viewport"),
        DevicePixelRatio::Quantized1,
        8,
        PresentationTimeZone::Utc,
        PresentationPlatform::MacOS,
        vec!["en-US".to_owned()],
        false,
    )
    .expect("consistent explicit profile")
}

#[test]
fn explicit_profile_reconstructs_the_same_identity_and_digest() {
    let first = profile();
    let second = profile();
    assert_eq!(first, second);
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.screen().color_depth_bits(), 24);
    assert!(first.viewport().width() <= first.screen().width());
    assert!(first.viewport().height() <= first.screen().height());
    assert_eq!(first.hardware_concurrency(), 8);
    assert_eq!(first.languages(), ["en-US"]);
}

#[test]
fn platform_and_pixel_ratio_never_form_a_known_contradictory_pair() {
    let screen = ScreenMetrics::new(1920, 1080).expect("valid screen");
    let viewport = ViewportBounds::new(1440, 900).expect("valid viewport");
    assert_eq!(
        PresentationProfile::new(
            screen,
            viewport,
            DevicePixelRatio::Quantized15,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::MacOS,
            vec!["en-US".to_owned()],
            false,
        ),
        Err(PresentationError::InconsistentIdentity)
    );
}

#[test]
fn digest_is_lowercase_sha256_identifier() {
    let profile = profile();
    let text = profile.digest().as_str();
    let hex = text.strip_prefix("sha256:").expect("digest prefix");
    assert_eq!(hex.len(), 64);
    assert!(
        hex.bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    );
}

#[test]
fn digest_type_rejects_malformed_identifiers() {
    assert!(
        PresentationDigest::new(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        )
        .is_ok()
    );
    assert_eq!(
        PresentationDigest::new("not-a-digest"),
        Err(PresentationError::InvalidDigest)
    );
    assert_eq!(
        PresentationDigest::new("sha256:ABCDEF"),
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
            "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        ),
        Err(PresentationError::InvalidDigest)
    );
}

#[test]
fn manual_construction_is_fail_closed_on_inconsistency() {
    assert_eq!(
        ViewportBounds::new(100, 7681),
        Err(PresentationError::InvalidField)
    );
    let screen = ScreenMetrics::new(1920, 1080).expect("valid screen");
    assert!(
        PresentationProfile::new(
            screen,
            ViewportBounds::new(1920, 1080).expect("fitting viewport"),
            DevicePixelRatio::Quantized15,
            8,
            PresentationTimeZone::Utc,
            PresentationPlatform::Windows,
            vec!["en-US".to_owned()],
            false,
        )
        .is_ok()
    );
    for viewport in [
        ViewportBounds::new(2560, 1080).expect("wide viewport"),
        ViewportBounds::new(1920, 1200).expect("tall viewport"),
    ] {
        assert!(
            PresentationProfile::new(
                screen,
                viewport,
                DevicePixelRatio::Quantized15,
                8,
                PresentationTimeZone::Utc,
                PresentationPlatform::Windows,
                vec!["en-US".to_owned()],
                false,
            )
            .is_err()
        );
    }
}

#[test]
fn explicit_profiles_use_one_named_timezone_without_dst_contradictions() {
    let profile = profile();
    assert_eq!(profile.timezone(), PresentationTimeZone::Utc);
    assert_eq!(profile.timezone().iana_name(), "UTC");
    assert_eq!(profile.timezone_offset_minutes(), 0);
}
