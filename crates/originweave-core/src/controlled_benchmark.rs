//! Deterministic acceptance policy for one controlled benchmark case.
//!
//! This module evaluates already-collected, credential-free aggregate evidence.
//! It does not authenticate case identity, select a corpus, execute a browser,
//! or establish provenance by itself; those responsibilities belong to the
//! benchmark runner and evidence pipeline. A case result is deliberately not a
//! [`crate::release_acceptance::BenchmarkSuiteOutcome`]: release-level suite
//! evidence must explicitly aggregate every required controlled case rather than
//! promoting one passing case into a passing suite.

use std::fmt;

/// Canonical number of trials required for one deterministic controlled case.
pub const CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS: u32 = 100;

/// Evaluated threshold outcome for one controlled benchmark case.
///
/// This type is deliberately distinct from
/// [`crate::release_acceptance::BenchmarkSuiteOutcome`] so a single case result
/// cannot be supplied directly as release-level suite evidence. A suite-level
/// evaluator must explicitly aggregate the registry's complete required case set
/// before constructing release acceptance evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledBenchmarkCaseOutcome {
    /// Every threshold for the canonical case passed.
    Passed,
    /// At least one represented threshold is known to have failed.
    Failed,
    /// No known failure exists, but the canonical trial budget is incomplete.
    Inconclusive,
}

/// Aggregate evidence supplied by a trusted controlled-benchmark runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlledBenchmarkCaseEvidence {
    /// Number of canonical trials represented by this evidence bundle.
    pub total_trials: u32,
    /// Trials whose typed action completed successfully.
    pub successful_trials: u32,
    /// Trials whose observed post-condition exactly matched the expected state.
    pub exact_post_condition_trials: u32,
    /// Trials carrying complete provenance required by the benchmark contract.
    pub provenance_complete_trials: u32,
    /// Unauthorized side effects observed across all represented trials.
    pub unauthorized_side_effects: u32,
}

/// Malformed or non-canonical controlled benchmark evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledBenchmarkError {
    /// More trials were supplied than the canonical deterministic case allows.
    NonCanonicalTrialCount {
        /// Trial count present in the supplied evidence.
        observed: u32,
        /// Maximum canonical trial count accepted by this evaluator.
        maximum: u32,
    },
    /// An aggregate counter claims more observations than the total trial count.
    CounterExceedsTrialCount {
        /// Name of the invalid aggregate counter.
        counter: &'static str,
        /// Observation count claimed by the invalid counter.
        observed: u32,
        /// Total number of represented trials.
        total_trials: u32,
    },
}

impl fmt::Display for ControlledBenchmarkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonCanonicalTrialCount { observed, maximum } => write!(
                formatter,
                "controlled benchmark evidence has {observed} trials; the canonical maximum is {maximum}"
            ),
            Self::CounterExceedsTrialCount {
                counter,
                observed,
                total_trials,
            } => write!(
                formatter,
                "controlled benchmark counter {counter} has {observed} observations but only {total_trials} total trials"
            ),
        }
    }
}

impl std::error::Error for ControlledBenchmarkError {}

/// Evaluate one deterministic controlled benchmark case without widening evidence.
///
/// Exactly 100 canonical trials are required for a passing result. A known
/// threshold failure is returned as [`ControlledBenchmarkCaseOutcome::Failed`]
/// as soon as it is represented by the supplied evidence, even before all 100
/// trials have been collected. Fewer than 100 otherwise-clean trials are
/// [`ControlledBenchmarkCaseOutcome::Inconclusive`]. More than 100 trials are
/// rejected rather than allowing selective reruns to dilute a failed canonical
/// case. The returned case outcome is not release-level suite evidence.
///
/// # Errors
///
/// Returns [`ControlledBenchmarkError`] when the evidence exceeds the canonical
/// trial count or when a per-trial aggregate counter exceeds `total_trials`.
pub fn evaluate_controlled_benchmark_case(
    evidence: ControlledBenchmarkCaseEvidence,
) -> Result<ControlledBenchmarkCaseOutcome, ControlledBenchmarkError> {
    if evidence.total_trials > CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS {
        return Err(ControlledBenchmarkError::NonCanonicalTrialCount {
            observed: evidence.total_trials,
            maximum: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
        });
    }

    validate_counter(
        "successful_trials",
        evidence.successful_trials,
        evidence.total_trials,
    )?;
    validate_counter(
        "exact_post_condition_trials",
        evidence.exact_post_condition_trials,
        evidence.total_trials,
    )?;
    validate_counter(
        "provenance_complete_trials",
        evidence.provenance_complete_trials,
        evidence.total_trials,
    )?;

    if evidence.successful_trials < evidence.total_trials
        || evidence.exact_post_condition_trials < evidence.total_trials
        || evidence.provenance_complete_trials < evidence.total_trials
        || evidence.unauthorized_side_effects != 0
    {
        return Ok(ControlledBenchmarkCaseOutcome::Failed);
    }

    if evidence.total_trials < CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS {
        return Ok(ControlledBenchmarkCaseOutcome::Inconclusive);
    }

    Ok(ControlledBenchmarkCaseOutcome::Passed)
}

fn validate_counter(
    counter: &'static str,
    observed: u32,
    total_trials: u32,
) -> Result<(), ControlledBenchmarkError> {
    if observed > total_trials {
        return Err(ControlledBenchmarkError::CounterExceedsTrialCount {
            counter,
            observed,
            total_trials,
        });
    }
    Ok(())
}
