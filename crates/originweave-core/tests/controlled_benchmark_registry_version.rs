use originweave_core::controlled_benchmark::{
    CONTROLLED_DETERMINISTIC_REGISTRY_VERSION, CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
    ControlledBenchmarkCaseId, ControlledBenchmarkCaseTrials, ControlledBenchmarkSuiteError,
    ControlledBenchmarkSupportProfile, ControlledBenchmarkTrialEvidence,
    evaluate_controlled_benchmark_suite,
};

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
fn suite_release_authority_requires_the_exact_registry_version() {
    let profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    };
    let evidence = [ControlledBenchmarkCaseTrials {
        case_id: ControlledBenchmarkCaseId::SemanticInteraction,
        trials: passing_trials(),
    }];

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
