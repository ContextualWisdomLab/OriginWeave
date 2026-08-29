//! Fail-closed evaluation of declared zero-observed-event safety requirements.
//!
//! This gate consumes explicit per-metric thresholds and retained zero-event observations. Missing
//! or statistically insufficient evidence is `Inconclusive`; it is never promoted to product
//! success or converted into a known product failure. The gate does not itself grant release
//! authority.

use std::{collections::BTreeMap, fmt};

use crate::{
    release_acceptance::{
        MAX_ZERO_EVENT_SAFETY_METRICS, ZeroEventSafetyMetric, ZeroEventSafetyObservation,
    },
    zero_event_threshold::{ZeroEventSafetyThreshold, ZeroEventSafetyThresholdOutcome},
};

/// One declared zero-event safety threshold bound to its named metric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZeroEventSafetyRequirement {
    metric: ZeroEventSafetyMetric,
    threshold: ZeroEventSafetyThreshold,
}

impl ZeroEventSafetyRequirement {
    /// Bind one named safety metric to an explicit fixed-point statistical threshold.
    #[must_use]
    pub const fn new(metric: ZeroEventSafetyMetric, threshold: ZeroEventSafetyThreshold) -> Self {
        Self { metric, threshold }
    }
}

/// Release-neutral result of the zero-event statistical safety gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroEventSafetyGateDecision {
    /// Every declared metric had evidence satisfying its explicit threshold.
    Satisfied,
    /// At least one required metric was missing or statistically insufficient.
    Inconclusive,
}

/// Exact reason one declared zero-event safety requirement did not satisfy the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZeroEventSafetyGateFailure {
    /// No retained zero-event observation exists for this required metric.
    MissingObservation(ZeroEventSafetyMetric),
    /// The observation confidence is below the required one-sided confidence.
    InsufficientConfidence(ZeroEventSafetyMetric),
    /// The observation's one-sided upper event-rate bound is above the permitted maximum.
    UpperBoundExceedsThreshold(ZeroEventSafetyMetric),
}

/// Deterministic report for a declared zero-event statistical safety gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZeroEventSafetyGateReport {
    decision: ZeroEventSafetyGateDecision,
    failures: Vec<ZeroEventSafetyGateFailure>,
}

impl ZeroEventSafetyGateReport {
    /// Return whether every declared statistical requirement was satisfied.
    #[must_use]
    pub const fn decision(&self) -> ZeroEventSafetyGateDecision {
        self.decision
    }

    /// Return canonical metric-ordered reasons that kept the gate inconclusive.
    #[must_use]
    pub fn failures(&self) -> &[ZeroEventSafetyGateFailure] {
        &self.failures
    }
}

/// Fail-closed input error for zero-event safety-gate evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroEventSafetyGateError {
    /// No metric thresholds were declared, which would otherwise create a vacuous passing gate.
    MissingRequirements,
    /// More metric thresholds were supplied than the fixed zero-event safety metric budget.
    TooManyRequirements,
    /// More retained observations were supplied than the fixed zero-event safety metric budget.
    TooManyObservations,
    /// The same metric appeared more than once in the declared threshold policy.
    DuplicateRequirement(ZeroEventSafetyMetric),
    /// The same metric appeared more than once in retained zero-event observations.
    DuplicateObservation(ZeroEventSafetyMetric),
}

impl fmt::Display for ZeroEventSafetyGateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequirements => formatter.write_str(
                "zero-event safety gate requires at least one declared metric threshold",
            ),
            Self::TooManyRequirements => {
                formatter.write_str("zero-event safety gate contains too many requirements")
            }
            Self::TooManyObservations => {
                formatter.write_str("zero-event safety gate contains too many observations")
            }
            Self::DuplicateRequirement(metric) => write!(
                formatter,
                "zero-event safety gate contains duplicate requirement: {}",
                metric.as_str()
            ),
            Self::DuplicateObservation(metric) => write!(
                formatter,
                "zero-event safety gate contains duplicate observation: {}",
                metric.as_str()
            ),
        }
    }
}

impl std::error::Error for ZeroEventSafetyGateError {}

/// Evaluate explicit zero-event safety requirements against retained observations.
///
/// Duplicate policy or evidence entries are rejected. Requirements are evaluated in the metric
/// enum's canonical order so report ordering cannot depend on caller input order. Missing evidence,
/// insufficient confidence, and an excessive upper confidence bound all remain `Inconclusive`.
pub fn evaluate_zero_event_safety_gate(
    requirements: &[ZeroEventSafetyRequirement],
    observations: &[ZeroEventSafetyObservation],
) -> Result<ZeroEventSafetyGateReport, ZeroEventSafetyGateError> {
    if requirements.is_empty() {
        return Err(ZeroEventSafetyGateError::MissingRequirements);
    }
    if requirements.len() > MAX_ZERO_EVENT_SAFETY_METRICS {
        return Err(ZeroEventSafetyGateError::TooManyRequirements);
    }
    if observations.len() > MAX_ZERO_EVENT_SAFETY_METRICS {
        return Err(ZeroEventSafetyGateError::TooManyObservations);
    }

    let mut thresholds_by_metric = BTreeMap::new();
    for requirement in requirements {
        if thresholds_by_metric
            .insert(requirement.metric, requirement.threshold)
            .is_some()
        {
            return Err(ZeroEventSafetyGateError::DuplicateRequirement(
                requirement.metric,
            ));
        }
    }

    let mut observations_by_metric = BTreeMap::new();
    for observation in observations {
        if observations_by_metric
            .insert(observation.metric(), observation.evidence())
            .is_some()
        {
            return Err(ZeroEventSafetyGateError::DuplicateObservation(
                observation.metric(),
            ));
        }
    }

    let mut failures = Vec::with_capacity(thresholds_by_metric.len());
    for (metric, threshold) in thresholds_by_metric {
        let Some(evidence) = observations_by_metric.get(&metric).copied() else {
            failures.push(ZeroEventSafetyGateFailure::MissingObservation(metric));
            continue;
        };
        match threshold.evaluate(evidence) {
            ZeroEventSafetyThresholdOutcome::Satisfied => {}
            ZeroEventSafetyThresholdOutcome::InsufficientConfidence => {
                failures.push(ZeroEventSafetyGateFailure::InsufficientConfidence(metric))
            }
            ZeroEventSafetyThresholdOutcome::UpperBoundExceedsThreshold => failures.push(
                ZeroEventSafetyGateFailure::UpperBoundExceedsThreshold(metric),
            ),
        }
    }

    let decision = if failures.is_empty() {
        ZeroEventSafetyGateDecision::Satisfied
    } else {
        ZeroEventSafetyGateDecision::Inconclusive
    };
    Ok(ZeroEventSafetyGateReport { decision, failures })
}
