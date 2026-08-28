use originweave_core::controlled_benchmark::{
    CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS, ControlledBenchmarkCaseOutcome,
    ControlledBenchmarkTrialAggregationError, ControlledBenchmarkTrialEvidence,
    aggregate_controlled_benchmark_trials, evaluate_controlled_benchmark_case,
};

fn clean_trial(trial_ordinal: u32) -> ControlledBenchmarkTrialEvidence {
    ControlledBenchmarkTrialEvidence {
        trial_ordinal,
        action_succeeded: true,
        exact_post_condition: true,
        provenance_complete: true,
        unauthorized_side_effects: 0,
    }
}

#[test]
fn unique_trial_ordinals_are_aggregated_without_caller_supplied_counters() {
    let trials: Vec<_> = (1..=CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS)
        .map(clean_trial)
        .collect();

    let evidence = aggregate_controlled_benchmark_trials(&trials).expect("canonical trials");
    assert_eq!(
        evidence.total_trials,
        CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS
    );
    assert_eq!(
        evidence.successful_trials,
        CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS
    );
    assert_eq!(
        evidence.exact_post_condition_trials,
        CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS
    );
    assert_eq!(
        evidence.provenance_complete_trials,
        CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS
    );
    assert_eq!(evidence.unauthorized_side_effects, 0);
    assert_eq!(
        evaluate_controlled_benchmark_case(evidence),
        Ok(ControlledBenchmarkCaseOutcome::Passed)
    );
}

#[test]
fn empty_trial_set_aggregates_to_inconclusive_zero_evidence() {
    let evidence =
        aggregate_controlled_benchmark_trials(&[]).expect("empty evidence is incomplete");

    assert_eq!(evidence.total_trials, 0);
    assert_eq!(evidence.successful_trials, 0);
    assert_eq!(evidence.exact_post_condition_trials, 0);
    assert_eq!(evidence.provenance_complete_trials, 0);
    assert_eq!(evidence.unauthorized_side_effects, 0);
    assert_eq!(
        evaluate_controlled_benchmark_case(evidence),
        Ok(ControlledBenchmarkCaseOutcome::Inconclusive)
    );
}

#[test]
fn duplicate_trial_ordinals_fail_closed_before_threshold_evaluation() {
    let duplicate = [clean_trial(1), clean_trial(1)];

    assert_eq!(
        aggregate_controlled_benchmark_trials(&duplicate),
        Err(ControlledBenchmarkTrialAggregationError::DuplicateTrialOrdinal { trial_ordinal: 1 })
    );
}

#[test]
fn trial_ordinals_outside_the_canonical_budget_fail_closed() {
    for observed in [0, CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS + 1] {
        assert_eq!(
            aggregate_controlled_benchmark_trials(&[clean_trial(observed)]),
            Err(
                ControlledBenchmarkTrialAggregationError::InvalidTrialOrdinal {
                    observed,
                    maximum: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
                }
            )
        );
    }
}

#[test]
fn aggregation_preserves_known_failures_and_multi_event_side_effect_counts() {
    let mut failed = clean_trial(1);
    failed.action_succeeded = false;
    failed.exact_post_condition = false;
    failed.provenance_complete = false;
    failed.unauthorized_side_effects = 2;

    let evidence = aggregate_controlled_benchmark_trials(&[failed]).expect("valid trial shape");
    assert_eq!(evidence.total_trials, 1);
    assert_eq!(evidence.successful_trials, 0);
    assert_eq!(evidence.exact_post_condition_trials, 0);
    assert_eq!(evidence.provenance_complete_trials, 0);
    assert_eq!(evidence.unauthorized_side_effects, 2);
    assert_eq!(
        evaluate_controlled_benchmark_case(evidence),
        Ok(ControlledBenchmarkCaseOutcome::Failed)
    );
}

#[test]
fn unauthorized_side_effect_event_count_overflow_is_rejected() {
    let mut first = clean_trial(1);
    first.unauthorized_side_effects = u32::MAX;
    let mut second = clean_trial(2);
    second.unauthorized_side_effects = 1;

    assert_eq!(
        aggregate_controlled_benchmark_trials(&[first, second]),
        Err(ControlledBenchmarkTrialAggregationError::UnauthorizedSideEffectCountOverflow)
    );
}

#[test]
fn aggregation_errors_explain_the_first_causal_boundary() {
    assert_eq!(
        ControlledBenchmarkTrialAggregationError::InvalidTrialOrdinal {
            observed: 0,
            maximum: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
        }
        .to_string(),
        "controlled benchmark trial ordinal 0 is outside the canonical range 1..=100"
    );
    assert_eq!(
        ControlledBenchmarkTrialAggregationError::DuplicateTrialOrdinal { trial_ordinal: 7 }
            .to_string(),
        "controlled benchmark trial ordinal 7 is duplicated"
    );
    assert_eq!(
        ControlledBenchmarkTrialAggregationError::UnauthorizedSideEffectCountOverflow.to_string(),
        "controlled benchmark unauthorized side-effect event count overflowed"
    );
}
