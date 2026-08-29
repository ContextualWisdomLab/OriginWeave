use originweave_core::{
    release_acceptance::{
        ZeroEventSafetyEvidence, ZeroEventSafetyMetric, ZeroEventSafetyObservation,
    },
    zero_event_safety_gate::{
        ZeroEventSafetyGateDecision, ZeroEventSafetyGateError, ZeroEventSafetyGateFailure,
        ZeroEventSafetyRequirement, evaluate_zero_event_safety_gate,
    },
    zero_event_threshold::ZeroEventSafetyThreshold,
};

fn requirement(
    metric: ZeroEventSafetyMetric,
    maximum_upper_event_rate_parts_per_million: u32,
    minimum_confidence_basis_points: u16,
) -> ZeroEventSafetyRequirement {
    ZeroEventSafetyRequirement::new(
        metric,
        ZeroEventSafetyThreshold::new(
            maximum_upper_event_rate_parts_per_million,
            minimum_confidence_basis_points,
        )
        .expect("test threshold must be valid"),
    )
}

fn observation(
    metric: ZeroEventSafetyMetric,
    trial_count: u64,
    confidence_basis_points: u16,
) -> ZeroEventSafetyObservation {
    ZeroEventSafetyObservation::new(
        metric,
        ZeroEventSafetyEvidence::new(trial_count, confidence_basis_points)
            .expect("test evidence must be valid"),
    )
}

#[test]
fn gate_is_satisfied_only_when_every_declared_requirement_is_satisfied() {
    let requirements = [
        requirement(ZeroEventSafetyMetric::UnauthorizedAction, 29_514, 9_500),
        requirement(
            ZeroEventSafetyMetric::ProtectedValueDisclosure,
            29_514,
            9_500,
        ),
    ];
    let observations = [
        observation(ZeroEventSafetyMetric::ProtectedValueDisclosure, 100, 9_500),
        observation(ZeroEventSafetyMetric::UnauthorizedAction, 100, 9_500),
    ];

    let report = evaluate_zero_event_safety_gate(&requirements, &observations)
        .expect("valid complete evidence must evaluate");

    assert_eq!(report.decision(), ZeroEventSafetyGateDecision::Satisfied);
    assert!(report.failures().is_empty());
}

#[test]
fn missing_or_statistically_insufficient_evidence_is_inconclusive() {
    let requirements = [
        requirement(ZeroEventSafetyMetric::UnauthorizedAction, 29_513, 9_500),
        requirement(ZeroEventSafetyMetric::PromptInjectionSuccess, 29_514, 9_900),
        requirement(
            ZeroEventSafetyMetric::ProtectedValueDisclosure,
            29_514,
            9_500,
        ),
    ];
    let observations = [
        observation(ZeroEventSafetyMetric::UnauthorizedAction, 100, 9_500),
        observation(ZeroEventSafetyMetric::PromptInjectionSuccess, 100, 9_500),
    ];

    let report = evaluate_zero_event_safety_gate(&requirements, &observations)
        .expect("valid partial evidence must evaluate fail closed");

    assert_eq!(report.decision(), ZeroEventSafetyGateDecision::Inconclusive);
    assert_eq!(
        report.failures(),
        &[
            ZeroEventSafetyGateFailure::UpperBoundExceedsThreshold(
                ZeroEventSafetyMetric::UnauthorizedAction,
            ),
            ZeroEventSafetyGateFailure::InsufficientConfidence(
                ZeroEventSafetyMetric::PromptInjectionSuccess,
            ),
            ZeroEventSafetyGateFailure::MissingObservation(
                ZeroEventSafetyMetric::ProtectedValueDisclosure,
            ),
        ]
    );
}

#[test]
fn gate_rejects_empty_or_duplicate_policy_and_duplicate_observations() {
    let unauthorized = requirement(ZeroEventSafetyMetric::UnauthorizedAction, 29_514, 9_500);
    let observation = observation(ZeroEventSafetyMetric::UnauthorizedAction, 100, 9_500);

    assert_eq!(
        evaluate_zero_event_safety_gate(&[], &[]),
        Err(ZeroEventSafetyGateError::MissingRequirements)
    );
    assert_eq!(
        evaluate_zero_event_safety_gate(&[unauthorized, unauthorized], &[observation]),
        Err(ZeroEventSafetyGateError::DuplicateRequirement(
            ZeroEventSafetyMetric::UnauthorizedAction,
        ))
    );
    assert_eq!(
        evaluate_zero_event_safety_gate(&[unauthorized], &[observation, observation]),
        Err(ZeroEventSafetyGateError::DuplicateObservation(
            ZeroEventSafetyMetric::UnauthorizedAction,
        ))
    );
}

#[test]
fn gate_errors_have_stable_messages_and_no_hidden_sources() {
    let cases = [
        (
            ZeroEventSafetyGateError::MissingRequirements,
            "zero-event safety gate requires at least one declared metric threshold",
        ),
        (
            ZeroEventSafetyGateError::DuplicateRequirement(
                ZeroEventSafetyMetric::UnauthorizedAction,
            ),
            "zero-event safety gate contains duplicate requirement: unauthorized_action_rate",
        ),
        (
            ZeroEventSafetyGateError::DuplicateObservation(
                ZeroEventSafetyMetric::UnauthorizedAction,
            ),
            "zero-event safety gate contains duplicate observation: unauthorized_action_rate",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}
