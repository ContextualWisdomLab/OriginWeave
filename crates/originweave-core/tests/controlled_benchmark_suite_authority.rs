use originweave_core::controlled_benchmark::{
    CONTROLLED_DETERMINISTIC_REGISTRY_VERSION, CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
    ControlledBenchmarkCaseId, ControlledBenchmarkCaseTrials, ControlledBenchmarkSupportProfile,
    ControlledBenchmarkTrialEvidence, evaluate_controlled_benchmark_suite,
};
use originweave_core::release_acceptance::BenchmarkSuiteOutcome;

fn passing_trials() -> Vec<ControlledBenchmarkTrialEvidence> {
    (1..=CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS)
        .map(|trial_ordinal| ControlledBenchmarkTrialEvidence {
            trial_ordinal,
            action_succeeded: true,
            exact_post_condition: true,
            provenance_complete: true,
            unauthorized_side_effects: 0,
        })
        .collect()
}

#[test]
fn suite_authority_is_derived_from_trial_evidence_not_caller_aggregate_counters() {
    let profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    };
    let evidence = [ControlledBenchmarkCaseTrials {
        case_id: ControlledBenchmarkCaseId::SemanticInteraction,
        trials: passing_trials(),
    }];

    assert_eq!(
        evaluate_controlled_benchmark_suite(
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            profile,
            &evidence,
        ),
        Ok(BenchmarkSuiteOutcome::Inconclusive)
    );
}
