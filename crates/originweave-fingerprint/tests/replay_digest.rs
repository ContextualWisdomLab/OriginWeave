#![allow(clippy::expect_used)]

use originweave_fingerprint::{
    DevicePixelRatio, PresentationDigest, PresentationError, PresentationPlatform,
    PresentationProfile, PresentationTimeZone, ScreenMetrics, ViewportBounds,
};

fn replay_fields() -> (
    ScreenMetrics,
    ViewportBounds,
    DevicePixelRatio,
    u16,
    PresentationTimeZone,
    PresentationPlatform,
    Vec<String>,
    bool,
) {
    (
        ScreenMetrics::new(1920, 1080).expect("screen"),
        ViewportBounds::new(1920, 900).expect("viewport"),
        DevicePixelRatio::Quantized1,
        8,
        PresentationTimeZone::Utc,
        PresentationPlatform::Linux,
        vec!["en-US".to_owned(), "en".to_owned()],
        false,
    )
}

#[test]
fn replay_requires_stored_digest_to_match_recomputed_identity() {
    let (screen, viewport, dpr, concurrency, timezone, platform, languages, reduced_motion) =
        replay_fields();
    let issued = PresentationProfile::new(
        screen,
        viewport,
        dpr,
        concurrency,
        timezone,
        platform,
        languages.clone(),
        reduced_motion,
    )
    .expect("issued profile");
    let matching_digest = issued.digest().clone();
    let mismatched_digest = PresentationDigest::new(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    )
    .expect("syntactically valid digest");

    assert_eq!(
        PresentationProfile::replay(
            screen,
            viewport,
            dpr,
            concurrency,
            timezone,
            platform,
            languages.clone(),
            reduced_motion,
            &mismatched_digest,
        ),
        Err(PresentationError::DigestMismatch)
    );

    let replayed = PresentationProfile::replay(
        screen,
        viewport,
        dpr,
        concurrency,
        timezone,
        platform,
        languages,
        reduced_motion,
        &matching_digest,
    )
    .expect("matching stored digest");
    assert_eq!(replayed.digest(), &matching_digest);

    // Invalid field construction fails closed via replay as well.
    let tall_viewport = ViewportBounds::new(1920, 1200).expect("viewport");
    assert_eq!(
        PresentationProfile::replay(
            screen,
            tall_viewport,
            dpr,
            concurrency,
            timezone,
            platform,
            vec!["en-US".to_owned()],
            false,
            &matching_digest,
        ),
        Err(PresentationError::InconsistentIdentity)
    );
}
