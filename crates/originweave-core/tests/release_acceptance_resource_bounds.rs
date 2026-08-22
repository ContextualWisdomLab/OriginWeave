use originweave_core::release_acceptance::{
    DeclaredLimitation, MAX_RELEASE_LIMITATION_TEXT_BYTES, ReleaseDecisionError,
};

#[test]
fn limitation_metadata_enforces_exact_utf8_byte_budget() -> Result<(), ReleaseDecisionError> {
    let maximum_claim = "c".repeat(MAX_RELEASE_LIMITATION_TEXT_BYTES);
    let maximum_consequence = "x".repeat(MAX_RELEASE_LIMITATION_TEXT_BYTES);
    let limitation = DeclaredLimitation::new(maximum_claim.as_str(), maximum_consequence.as_str())?;

    assert_eq!(limitation.unsupported_claim(), maximum_claim.as_str());
    assert_eq!(limitation.buyer_consequence(), maximum_consequence.as_str());

    let oversized_claim = "c".repeat(MAX_RELEASE_LIMITATION_TEXT_BYTES + 1);
    assert_eq!(
        DeclaredLimitation::new(oversized_claim.as_str(), "bounded buyer consequence"),
        Err(ReleaseDecisionError::LimitationClaimTooLong)
    );

    let oversized_consequence = "x".repeat(MAX_RELEASE_LIMITATION_TEXT_BYTES + 1);
    assert_eq!(
        DeclaredLimitation::new("bounded_claim", oversized_consequence.as_str()),
        Err(ReleaseDecisionError::LimitationConsequenceTooLong)
    );
    Ok(())
}

#[test]
fn limitation_byte_budget_applies_to_international_text() {
    let korean_character = "가";
    let repeated =
        korean_character.repeat(MAX_RELEASE_LIMITATION_TEXT_BYTES / korean_character.len() + 1);
    assert!(repeated.len() > MAX_RELEASE_LIMITATION_TEXT_BYTES);
    assert_eq!(
        DeclaredLimitation::new(repeated.as_str(), "지원 범위를 설명하는 구매자 안내"),
        Err(ReleaseDecisionError::LimitationClaimTooLong)
    );
}

#[test]
fn release_resource_limit_errors_have_deterministic_standard_error_contracts() {
    let cases = [
        (
            ReleaseDecisionError::LimitationClaimTooLong,
            "declared release limitation claim exceeds the byte budget",
        ),
        (
            ReleaseDecisionError::LimitationConsequenceTooLong,
            "declared release limitation consequence exceeds the byte budget",
        ),
        (
            ReleaseDecisionError::TooManyDeclaredLimitations,
            "benchmark release decision contains too many declared limitations",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}
