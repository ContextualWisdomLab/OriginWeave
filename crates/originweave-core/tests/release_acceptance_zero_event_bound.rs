use originweave_core::release_acceptance::{ReleaseDecisionError, ZeroEventSafetyEvidence};

#[test]
fn zero_observed_events_report_an_exact_one_sided_binomial_upper_bound()
-> Result<(), ReleaseDecisionError> {
    let evidence = ZeroEventSafetyEvidence::new(100, 9_500)?;

    assert_eq!(evidence.trial_count(), 100);
    assert_eq!(evidence.confidence_basis_points(), 9_500);
    assert!((evidence.upper_event_rate() - 0.029_513_049_607_039_932).abs() < 1.0e-15);
    Ok(())
}

#[test]
fn zero_event_bound_requires_at_least_one_trial() {
    assert_eq!(
        ZeroEventSafetyEvidence::new(0, 9_500),
        Err(ReleaseDecisionError::MissingSafetyTrials)
    );
}

#[test]
fn zero_event_bound_rejects_zero_or_complete_confidence() {
    assert_eq!(
        ZeroEventSafetyEvidence::new(100, 0),
        Err(ReleaseDecisionError::InvalidSafetyConfidenceBasisPoints)
    );
    assert_eq!(
        ZeroEventSafetyEvidence::new(100, 10_000),
        Err(ReleaseDecisionError::InvalidSafetyConfidenceBasisPoints)
    );
}

#[test]
fn more_zero_event_trials_tighten_the_same_confidence_bound()
-> Result<(), ReleaseDecisionError> {
    let one_hundred = ZeroEventSafetyEvidence::new(100, 9_500)?;
    let one_thousand = ZeroEventSafetyEvidence::new(1_000, 9_500)?;

    assert!(one_thousand.upper_event_rate() < one_hundred.upper_event_rate());
    Ok(())
}

#[test]
fn zero_event_bound_errors_have_deterministic_standard_error_contracts() {
    let cases = [
        (
            ReleaseDecisionError::MissingSafetyTrials,
            "zero-event safety evidence requires at least one trial",
        ),
        (
            ReleaseDecisionError::InvalidSafetyConfidenceBasisPoints,
            "zero-event safety confidence must be between 1 and 9999 basis points",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}
