use originweave_core::release_acceptance::{
    BenchmarkSuite, BenchmarkSuiteOutcome, ReleaseDecision, ReleaseDecisionError,
    ZeroEventSafetyEvidence, ZeroEventSafetyMetric, ZeroEventSafetyObservation,
    decide_release_with_zero_event_safety,
};

fn passing_suites() -> [(BenchmarkSuite, BenchmarkSuiteOutcome); 5] {
    BenchmarkSuite::ALL.map(|suite| (suite, BenchmarkSuiteOutcome::Passed))
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
