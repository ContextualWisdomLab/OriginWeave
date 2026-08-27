//! Realistic User-Agent Client Hints contracts for a stealth presentation.
//!
//! These tests exercise the bounded UA-CH surface an adapter must prove
//! before it can claim a coherent stealth identity: brand-name length and
//! grammar bounds, enumerated architecture/bitness/platform tokens, and the
//! spec rule that a non-mobile user agent reports an empty model.
//! Authority: User-Agent Client Hints Draft Community Group Report
//! (WICG, 2026).
#![allow(clippy::expect_used)]

use originweave_fingerprint::{
    ClientHintsError, HintsArchitecture, HintsBitness, HintsPlatform, UaBrand, UaClientHints,
};

#[test]
fn ua_brand_accepts_ascii_bounded_names_and_versions() {
    assert!(UaBrand::new("Chromium", "131.0.0.0").is_ok());
    assert!(UaBrand::new("a", "1").is_ok());
}

#[test]
fn ua_brand_accepts_realistic_chromium_and_grease_names() {
    assert!(UaBrand::new("Google Chrome", "131").is_ok());
    assert!(UaBrand::new("Not/A)Brand", "99").is_ok());
    assert!(UaBrand::new("Not_A Brand", "24.0.0.0").is_ok());
}

#[test]
fn empty_brand_name_or_version_fails_closed() {
    assert_eq!(
        UaBrand::new("", "131").expect_err("empty brand name"),
        ClientHintsError::InvalidBrandName
    );
    assert_eq!(
        UaBrand::new("Chromium", "").expect_err("empty brand version"),
        ClientHintsError::InvalidBrandName
    );
}

#[test]
fn brand_names_over_length_limit_fail_closed() {
    let long_name = "X".repeat(33);
    assert_eq!(
        UaBrand::new(&long_name, "1.0").expect_err("long name"),
        ClientHintsError::BrandTooLong
    );
}

#[test]
fn brand_versions_over_resource_limit_fail_closed() {
    let boundary_version = "1".repeat(32);
    assert!(UaBrand::new("Chromium", &boundary_version).is_ok());

    let long_version = "1".repeat(33);
    assert_eq!(
        UaBrand::new("Chromium", &long_version).expect_err("long version"),
        ClientHintsError::BrandVersionTooLong
    );
}

#[test]
fn brand_names_with_invalid_grammar_fail_closed() {
    assert_eq!(
        UaBrand::new("Chromium!", "1.0").expect_err("bad name"),
        ClientHintsError::InvalidBrandName
    );
}

#[test]
fn brand_versions_with_invalid_grammar_fail_closed() {
    assert_eq!(
        UaBrand::new("Chromium", "1.0-beta!").expect_err("bad version"),
        ClientHintsError::InvalidBrandName
    );
}

#[test]
fn hints_bound_architectures_to_enumerated_tokens() {
    assert!(HintsArchitecture::from_token("x86").is_some());
    assert!(HintsArchitecture::from_token("arm").is_some());
    assert!(HintsArchitecture::from_token("m68k").is_none());
}

#[test]
fn hints_bitness_bound_to_enumerated_tokens() {
    assert!(HintsBitness::from_token("32").is_some());
    assert!(HintsBitness::from_token("64").is_some());
    assert!(HintsBitness::from_token("128").is_none());
}

#[test]
fn hints_platform_normalizes_to_the_low_entropy_set() {
    assert_eq!(
        HintsPlatform::normalize("Windows"),
        Ok(HintsPlatform::Windows)
    );
    assert_eq!(HintsPlatform::normalize("macOS"), Ok(HintsPlatform::MacOs));
    assert_eq!(HintsPlatform::normalize("Linux"), Ok(HintsPlatform::Linux));
    assert_eq!(
        HintsPlatform::normalize("AmazingOS"),
        Err(ClientHintsError::InvalidPlatform)
    );
}

#[test]
fn non_mobile_client_hints_require_an_empty_model() {
    let ok = UaClientHints::new(
        HintsPlatform::Windows,
        HintsArchitecture::from_token("x86").expect("arch"),
        HintsBitness::from_token("64").expect("bits"),
        false,
        "",
        vec![UaBrand::new("Chromium", "131.0.0.0").expect("brand")],
    );
    assert!(ok.is_ok());

    let contradiction = UaClientHints::new(
        HintsPlatform::Windows,
        HintsArchitecture::from_token("x86").expect("arch"),
        HintsBitness::from_token("64").expect("bits"),
        false,
        "Pixel 2 XL",
        vec![UaBrand::new("Chromium", "131.0.0.0").expect("brand")],
    );
    assert_eq!(contradiction, Err(ClientHintsError::ModelWithoutMobile));
}

