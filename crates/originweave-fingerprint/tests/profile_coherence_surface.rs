//! Cross-surface platform-coherence contracts for stealth presentations.
//!
//! A page can reconcile the static profile, the UA string, and the UA Client
//! Hints object into one identity. If an adapter presents a `Windows`
//! presentation platform but reports `macOS` UA Client Hints (or a mismatched
//! UA token), the contradiction is itself a reidentification signal. These
//! tests bind the hints platform and UA token to the presentation platform so
//! the three surfaces stay mutually coherent.
#![allow(clippy::expect_used)]

use originweave_fingerprint::{
    CoherenceError, HintsArchitecture, HintsBitness, HintsPlatform, PresentationPlatform, UaBrand,
    UaClientHints, require_hints_coherence,
};

fn hints_for(platform: HintsPlatform) -> UaClientHints {
    UaClientHints::new(
        platform,
        HintsArchitecture::from_token("x86").expect("arch"),
        HintsBitness::from_token("64").expect("bits"),
        false,
        "",
        vec![UaBrand::new("Chromium", "131.0.0.0").expect("brand")],
    )
    .expect("hints")
}

#[test]
fn matching_hints_platform_is_accepted() {
    let windows = hints_for(HintsPlatform::Windows);
    assert_eq!(
        require_hints_coherence(&windows, PresentationPlatform::Windows),
        Ok(())
    );
    let macos = hints_for(HintsPlatform::MacOs);
    assert_eq!(
        require_hints_coherence(&macos, PresentationPlatform::MacOS),
        Ok(())
    );
    let linux = hints_for(HintsPlatform::Linux);
    assert_eq!(
        require_hints_coherence(&linux, PresentationPlatform::Linux),
        Ok(())
    );
}

#[test]
fn mismatched_hints_platform_fails_closed() {
    let windows_hints = hints_for(HintsPlatform::Windows);
    assert_eq!(
        require_hints_coherence(&windows_hints, PresentationPlatform::MacOS),
        Err(CoherenceError::HintsPlatformMismatch)
    );
    assert_eq!(
        require_hints_coherence(&windows_hints, PresentationPlatform::Linux),
        Err(CoherenceError::HintsPlatformMismatch)
    );
}

#[test]
fn every_presentation_platform_maps_to_its_canonical_hints_token() {
    assert_eq!(
        PresentationPlatform::Windows.hints_platform(),
        HintsPlatform::Windows
    );
    assert_eq!(
        PresentationPlatform::MacOS.hints_platform(),
        HintsPlatform::MacOs
    );
    assert_eq!(
        PresentationPlatform::Linux.hints_platform(),
        HintsPlatform::Linux
    );
}

#[test]
fn every_presentation_platform_maps_to_its_canonical_ua_token() {
    assert_eq!(PresentationPlatform::Windows.user_agent_token(), "Win32");
    assert_eq!(PresentationPlatform::MacOS.user_agent_token(), "MacIntel");
    assert_eq!(
        PresentationPlatform::Linux.user_agent_token(),
        "Linux x86_64"
    );
}

#[test]
fn coherence_error_has_deterministic_display() {
    assert_eq!(
        CoherenceError::HintsPlatformMismatch.to_string(),
        "UA Client Hints platform contradicts the presentation platform"
    );
}

#[test]
fn coherent_profile_round_trips_hints_to_presentation() {
    for (presentation, hints) in [
        (PresentationPlatform::Windows, HintsPlatform::Windows),
        (PresentationPlatform::MacOS, HintsPlatform::MacOs),
        (PresentationPlatform::Linux, HintsPlatform::Linux),
    ] {
        let profile_hints = hints_for(hints);
        assert_eq!(
            profile_hints.platform(),
            presentation.hints_platform(),
            "canonical hints token must equal the presentation mapping"
        );
        assert_eq!(
            require_hints_coherence(&profile_hints, presentation),
            Ok(())
        );
    }
}
