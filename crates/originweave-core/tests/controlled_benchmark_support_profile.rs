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
fn native_messaging_support_cannot_omit_manifest_v3_isolation() {
    let malformed_profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: true,
    };
    let evidence: Vec<_> = ControlledBenchmarkCaseId::ALL
        .into_iter()
        .filter(|case_id| case_id.required_for(malformed_profile))
        .map(|case_id| (case_id, passing_evidence()))
        .collect();

    let error = ControlledBenchmarkSuiteError::InvalidSupportProfile;
    assert_eq!(
        evaluate_controlled_benchmark_suite(
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            malformed_profile,
            &evidence,
        ),
        Err(error),
    );
    assert_eq!(
        error.to_string(),
        "controlled benchmark support profile cannot claim native messaging without Manifest V3 extension support"
    );
    let standard_error: &dyn std::error::Error = &error;
    assert!(standard_error.source().is_none());
}
