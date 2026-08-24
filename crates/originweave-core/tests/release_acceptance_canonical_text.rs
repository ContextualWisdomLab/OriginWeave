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