#[test]
fn mobile_hints_may_carry_a_model_without_exceeding_the_set() {
    let mobile = UaClientHints::new(
        HintsPlatform::Linux,
        HintsArchitecture::from_token("arm").expect("arch"),
        HintsBitness::from_token("64").expect("bits"),
        true,
        "Pixel 2 XL",
        vec![UaBrand::new("Chromium", "131.0.0.0").expect("brand")],
    );
    assert!(mobile.is_ok());
}

#[test]
fn mobile_models_over_resource_limit_fail_closed() {
    let brand = UaBrand::new("Chromium", "131.0.0.0").expect("brand");
    let boundary_model = "M".repeat(64);
    assert!(
        UaClientHints::new(
            HintsPlatform::Linux,
            HintsArchitecture::Arm,
            HintsBitness::Bit64,
            true,
            &boundary_model,
            vec![brand.clone()],
        )
        .is_ok()
    );

    let long_model = "M".repeat(65);
    assert_eq!(
        UaClientHints::new(
            HintsPlatform::Linux,
            HintsArchitecture::Arm,
            HintsBitness::Bit64,
            true,
            &long_model,
            vec![brand],
        ),
        Err(ClientHintsError::ModelTooLong)
    );
}

#[test]
fn non_mobile_model_semantics_precede_model_length_budget() {
    let long_model = "M".repeat(65);
    assert_eq!(
        UaClientHints::new(
            HintsPlatform::Linux,
            HintsArchitecture::Arm,
            HintsBitness::Bit64,
            false,
            &long_model,
            vec![UaBrand::new("Chromium", "131.0.0.0").expect("brand")],
        ),
        Err(ClientHintsError::ModelWithoutMobile)
    );
}

#[test]
fn empty_brand_list_fails_closed() {
    assert_eq!(
        UaClientHints::new(
            HintsPlatform::Linux,
            HintsArchitecture::from_token("x86").expect("arch"),
            HintsBitness::from_token("64").expect("bits"),
            false,
            "",
            vec![],
        ),
        Err(ClientHintsError::MissingBrand)
    );
}

#[test]
fn client_hints_error_has_deterministic_display() {
    assert_eq!(
        ClientHintsError::InvalidPlatform.to_string(),
        "platform must be one of the enumerated UA Client Hints platform values"
    );
    assert_eq!(
        ClientHintsError::ModelWithoutMobile.to_string(),
        "a non-mobile user agent must report an empty model"
    );
    assert_eq!(
        ClientHintsError::BrandTooLong.to_string(),
        "brand name must be at most 32 ASCII characters"
    );
    assert_eq!(
        ClientHintsError::BrandVersionTooLong.to_string(),
        "brand version must be at most 32 ASCII characters"
    );
    assert_eq!(
        ClientHintsError::ModelTooLong.to_string(),
        "mobile model must be at most 64 bytes"
    );
    assert_eq!(
        ClientHintsError::InvalidBrandName.to_string(),
        "brand name must use bounded UA-CH-compatible ASCII and version must be non-empty dotted ASCII alphanumeric"
    );
    assert_eq!(
        ClientHintsError::MissingBrand.to_string(),
        "a client-hints value must contain at least one brand"
    );
}

#[test]
fn every_public_accessor_exposes_the_validated_value() {
    let brand = UaBrand::new("Chromium", "131.0.0.0").expect("brand");
    assert_eq!(brand.name(), "Chromium");
    assert_eq!(brand.version(), "131.0.0.0");

    assert_eq!(
        HintsArchitecture::from_token("x86").expect("x").token(),
        "x86"
    );
    assert_eq!(
        HintsArchitecture::from_token("arm").expect("a").token(),
        "arm"
    );

    assert_eq!(HintsBitness::from_token("32").expect("b").token(), "32");
    assert_eq!(HintsBitness::from_token("64").expect("b").token(), "64");

    assert_eq!(
        HintsPlatform::normalize("Windows").expect("w").token(),
        "Windows"
    );
    assert_eq!(
        HintsPlatform::normalize("macOS").expect("m").token(),
        "macOS"
    );
    assert_eq!(
        HintsPlatform::normalize("Linux").expect("l").token(),
        "Linux"
    );

    let hints = UaClientHints::new(
        HintsPlatform::Windows,
        HintsArchitecture::from_token("x86").expect("arch"),
        HintsBitness::from_token("64").expect("bits"),
        false,
        "",
        vec![brand.clone()],
    )
    .expect("hints");
    assert_eq!(hints.platform(), HintsPlatform::Windows);
    assert_eq!(hints.architecture(), HintsArchitecture::X86);
    assert_eq!(hints.bitness(), HintsBitness::Bit64);
    assert!(!hints.mobile());
    assert_eq!(hints.model(), "");
    assert_eq!(hints.brands(), [brand]);
}
