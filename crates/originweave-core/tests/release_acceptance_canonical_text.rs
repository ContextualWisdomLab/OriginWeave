use originweave_core::release_acceptance::{DeclaredLimitation, ReleaseDecisionError};

#[test]
fn limitation_accepts_canonical_boundary_text() {
    let limitation = DeclaredLimitation::new(
        "linux_arm64",
        "Linux ARM64 is excluded from the support profile.",
    );

    assert_eq!(
        limitation
            .as_ref()
            .map(|value| (value.unsupported_claim(), value.buyer_consequence())),
        Ok((
            "linux_arm64",
            "Linux ARM64 is excluded from the support profile."
        ))
    );
}

#[test]
fn limitation_rejects_empty_fields_for_the_canonical_string_input_shape() {
    assert_eq!(
        DeclaredLimitation::new("", "Linux ARM64 is excluded from the support profile."),
        Err(ReleaseDecisionError::EmptyLimitationClaim),
    );
    assert_eq!(
        DeclaredLimitation::new("linux_arm64", ""),
        Err(ReleaseDecisionError::EmptyLimitationConsequence),
    );
}

#[test]
fn limitation_rejects_surrounding_whitespace_that_changes_claim_identity() {
    for unsupported_claim in [" linux_arm64", "linux_arm64 ", "\tlinux_arm64"] {
        assert_eq!(
            DeclaredLimitation::new(
                unsupported_claim,
                "Linux ARM64 is excluded from the support profile.",
            ),
            Err(ReleaseDecisionError::InvalidLimitationClaim),
            "surrounding whitespace must not create a second spelling for one claim identity: {unsupported_claim:?}",
        );
    }
}

#[test]
fn limitation_rejects_surrounding_whitespace_in_buyer_consequence() {
    for buyer_consequence in [
        " Linux ARM64 is excluded from the support profile.",
        "Linux ARM64 is excluded from the support profile. ",
        "Linux ARM64 is excluded from the support profile.\t",
    ] {
        assert_eq!(
            DeclaredLimitation::new("linux_arm64", buyer_consequence),
            Err(ReleaseDecisionError::InvalidLimitationConsequence),
            "buyer-visible consequence must have one canonical boundary spelling: {buyer_consequence:?}",
        );
    }
}

#[test]
fn limitation_rejects_non_nfc_claim_identity() {
    let nfc_claim = "caf\u{e9}";
    let canonically_equivalent_nfd_claim = "cafe\u{301}";

    assert!(
        DeclaredLimitation::new(
            nfc_claim,
            "This normalized claim remains a supported buyer-visible spelling.",
        )
        .is_ok(),
        "NFC international text must remain admissible",
    );
    assert_eq!(
        DeclaredLimitation::new(
            canonically_equivalent_nfd_claim,
            "This decomposed spelling must not create a second claim identity.",
        ),
        Err(ReleaseDecisionError::InvalidLimitationClaim),
        "canonically equivalent NFD text must not bypass limitation identity",
    );
}

#[test]
fn limitation_rejects_non_nfc_buyer_consequence() {
    assert_eq!(
        DeclaredLimitation::new(
            "linux_arm64",
            "Cafe\u{301} support is excluded from this profile.",
        ),
        Err(ReleaseDecisionError::InvalidLimitationConsequence),
        "buyer-visible consequences must use one canonical Unicode spelling",
    );
}

#[test]
fn invalid_canonical_text_errors_describe_all_rejected_causes() {
    let claim_error = DeclaredLimitation::new(
        " linux_arm64",
        "Linux ARM64 is excluded from the support profile.",
    )
    .expect_err("surrounding claim whitespace must remain invalid");
    assert_eq!(
        claim_error.to_string(),
        "declared release limitation claim is not canonical or contains an unsafe presentation character"
    );

    let consequence_error = DeclaredLimitation::new(
        "linux_arm64",
        "Cafe\u{301} support is excluded from this profile.",
    )
    .expect_err("non-NFC consequence text must remain invalid");
    assert_eq!(
        consequence_error.to_string(),
        "declared release limitation consequence is not canonical or contains an unsafe presentation character"
    );
}
