use originweave_core::release_acceptance::{
    BenchmarkSuite, BenchmarkSuiteOutcome, DeclaredLimitation, MAX_DECLARED_RELEASE_LIMITATIONS,
    ReleaseDecision, ReleaseDecisionError, ZeroEventSafetyEvidence, ZeroEventSafetyMetric,
    ZeroEventSafetyObservation, decide_release_with_zero_event_safety,
};

fn passing_suites() -> [(BenchmarkSuite, BenchmarkSuiteOutcome); 5] {
    BenchmarkSuite::ALL.map(|suite| (suite, BenchmarkSuiteOutcome::Passed))
}

fn declared_limitation(
    claim: impl Into<String>,
) -> Result<DeclaredLimitation, ReleaseDecisionError> {
    DeclaredLimitation::new(
        claim,
        "This profile is excluded from the declared support profile.",
    )
}

#[test]
fn release_report_retains_named_zero_event_safety_evidence() -> Result<(), ReleaseDecisionError> {
    let observation = ZeroEventSafetyObservation::new(
        ZeroEventSafetyMetric::UnauthorizedAction,
        ZeroEventSafetyEvidence::new(10_000, 9_500)?,
    );

    let report = decide_release_with_zero_event_safety(passing_suites(), &[], &[observation])?;

    assert_eq!(report.decision(), ReleaseDecision::Accepted);
    assert_eq!(report.zero_event_safety_observations(), &[observation]);
    assert_eq!(
        report.zero_event_safety_observations()[0].metric().as_str(),
        "unauthorized_action_rate"
    );
    assert_eq!(
        report.zero_event_safety_observations()[0]
            .evidence()
            .trial_count(),
        10_000
    );
    assert!(
        report.zero_event_safety_observations()[0]
            .evidence()
            .upper_event_rate()
            > 0.0
    );
    Ok(())
}

#[test]
fn zero_event_safety_observations_are_canonicalized_by_metric() -> Result<(), ReleaseDecisionError>
{
    let authority_escalation = ZeroEventSafetyObservation::new(
        ZeroEventSafetyMetric::AuthorityEscalation,
        ZeroEventSafetyEvidence::new(2_000, 9_900)?,
    );
    let stale_authority = ZeroEventSafetyObservation::new(
        ZeroEventSafetyMetric::StaleAuthorityAcceptance,
        ZeroEventSafetyEvidence::new(500, 9_500)?,
    );
    let prompt_injection = ZeroEventSafetyObservation::new(
        ZeroEventSafetyMetric::PromptInjectionSuccess,
        ZeroEventSafetyEvidence::new(1_000, 9_900)?,
    );

    let report = decide_release_with_zero_event_safety(
        passing_suites(),
        &[],
        &[authority_escalation, stale_authority, prompt_injection],
    )?;

    assert_eq!(
        report
            .zero_event_safety_observations()
            .iter()
            .map(|observation| observation.metric())
            .collect::<Vec<_>>(),
        vec![
            ZeroEventSafetyMetric::PromptInjectionSuccess,
            ZeroEventSafetyMetric::StaleAuthorityAcceptance,
            ZeroEventSafetyMetric::AuthorityEscalation,
        ]
    );
    Ok(())
}

#[test]
fn duplicate_zero_event_metric_fails_closed() -> Result<(), ReleaseDecisionError> {
    let first = ZeroEventSafetyObservation::new(
        ZeroEventSafetyMetric::ProtectedValueDisclosure,
        ZeroEventSafetyEvidence::new(100, 9_500)?,
    );
    let duplicate = ZeroEventSafetyObservation::new(
        ZeroEventSafetyMetric::ProtectedValueDisclosure,
        ZeroEventSafetyEvidence::new(1_000, 9_900)?,
    );
    let duplicate_error = ReleaseDecisionError::DuplicateZeroEventSafetyMetric(
        ZeroEventSafetyMetric::ProtectedValueDisclosure,
    );

    assert_eq!(
        decide_release_with_zero_event_safety(passing_suites(), &[], &[first, duplicate]),
        Err(duplicate_error)
    );
    assert_eq!(
        duplicate_error.to_string(),
        "benchmark release evidence contains duplicate zero-event safety metric: protected_value_disclosure_rate"
    );
    Ok(())
}

#[test]
fn zero_event_entrypoint_bounds_declared_limitations_before_cloning()
-> Result<(), ReleaseDecisionError> {
    let maximum = (0..MAX_DECLARED_RELEASE_LIMITATIONS)
        .map(|index| declared_limitation(format!("unsupported_profile_{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let report = decide_release_with_zero_event_safety(passing_suites(), &maximum, &[])?;

    assert_eq!(
        report.decision(),
        ReleaseDecision::AcceptedWithDeclaredLimitations
    );
    assert_eq!(
        report.declared_limitations().len(),
        MAX_DECLARED_RELEASE_LIMITATIONS
    );

    let too_many = (0..=MAX_DECLARED_RELEASE_LIMITATIONS)
        .map(|index| declared_limitation(format!("unsupported_profile_{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        decide_release_with_zero_event_safety(passing_suites(), &too_many, &[]),
        Err(ReleaseDecisionError::TooManyDeclaredLimitations)
    );
    Ok(())
}

#[test]
fn zero_event_entrypoint_rejects_duplicate_limitation_claims() -> Result<(), ReleaseDecisionError> {
    let first = declared_limitation("linux_arm64")?;
    let duplicate = declared_limitation("linux_arm64")?;

    assert_eq!(
        decide_release_with_zero_event_safety(passing_suites(), &[first, duplicate], &[]),
        Err(ReleaseDecisionError::DuplicateLimitationClaim)
    );
    Ok(())
}

#[test]
fn safety_metric_identifiers_cover_release_zero_event_claims() {
    let cases = [
        (
            ZeroEventSafetyMetric::UnauthorizedAction,
            "unauthorized_action_rate",
        ),
        (
            ZeroEventSafetyMetric::PromptInjectionSuccess,
            "prompt_injection_success_rate",
        ),
        (
            ZeroEventSafetyMetric::StaleAuthorityAcceptance,
            "stale_authority_acceptance_rate",
        ),
        (
            ZeroEventSafetyMetric::ProtectedValueDisclosure,
            "protected_value_disclosure_rate",
        ),
        (
            ZeroEventSafetyMetric::AuthorityEscalation,
            "authority_escalation_rate",
        ),
    ];

    for (metric, expected_identifier) in cases {
        assert_eq!(metric.as_str(), expected_identifier);
    }
}

#[test]
fn safety_metric_registry_is_the_canonical_complete_order() {
    assert_eq!(
        ZeroEventSafetyMetric::ALL,
        [
            ZeroEventSafetyMetric::UnauthorizedAction,
            ZeroEventSafetyMetric::PromptInjectionSuccess,
            ZeroEventSafetyMetric::StaleAuthorityAcceptance,
            ZeroEventSafetyMetric::ProtectedValueDisclosure,
            ZeroEventSafetyMetric::AuthorityEscalation,
        ]
    );
    assert_eq!(
        ZeroEventSafetyMetric::ALL.map(ZeroEventSafetyMetric::as_str),
        [
            "unauthorized_action_rate",
            "prompt_injection_success_rate",
            "stale_authority_acceptance_rate",
            "protected_value_disclosure_rate",
            "authority_escalation_rate",
        ]
    );
}
