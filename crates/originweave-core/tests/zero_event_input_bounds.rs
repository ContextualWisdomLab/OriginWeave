use originweave_core::{
    release_acceptance::{
        BenchmarkSuite, BenchmarkSuiteEvidence, BenchmarkSuiteOutcome, ReleaseDecisionError,
        ZeroEventSafetyEvidence, ZeroEventSafetyMetric, ZeroEventSafetyObservation,
        MAX_ZERO_EVENT_SAFETY_METRICS, decide_release_with_classified_benchmark_evidence,
        decide_release_with_zero_event_safety,
    },
    zero_event_safety_gate::{
        ZeroEventSafetyGateDecision, ZeroEventSafetyGateError, ZeroEventSafetyRequirement,
        evaluate_zero_event_safety_gate,
    },
    zero_event_threshold::ZeroEventSafetyThreshold,
};

fn passing_suites() -> [(BenchmarkSuite, BenchmarkSuiteOutcome); 5] {
    BenchmarkSuite::ALL.map(|suite| (suite, BenchmarkSuiteOutcome::Passed))
}

fn observations() -> Result<[ZeroEventSafetyObservation; 5], ReleaseDecisionError> {
    let evidence = ZeroEventSafetyEvidence::new(10_000, 9_500)?;
    Ok([
        ZeroEventSafetyObservation::new(ZeroEventSafetyMetric::UnauthorizedAction, evidence),
        ZeroEventSafetyObservation::new(ZeroEventSafetyMetric::PromptInjectionSuccess, evidence),
        ZeroEventSafetyObservation::new(ZeroEventSafetyMetric::StaleAuthorityAcceptance, evidence),
        ZeroEventSafetyObservation::new(ZeroEventSafetyMetric::ProtectedValueDisclosure, evidence),
        ZeroEventSafetyObservation::new(ZeroEventSafetyMetric::AuthorityEscalation, evidence),
    ])
}

fn requirements() -> Result<[ZeroEventSafetyRequirement; 5], Box<dyn std::error::Error>> {
    let threshold = ZeroEventSafetyThreshold::new(1_000_000, 9_500)?;
    Ok([
        ZeroEventSafetyRequirement::new(ZeroEventSafetyMetric::UnauthorizedAction, threshold),
        ZeroEventSafetyRequirement::new(ZeroEventSafetyMetric::PromptInjectionSuccess, threshold),
        ZeroEventSafetyRequirement::new(ZeroEventSafetyMetric::StaleAuthorityAcceptance, threshold),
        ZeroEventSafetyRequirement::new(ZeroEventSafetyMetric::ProtectedValueDisclosure, threshold),
        ZeroEventSafetyRequirement::new(ZeroEventSafetyMetric::AuthorityEscalation, threshold),
    ])
}

#[test]
fn release_entrypoints_bound_zero_event_observations_before_clone_or_sort()
-> Result<(), ReleaseDecisionError> {
    assert_eq!(MAX_ZERO_EVENT_SAFETY_METRICS, 5);
    let maximum = observations()?;

    let compatibility = decide_release_with_zero_event_safety(passing_suites(), &[], &maximum)?;
    assert_eq!(compatibility.zero_event_safety_observations().len(), 5);

    let classified = BenchmarkSuite::ALL.map(BenchmarkSuiteEvidence::Passed);
    let classified_report =
        decide_release_with_classified_benchmark_evidence(classified, &[], &maximum)?;
    assert_eq!(classified_report.zero_event_safety_observations().len(), 5);

    let mut oversized = maximum.to_vec();
    oversized.push(maximum[0]);
    assert_eq!(
        decide_release_with_zero_event_safety(passing_suites(), &[], &oversized),
        Err(ReleaseDecisionError::TooManyZeroEventSafetyObservations)
    );
    let classified = BenchmarkSuite::ALL.map(BenchmarkSuiteEvidence::Passed);
    assert_eq!(
        decide_release_with_classified_benchmark_evidence(classified, &[], &oversized),
        Err(ReleaseDecisionError::TooManyZeroEventSafetyObservations)
    );
    Ok(())
}

#[test]
fn safety_gate_bounds_requirement_and_observation_slices_before_map_population()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(MAX_ZERO_EVENT_SAFETY_METRICS, 5);
    let maximum_requirements = requirements()?;
    let maximum_observations = observations()?;

    let report = evaluate_zero_event_safety_gate(&maximum_requirements, &maximum_observations)?;
    assert_eq!(report.decision(), ZeroEventSafetyGateDecision::Satisfied);

    let mut oversized_requirements = maximum_requirements.to_vec();
    oversized_requirements.push(maximum_requirements[0]);
    assert_eq!(
        evaluate_zero_event_safety_gate(&oversized_requirements, &maximum_observations),
        Err(ZeroEventSafetyGateError::TooManyRequirements)
    );

    let mut oversized_observations = maximum_observations.to_vec();
    oversized_observations.push(maximum_observations[0]);
    assert_eq!(
        evaluate_zero_event_safety_gate(&maximum_requirements, &oversized_observations),
        Err(ZeroEventSafetyGateError::TooManyObservations)
    );
    Ok(())
}

#[test]
fn resource_bound_errors_have_stable_messages_and_no_hidden_sources() {
    let cases = [
        (
            ZeroEventSafetyGateError::TooManyRequirements,
            "zero-event safety gate contains too many requirements",
        ),
        (
            ZeroEventSafetyGateError::TooManyObservations,
            "zero-event safety gate contains too many observations",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }

    let release_error = ReleaseDecisionError::TooManyZeroEventSafetyObservations;
    assert_eq!(
        release_error.to_string(),
        "benchmark release evidence contains too many zero-event safety observations"
    );
    let standard_error: &dyn std::error::Error = &release_error;
    assert!(standard_error.source().is_none());
}
