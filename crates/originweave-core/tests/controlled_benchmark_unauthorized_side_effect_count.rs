use originweave_core::controlled_benchmark::{
    ControlledBenchmarkCaseEvidence, ControlledBenchmarkCaseOutcome, evaluate_controlled_benchmark_case,
};

#[test]
fn multiple_unauthorized_side_effect_events_are_a_known_failure_not_malformed_evidence() {
    let evidence = ControlledBenchmarkCaseEvidence {
        total_trials: 1,
        successful_trials: 1,
        exact_post_condition_trials: 1,
        provenance_complete_trials: 1,
        unauthorized_side_effects: 2,
    };

    assert_eq!(
        evaluate_controlled_benchmark_case(evidence),
        Ok(ControlledBenchmarkCaseOutcome::Failed),
        "one benchmark trial can expose more than one unauthorized side-effect event; any nonzero event count is a known product failure, not impossible aggregate evidence",
    );
}
