use originweave_core::release_acceptance::{DeclaredLimitation, ReleaseDecisionError};

#[test]
fn punctuation_only_limitation_claim_does_not_name_an_unsupported_claim() {
    assert_eq!(
        DeclaredLimitation::new("---", "Linux ARM64 is excluded from the support profile."),
        Err(ReleaseDecisionError::InvalidLimitationClaim)
    );
}

#[test]
fn punctuation_only_limitation_consequence_does_not_state_a_buyer_consequence() {
    assert_eq!(
        DeclaredLimitation::new("linux_arm64", "..."),
        Err(ReleaseDecisionError::InvalidLimitationConsequence)
    );
}

#[test]
fn international_alphanumeric_limitation_text_remains_admissible() {
    assert!(
        DeclaredLimitation::new(
            "한국어_운영환경",
            "이 운영환경은 현재 지원 범위에 포함되지 않습니다.",
        )
        .is_ok()
    );
}
