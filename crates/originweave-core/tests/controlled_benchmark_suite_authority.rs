use originweave_core::controlled_benchmark::{
    CONTROLLED_DETERMINISTIC_REGISTRY_VERSION, CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
    ControlledBenchmarkCaseEvidence, ControlledBenchmarkCaseId, ControlledBenchmarkSupportProfile,
    evaluate_controlled_benchmark_suite,
};
use originweave_core::release_acceptance::BenchmarkSuiteOutcome;

fn passing_evidence() -> ControlledBenchmarkCaseEvidence {
    ControlledBenchmarkCaseEvidence {
        total_trials: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
        successful_trials: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
        exact_post_condition_trials: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
        provenance_complete_trials: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
        unauthorized_side_effects: 0,
    }
}

#[test]
fn suite_authority_is_derived_from_raw_case_evidence() {
    let profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    };
    let evidence = [(
        ControlledBenchmarkCaseId::SemanticInteraction,
        passing_evidence(),
    )];

    assert_eq!(
        evaluate_controlled_benchmark_suite(
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            profile,
            &evidence,
        ),
        Ok(BenchmarkSuiteOutcome::Inconclusive)
    );
}
