use originweave_core::controlled_benchmark::{
    CONTROLLED_DETERMINISTIC_REGISTRY_VERSION, CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
    ControlledBenchmarkCaseEvidence, ControlledBenchmarkCaseId, ControlledBenchmarkCaseOutcome,
    ControlledBenchmarkError, ControlledBenchmarkSuiteError, ControlledBenchmarkSupportProfile,
    evaluate_controlled_benchmark_case, evaluate_controlled_benchmark_suite,
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

fn required_suite_evidence(
    profile: ControlledBenchmarkSupportProfile,
) -> Vec<(ControlledBenchmarkCaseId, ControlledBenchmarkCaseOutcome)> {
    ControlledBenchmarkCaseId::ALL
        .into_iter()
        .filter(|case_id| case_id.required_for(profile))
        .map(|case_id| (case_id, ControlledBenchmarkCaseOutcome::Passed))
        .collect()
}

#[test]
fn exactly_one_hundred_clean_runs_pass_the_controlled_case() {
    assert_eq!(
        evaluate_controlled_benchmark_case(passing_evidence()),
        Ok(ControlledBenchmarkCaseOutcome::Passed)
    );
}

#[test]
fn fewer_than_one_hundred_runs_are_inconclusive_even_when_all_observed_runs_pass() {
    let trials = CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS - 1;
    let evidence = ControlledBenchmarkCaseEvidence {
        total_trials: trials,
        successful_trials: trials,
        exact_post_condition_trials: trials,
        provenance_complete_trials: trials,
        unauthorized_side_effects: 0,
    };

    assert_eq!(
        evaluate_controlled_benchmark_case(evidence),
        Ok(ControlledBenchmarkCaseOutcome::Inconclusive)
    );
}

#[test]
fn known_observed_failure_fails_even_before_trial_budget_completes() {
    let trials = CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS - 1;
    let clean_partial = ControlledBenchmarkCaseEvidence {
        total_trials: trials,
        successful_trials: trials,
        exact_post_condition_trials: trials,
        provenance_complete_trials: trials,
        unauthorized_side_effects: 0,
    };

    for evidence in [
        ControlledBenchmarkCaseEvidence {
            successful_trials: trials - 1,
            ..clean_partial
        },
        ControlledBenchmarkCaseEvidence {
            exact_post_condition_trials: trials - 1,
            ..clean_partial
        },
        ControlledBenchmarkCaseEvidence {
            provenance_complete_trials: trials - 1,
            ..clean_partial
        },
        ControlledBenchmarkCaseEvidence {
            unauthorized_side_effects: 1,
            ..clean_partial
        },
    ] {
        assert_eq!(
            evaluate_controlled_benchmark_case(evidence),
            Ok(ControlledBenchmarkCaseOutcome::Failed)
        );
    }
}

#[test]
fn more_than_the_canonical_trial_count_is_rejected_as_noncanonical_evidence() {
    let mut evidence = passing_evidence();
    evidence.total_trials += 1;

    assert_eq!(
        evaluate_controlled_benchmark_case(evidence),
        Err(ControlledBenchmarkError::NonCanonicalTrialCount {
            observed: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS + 1,
            maximum: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
        })
    );
}

#[test]
fn counters_cannot_claim_more_observations_than_the_total_trial_count() {
    let total_trials = CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS - 1;
    let impossible = CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS;

    for (evidence, expected_counter) in [
        (
            ControlledBenchmarkCaseEvidence {
                total_trials,
                successful_trials: impossible,
                exact_post_condition_trials: total_trials,
                provenance_complete_trials: total_trials,
                unauthorized_side_effects: 0,
            },
            "successful_trials",
        ),
        (
            ControlledBenchmarkCaseEvidence {
                total_trials,
                successful_trials: total_trials,
                exact_post_condition_trials: impossible,
                provenance_complete_trials: total_trials,
                unauthorized_side_effects: 0,
            },
            "exact_post_condition_trials",
        ),
        (
            ControlledBenchmarkCaseEvidence {
                total_trials,
                successful_trials: total_trials,
                exact_post_condition_trials: total_trials,
                provenance_complete_trials: impossible,
                unauthorized_side_effects: 0,
            },
            "provenance_complete_trials",
        ),
    ] {
        assert_eq!(
            evaluate_controlled_benchmark_case(evidence),
            Err(ControlledBenchmarkError::CounterExceedsTrialCount {
                counter: expected_counter,
                observed: impossible,
                total_trials,
            })
        );
    }
}

#[test]
fn any_known_controlled_threshold_failure_fails_the_case() {
    let required = CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS;

    for evidence in [
        ControlledBenchmarkCaseEvidence {
            successful_trials: required - 1,
            ..passing_evidence()
        },
        ControlledBenchmarkCaseEvidence {
            exact_post_condition_trials: required - 1,
            ..passing_evidence()
        },
        ControlledBenchmarkCaseEvidence {
            provenance_complete_trials: required - 1,
            ..passing_evidence()
        },
        ControlledBenchmarkCaseEvidence {
            unauthorized_side_effects: 1,
            ..passing_evidence()
        },
    ] {
        assert_eq!(
            evaluate_controlled_benchmark_case(evidence),
            Ok(ControlledBenchmarkCaseOutcome::Failed)
        );
    }
}

#[test]
fn benchmark_errors_have_deterministic_standard_error_contracts() {
    let cases = [
        (
            ControlledBenchmarkError::NonCanonicalTrialCount {
                observed: 101,
                maximum: 100,
            },
            "controlled benchmark evidence has 101 trials; the canonical maximum is 100",
        ),
        (
            ControlledBenchmarkError::CounterExceedsTrialCount {
                counter: "successful_trials",
                observed: 100,
                total_trials: 99,
            },
            "controlled benchmark counter successful_trials has 100 observations but only 99 total trials",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}

#[test]
fn controlled_registry_has_versioned_stable_case_identities() {
    assert_eq!(
        CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
        "controlled-deterministic-v1"
    );
    assert_eq!(
        ControlledBenchmarkCaseId::ALL.map(ControlledBenchmarkCaseId::as_str),
        [
            "semantic_interaction",
            "same_document_post_condition",
            "navigation_post_condition",
            "dom_accessibility_extraction",
            "json_ld_extraction",
            "table_extraction",
            "bounded_network_extraction",
            "iframe_interaction",
            "shadow_dom_interaction",
            "approved_download",
            "approved_upload",
            "approval_required_reversible_action",
            "secret_handle_form_fill",
            "redirect_origin_transition",
            "dynamic_mutation_stale_node",
            "session_checkpoint_cancel_resume",
            "browser_crash_cleanup",
            "warc_prov_replay",
            "manifest_v3_isolation",
            "native_messaging_isolation",
        ]
    );
}

#[test]
fn base_profile_requires_every_nonconditional_case_and_rejects_extra_conditional_evidence() {
    let profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    };
    let evidence = required_suite_evidence(profile);

    assert_eq!(
        evaluate_controlled_benchmark_suite(profile, &evidence),
        Ok(BenchmarkSuiteOutcome::Passed)
    );

    for conditional in [
        ControlledBenchmarkCaseId::ManifestV3Isolation,
        ControlledBenchmarkCaseId::NativeMessagingIsolation,
    ] {
        let mut with_unclaimed_case = evidence.clone();
        with_unclaimed_case.push((conditional, ControlledBenchmarkCaseOutcome::Passed));
        assert_eq!(
            evaluate_controlled_benchmark_suite(profile, &with_unclaimed_case),
            Err(ControlledBenchmarkSuiteError::UnexpectedConditionalCase {
                case_id: conditional,
            })
        );
    }
}

#[test]
fn declared_optional_surfaces_become_required_suite_evidence() {
    let profile = ControlledBenchmarkSupportProfile {
        manifest_v3: true,
        native_messaging: true,
    };
    let mut evidence = required_suite_evidence(profile);

    assert_eq!(
        evaluate_controlled_benchmark_suite(profile, &evidence),
        Ok(BenchmarkSuiteOutcome::Passed)
    );

    evidence.retain(|(case_id, _)| *case_id != ControlledBenchmarkCaseId::ManifestV3Isolation);
    assert_eq!(
        evaluate_controlled_benchmark_suite(profile, &evidence),
        Ok(BenchmarkSuiteOutcome::Inconclusive)
    );
}

#[test]
fn complete_registry_never_promotes_failed_or_inconclusive_case_evidence() {
    let profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    };

    for (case_outcome, expected_suite_outcome) in [
        (
            ControlledBenchmarkCaseOutcome::Failed,
            BenchmarkSuiteOutcome::Failed,
        ),
        (
            ControlledBenchmarkCaseOutcome::Inconclusive,
            BenchmarkSuiteOutcome::Inconclusive,
        ),
    ] {
        let mut evidence = required_suite_evidence(profile);
        evidence[0].1 = case_outcome;
        assert_eq!(
            evaluate_controlled_benchmark_suite(profile, &evidence),
            Ok(expected_suite_outcome)
        );
    }
}

#[test]
fn duplicate_case_evidence_fails_closed_before_suite_outcome_is_computed() {
    let profile = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    };
    let mut evidence = required_suite_evidence(profile);
    let duplicate = evidence[0];
    evidence.push(duplicate);

    assert_eq!(
        evaluate_controlled_benchmark_suite(profile, &evidence),
        Err(ControlledBenchmarkSuiteError::DuplicateCase {
            case_id: duplicate.0,
        })
    );
}

#[test]
fn support_profile_marks_only_declared_conditional_cases_as_required() {
    let base = ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    };
    let all = ControlledBenchmarkSupportProfile {
        manifest_v3: true,
        native_messaging: true,
    };

    for case_id in ControlledBenchmarkCaseId::ALL {
        if matches!(
            case_id,
            ControlledBenchmarkCaseId::ManifestV3Isolation
                | ControlledBenchmarkCaseId::NativeMessagingIsolation
        ) {
            assert!(!case_id.required_for(base));
        } else {
            assert!(case_id.required_for(base));
        }
        assert!(case_id.required_for(all));
    }
}

#[test]
fn suite_errors_have_deterministic_standard_error_contracts() {
    let cases = [
        (
            ControlledBenchmarkSuiteError::DuplicateCase {
                case_id: ControlledBenchmarkCaseId::SemanticInteraction,
            },
            "controlled benchmark suite contains duplicate case semantic_interaction",
        ),
        (
            ControlledBenchmarkSuiteError::UnexpectedConditionalCase {
                case_id: ControlledBenchmarkCaseId::ManifestV3Isolation,
            },
            "controlled benchmark suite contains manifest_v3_isolation evidence outside the declared support profile",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}
