use originweave_core::{
    release_acceptance::ZeroEventSafetyEvidence,
    zero_event_threshold::{
        MAX_SAFETY_EVENT_RATE_PARTS_PER_MILLION, ZeroEventSafetyThreshold,
        ZeroEventSafetyThresholdError, ZeroEventSafetyThresholdOutcome,
    },
};

#[test]
fn zero_event_threshold_requires_both_confidence_and_a_tight_enough_upper_bound()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = ZeroEventSafetyEvidence::new(100, 9_500)?;
    let satisfied = ZeroEventSafetyThreshold::new(29_514, 9_500)?;
    let too_tight = ZeroEventSafetyThreshold::new(29_513, 9_500)?;
    let stronger_confidence_required = ZeroEventSafetyThreshold::new(29_514, 9_900)?;

    assert_eq!(
        satisfied.evaluate(evidence),
        ZeroEventSafetyThresholdOutcome::Satisfied
    );
    assert_eq!(
        too_tight.evaluate(evidence),
        ZeroEventSafetyThresholdOutcome::UpperBoundExceedsThreshold
    );
    assert_eq!(
        stronger_confidence_required.evaluate(evidence),
        ZeroEventSafetyThresholdOutcome::InsufficientConfidence
    );
    Ok(())
}

#[test]
fn exact_fixed_point_boundary_is_satisfied_without_float_rounding_drift()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = ZeroEventSafetyEvidence::new(1, 10)?;
    let exact_boundary = ZeroEventSafetyThreshold::new(1_000, 10)?;
    let one_ppm_tighter = ZeroEventSafetyThreshold::new(999, 10)?;

    assert_eq!(
        exact_boundary.evaluate(evidence),
        ZeroEventSafetyThresholdOutcome::Satisfied
    );
    assert_eq!(
        one_ppm_tighter.evaluate(evidence),
        ZeroEventSafetyThresholdOutcome::UpperBoundExceedsThreshold
    );
    Ok(())
}

#[test]
fn zero_event_threshold_retains_exact_fixed_point_policy_inputs()
-> Result<(), ZeroEventSafetyThresholdError> {
    let threshold = ZeroEventSafetyThreshold::new(2_500, 9_900)?;

    assert_eq!(
        threshold.maximum_upper_event_rate_parts_per_million(),
        2_500
    );
    assert_eq!(threshold.minimum_confidence_basis_points(), 9_900);
    Ok(())
}

#[test]
fn zero_event_threshold_rejects_rates_above_one_and_invalid_confidence() {
    assert_eq!(
        ZeroEventSafetyThreshold::new(MAX_SAFETY_EVENT_RATE_PARTS_PER_MILLION + 1, 9_500),
        Err(ZeroEventSafetyThresholdError::InvalidUpperRatePartsPerMillion)
    );
    assert_eq!(
        ZeroEventSafetyThreshold::new(1_000, 0),
        Err(ZeroEventSafetyThresholdError::InvalidConfidenceBasisPoints)
    );
    assert_eq!(
        ZeroEventSafetyThreshold::new(1_000, 10_000),
        Err(ZeroEventSafetyThresholdError::InvalidConfidenceBasisPoints)
    );
}

#[test]
fn zero_rate_threshold_is_valid_but_finite_trials_remain_inconclusive()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = ZeroEventSafetyEvidence::new(u64::MAX, 9_500)?;
    let threshold = ZeroEventSafetyThreshold::new(0, 9_500)?;

    assert_eq!(
        threshold.evaluate(evidence),
        ZeroEventSafetyThresholdOutcome::UpperBoundExceedsThreshold
    );
    Ok(())
}

#[test]
fn safety_threshold_validation_errors_have_stable_standard_error_contracts() {
    let cases = [
        (
            ZeroEventSafetyThresholdError::InvalidUpperRatePartsPerMillion,
            "zero-event safety upper-rate threshold must be at most 1000000 parts per million",
        ),
        (
            ZeroEventSafetyThresholdError::InvalidConfidenceBasisPoints,
            "zero-event safety threshold confidence must be between 1 and 9999 basis points",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}
