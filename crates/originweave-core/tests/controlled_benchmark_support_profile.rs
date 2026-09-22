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
fn native_messaging_support_cannot_omit_manifest_v3_isolation() {
    let malformed_profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: true,
    };
    let evidence: Vec<_> = ControlledBenchmarkCaseId::ALL
        .into_iter()
        .filter(|case_id| case_id.required_for(malformed_profile))
        .map(|case_id| ControlledBenchmarkCaseTrials {
            case_id,
            trials: passing_trials(),
        })
        .collect();

    let error = ControlledBenchmarkSuiteError::InvalidSupportProfile;
    assert_eq!(
        evaluate_controlled_benchmark_suite(
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            malformed_profile,
            &evidence,
        ),
        Err(error.clone()),
    );
    assert_eq!(
        error.to_string(),
        "controlled benchmark support profile cannot claim native messaging without Manifest V3 extension support"
    );
    let standard_error: &dyn std::error::Error = &error;
    assert!(standard_error.source().is_none());
}
