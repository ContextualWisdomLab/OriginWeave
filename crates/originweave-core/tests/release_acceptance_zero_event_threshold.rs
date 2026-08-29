use originweave_core::release_acceptance::{
    MAX_SAFETY_EVENT_RATE_PARTS_PER_MILLION, ReleaseDecisionError, ZeroEventSafetyEvidence,
    ZeroEventSafetyThreshold, ZeroEventSafetyThresholdOutcome,
};

#[test]
fn zero_event_threshold_requires_both_confidence_and_a_tight_enough_upper_bound()
-> Result<(), ReleaseDecisionError> {
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
fn zero_event_threshold_retains_exact_fixed_point_policy_inputs() -> Result<(), ReleaseDecisionError>
{
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
        Err(ReleaseDecisionError::InvalidSafetyUpperRatePartsPerMillion)
    );
    assert_eq!(
        ZeroEventSafetyThreshold::new(1_000, 0),
        Err(ReleaseDecisionError::InvalidSafetyConfidenceBasisPoints)
    );
    assert_eq!(
        ZeroEventSafetyThreshold::new(1_000, 10_000),
        Err(ReleaseDecisionError::InvalidSafetyConfidenceBasisPoints)
    );
}

#[test]
fn zero_rate_threshold_is_valid_but_finite_trials_remain_inconclusive()
-> Result<(), ReleaseDecisionError> {
    let evidence = ZeroEventSafetyEvidence::new(u64::MAX, 9_500)?;
    let threshold = ZeroEventSafetyThreshold::new(0, 9_500)?;

    assert_eq!(
        threshold.evaluate(evidence),
        ZeroEventSafetyThresholdOutcome::UpperBoundExceedsThreshold
    );
    Ok(())
}

#[test]
fn safety_threshold_validation_error_has_a_stable_message() {
    let error = ReleaseDecisionError::InvalidSafetyUpperRatePartsPerMillion;

    assert_eq!(
        error.to_string(),
        "zero-event safety upper-rate threshold must be at most 1000000 parts per million"
    );
    let standard_error: &dyn std::error::Error = &error;
    assert!(standard_error.source().is_none());
}
