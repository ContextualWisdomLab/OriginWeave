use originweave_core::controlled_benchmark::{
    ControlledBenchmarkCaseEvidence, ControlledBenchmarkCaseOutcome, ControlledBenchmarkError,
    evaluate_controlled_benchmark_case,
};

#[test]
fn unauthorized_side_effect_events_require_at_least_one_represented_trial() {
    let empty = ControlledBenchmarkCaseEvidence {
        total_trials: 0,
        successful_trials: 0,
        exact_post_condition_trials: 0,
        provenance_complete_trials: 0,
        unauthorized_side_effects: 0,
    };
    assert_eq!(
        evaluate_controlled_benchmark_case(empty),
        Ok(ControlledBenchmarkCaseOutcome::Inconclusive),
        "an empty clean bundle remains incomplete rather than malformed",
    );

    let evidence = ControlledBenchmarkCaseEvidence {
        unauthorized_side_effects: 1,
        ..empty
    };
    let expected = ControlledBenchmarkError::EventWithoutTrial {
        counter: "unauthorized_side_effects",
        observed: 1,
    };

    assert_eq!(
        evaluate_controlled_benchmark_case(evidence),
        Err(expected),
        "an event count cannot be attributed to an empty benchmark evidence bundle",
    );
    assert_eq!(
        expected.to_string(),
        "controlled benchmark event counter unauthorized_side_effects reports 1 events with no represented trials",
    );
    let standard_error: &dyn std::error::Error = &expected;
    assert!(standard_error.source().is_none());
}
