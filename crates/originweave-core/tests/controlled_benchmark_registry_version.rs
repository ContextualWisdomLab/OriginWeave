use originweave_core::controlled_benchmark::{
    CONTROLLED_DETERMINISTIC_REGISTRY_VERSION, CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
    ControlledBenchmarkCaseEvidence, ControlledBenchmarkCaseId, ControlledBenchmarkSuiteError,
    ControlledBenchmarkSupportProfile, evaluate_controlled_benchmark_suite,
};

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
fn suite_release_authority_requires_the_exact_registry_version() {
    let profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    };
    let evidence = [(
        ControlledBenchmarkCaseId::SemanticInteraction,
        passing_evidence(),
    )];

    assert_eq!(
        evaluate_controlled_benchmark_suite("controlled-deterministic-v0", profile, &evidence,),
        Err(ControlledBenchmarkSuiteError::RegistryVersionMismatch)
    );
    assert_eq!(
        evaluate_controlled_benchmark_suite(
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            profile,
            &evidence,
        ),
        Ok(originweave_core::release_acceptance::BenchmarkSuiteOutcome::Inconclusive)
    );
}
