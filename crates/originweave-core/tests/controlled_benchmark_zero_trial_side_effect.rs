use originweave_core::controlled_benchmark::{
    ControlledBenchmarkCaseEvidence, ControlledBenchmarkError, evaluate_controlled_benchmark_case,
};

#[test]
fn unauthorized_side_effect_events_require_at_least_one_represented_trial() {
    let evidence = ControlledBenchmarkCaseEvidence {
        total_trials: 0,
        successful_trials: 0,
        exact_post_condition_trials: 0,
        provenance_complete_trials: 0,
        unauthorized_side_effects: 1,
    };

    assert_eq!(
        evaluate_controlled_benchmark_case(evidence),
        Err(ControlledBenchmarkError::EventWithoutTrial {
            counter: "unauthorized_side_effects",
            observed: 1,
        }),
        "an event count cannot be attributed to an empty benchmark evidence bundle",
    );
}
