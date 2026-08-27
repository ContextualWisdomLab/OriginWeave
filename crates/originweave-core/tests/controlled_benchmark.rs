use originweave_core::controlled_benchmark::{
    CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS, ControlledBenchmarkCaseEvidence,
    ControlledBenchmarkError, evaluate_controlled_benchmark_case,
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
fn exactly_one_hundred_clean_runs_pass_the_controlled_case() {
    assert_eq!(
        evaluate_controlled_benchmark_case(passing_evidence()),
        Ok(BenchmarkSuiteOutcome::Passed)
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
        Ok(BenchmarkSuiteOutcome::Inconclusive)
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
            Ok(BenchmarkSuiteOutcome::Failed)
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
            Ok(BenchmarkSuiteOutcome::Failed)
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
