use originweave_core::release_acceptance::{
    DeclaredLimitation, MAX_RELEASE_LIMITATION_TEXT_BYTES, ReleaseDecisionError,
};

#[test]
fn international_claim_preserves_exact_1024_bytes_and_rejects_1025()
-> Result<(), ReleaseDecisionError> {
    assert_eq!(MAX_RELEASE_LIMITATION_TEXT_BYTES, 1024);
    let exact = format!("{}x", "가".repeat(341));
    assert_eq!(exact.len(), 1024);
    let consequence = "지원 범위를 설명하는 구매자 안내";
    let limitation = DeclaredLimitation::new(&exact, consequence)?;
    assert_eq!(limitation.unsupported_claim(), exact);
    assert_eq!(limitation.buyer_consequence(), consequence);

    let oversized = format!("{exact}x");
    assert_eq!(oversized.len(), 1025);
    assert_eq!(
        DeclaredLimitation::new(&oversized, consequence),
        Err(ReleaseDecisionError::LimitationClaimTooLong),
    );
    Ok(())
}

#[test]
fn international_consequence_preserves_exact_1024_bytes_and_rejects_1025()
-> Result<(), ReleaseDecisionError> {
    assert_eq!(MAX_RELEASE_LIMITATION_TEXT_BYTES, 1024);
    let exact = format!("{}x", "가".repeat(341));
    assert_eq!(exact.len(), 1024);
    let claim = "bounded_claim";
    let limitation = DeclaredLimitation::new(claim, &exact)?;
    assert_eq!(limitation.unsupported_claim(), claim);
    assert_eq!(limitation.buyer_consequence(), exact);

    let oversized = format!("{exact}x");
    assert_eq!(oversized.len(), 1025);
    assert_eq!(
        DeclaredLimitation::new(claim, &oversized),
        Err(ReleaseDecisionError::LimitationConsequenceTooLong),
    );
    Ok(())
}
