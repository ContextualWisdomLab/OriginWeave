use originweave_core::release_acceptance::{
    BenchmarkSuite, BenchmarkSuiteOutcome, DeclaredLimitation, MAX_DECLARED_RELEASE_LIMITATIONS,
    MAX_RELEASE_LIMITATION_TEXT_BYTES, ReleaseDecision, ReleaseDecisionError, decide_release,
};

fn passing_results() -> Vec<(BenchmarkSuite, BenchmarkSuiteOutcome)> {
    BenchmarkSuite::ALL
        .into_iter()
        .map(|suite| (suite, BenchmarkSuiteOutcome::Passed))
        .collect()
}

#[test]
fn limitation_metadata_enforces_exact_utf8_byte_budget() -> Result<(), ReleaseDecisionError> {
    let maximum_claim = "c".repeat(MAX_RELEASE_LIMITATION_TEXT_BYTES);
    let maximum_consequence = "x".repeat(MAX_RELEASE_LIMITATION_TEXT_BYTES);
    let limitation = DeclaredLimitation::new(
        maximum_claim.as_str(),
        maximum_consequence.as_str(),
    )?;

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
fn release_report_bounds_declared_limitation_count_before_cloning()
-> Result<(), ReleaseDecisionError> {
    let limitation = DeclaredLimitation::new(
        "linux_arm64",
        "Linux ARM64 is outside the declared support profile.",
    )?;
    let maximum = vec![limitation.clone(); MAX_DECLARED_RELEASE_LIMITATIONS];
    let report = decide_release(passing_results(), &maximum)?;

    assert_eq!(
        report.decision(),
        ReleaseDecision::AcceptedWithDeclaredLimitations
    );
    assert_eq!(
        report.declared_limitations().len(),
        MAX_DECLARED_RELEASE_LIMITATIONS
    );

    let too_many = vec![limitation; MAX_DECLARED_RELEASE_LIMITATIONS + 1];
    assert_eq!(
        decide_release(passing_results(), &too_many),
        Err(ReleaseDecisionError::TooManyDeclaredLimitations)
    );
    Ok(())
}

#[test]
fn resource_bounds_vector_iterator_preserves_every_release_decision_branch()
-> Result<(), ReleaseDecisionError> {
    assert_eq!(
        decide_release(passing_results(), &[])?.decision(),
        ReleaseDecision::Accepted
    );

    let mut failed = passing_results();
    failed[0].1 = BenchmarkSuiteOutcome::Failed;
    assert_eq!(
        decide_release(failed, &[])?.decision(),
        ReleaseDecision::Rejected
    );

    let mut inconclusive = passing_results();
    inconclusive[0].1 = BenchmarkSuiteOutcome::Inconclusive;
    assert_eq!(
        decide_release(inconclusive, &[])?.decision(),
        ReleaseDecision::Inconclusive
    );

    let mut missing = passing_results();
    assert!(missing.pop().is_some());
    assert_eq!(
        decide_release(missing, &[])?.decision(),
        ReleaseDecision::Inconclusive
    );

    assert_eq!(
        decide_release(
            vec![
                (
                    BenchmarkSuite::ControlledDeterministic,
                    BenchmarkSuiteOutcome::Passed,
                ),
                (
                    BenchmarkSuite::ControlledDeterministic,
                    BenchmarkSuiteOutcome::Passed,
                ),
            ],
            &[],
        ),
        Err(ReleaseDecisionError::DuplicateSuite(
            BenchmarkSuite::ControlledDeterministic
        ))
    );
    Ok(())
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
