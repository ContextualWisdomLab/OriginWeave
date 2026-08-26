//! Realistic presentation-profile contracts for the fingerprint kernel.
//!
//! These tests exercise the public surface a Chromium adapter would consume:
//! seeded derivation, per-session stability, cross-field consistency, and
//! fail-closed rejection of degenerate or inconsistent identities.
#![allow(clippy::expect_used)]

use originweave_fingerprint::{
    DevicePixelRatio, PresentationDigest, PresentationError, PresentationPlatform,
    PresentationProfile, PresentationSeed, PresentationTimeZone, ScreenMetrics, ViewportBounds,
};

const SEED_A: [u8; 32] = [
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
];

#[allow(dead_code)]
const SEED_B: [u8; 32] = [
    0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01,
    0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88,
];

fn seed(bytes: [u8; 32]) -> PresentationSeed {
    PresentationSeed::new(bytes).expect("valid nonzero seed")
}

#[test]
fn seed_rejects_all_zero_and_accepts_valid_seed() {
    assert_eq!(
        PresentationSeed::new([0u8; 32]),
        Err(PresentationError::DegenerateSeed)
    );
    let accepted = seed(SEED_A);
    assert_eq!(accepted.bytes(), &SEED_A);
}

#[test]
fn derivation_is_deterministic_per_seed() {
    let first = PresentationProfile::derive(&seed(SEED_A));
    let second = PresentationProfile::derive(&seed(SEED_A));
    assert_eq!(first, second);
    assert_eq!(first.digest(), second.digest());
}

#[test]
fn distinct_seeds_yield_distinct_identities() {
    let left = PresentationProfile::derive(&seed(SEED_A));
    let right = PresentationProfile::derive(&seed(SEED_B));
    assert_ne!(left, right);
    assert_ne!(left.digest(), right.digest());
}

#[test]
fn derived_profiles_stay_internally_consistent() {
    for offset in 0..64u8 {
        let mut bytes = SEED_A;
        bytes[31] = bytes[31].wrapping_add(offset);
        let profile = PresentationProfile::derive(&seed(bytes));

        let screen = profile.screen();
        assert!((1280..=3840).contains(&screen.width()));
        assert!((720..=2160).contains(&screen.height()));
        assert_eq!(screen.color_depth_bits(), 24);

        let viewport = profile.viewport();
        assert!(viewport.width() > 0 && viewport.height() > 0);
        assert!(viewport.width() <= screen.width());
        assert!(viewport.height() <= screen.height());

        assert!(matches!(
            profile.device_pixel_ratio(),
            DevicePixelRatio::Quantized1
                | DevicePixelRatio::Quantized15
                | DevicePixelRatio::Quantized2
        ));
        assert!((2..=16).contains(&profile.hardware_concurrency()));
        assert!(profile.timezone_offset_minutes() == 0);
        assert!(!profile.languages().is_empty());
        assert!(profile.languages().len() <= 4);
        assert!(!profile.platform().user_agent_token().is_empty());
    }
}

#[test]
fn digest_is_lowercase_sha256_identifier() {
    let profile = PresentationProfile::derive(&seed(SEED_A));
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
fn derived_profiles_use_one_named_timezone_without_dst_contradictions() {
    for bytes in [SEED_A, SEED_B] {
        let profile = PresentationProfile::derive(&seed(bytes));
        assert_eq!(profile.timezone(), PresentationTimeZone::Utc);
        assert_eq!(profile.timezone().iana_name(), "UTC");
        assert_eq!(profile.timezone_offset_minutes(), 0);
    }
}

#[test]
fn derivation_covers_one_and_two_language_profiles() {
    let mut observed_lengths = std::collections::BTreeSet::new();
    for last_byte in 0..=u8::MAX {
        let mut bytes = SEED_A;
        bytes[31] = last_byte;
        observed_lengths.insert(PresentationProfile::derive(&seed(bytes)).languages().len());
    }
    assert_eq!(observed_lengths, std::collections::BTreeSet::from([1, 2]));
}
