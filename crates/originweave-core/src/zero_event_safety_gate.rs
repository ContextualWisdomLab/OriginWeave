//! Fail-closed evaluation of declared zero-observed-event safety requirements.
//!
//! This gate consumes explicit per-metric thresholds and retained zero-event observations. Missing
//! or statistically insufficient evidence is `Inconclusive`; it is never promoted to product
//! success or converted into a known product failure. The gate does not itself grant release
//! authority. [`decide_commercial_release_with_zero_event_safety`] combines the gate with mandatory
//! benchmark evidence so a threshold miss cannot remain commercially accepted while repository,
//! review, provenance, and operator release authority remain external controls.

use std::{collections::BTreeMap, fmt};

use crate::{
    release_acceptance::{
        BenchmarkSuiteEvidence, DeclaredLimitation, MAX_ZERO_EVENT_SAFETY_METRICS, ReleaseDecision,
        ReleaseDecisionError, ReleaseDecisionReport, ZeroEventSafetyMetric,
        ZeroEventSafetyObservation, decide_release_with_classified_benchmark_evidence,
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

/// Combined benchmark and zero-event evidence used by commercial release acceptance.
///
/// The final decision is still evidence, not permission to merge, tag, publish, or release. A
/// caller must separately satisfy repository governance, authenticated provenance, independent
/// review, operator authorization, and every other mandatory release control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommercialReleaseAcceptanceReport {
    benchmark_report: ReleaseDecisionReport,
    zero_event_safety_gate_report: ZeroEventSafetyGateReport,
    decision: ReleaseDecision,
}

impl CommercialReleaseAcceptanceReport {
    /// Return the fail-closed combined evidence decision.
    #[must_use]
    pub const fn decision(&self) -> ReleaseDecision {
        self.decision
    }

    /// Return the retained mandatory-suite release evidence.
    #[must_use]
    pub const fn benchmark_report(&self) -> &ReleaseDecisionReport {
        &self.benchmark_report
    }

    /// Return the retained quantitative zero-event safety-gate evidence.
    #[must_use]
    pub const fn zero_event_safety_gate_report(&self) -> &ZeroEventSafetyGateReport {
        &self.zero_event_safety_gate_report
    }
}

/// Fail-closed error while constructing combined commercial release evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommercialReleaseAcceptanceError {
    /// Mandatory-suite or buyer-visible limitation evidence was invalid.
    ReleaseEvidence(ReleaseDecisionError),
    /// Quantitative zero-event safety requirements or observations were invalid.
    ZeroEventSafetyGate(ZeroEventSafetyGateError),
}

impl fmt::Display for CommercialReleaseAcceptanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReleaseEvidence(error) => write!(formatter, "invalid release evidence: {error}"),
            Self::ZeroEventSafetyGate(error) => {
                write!(formatter, "invalid zero-event safety gate: {error}")
            }
        }
    }
}

impl std::error::Error for CommercialReleaseAcceptanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReleaseEvidence(error) => Some(error),
            Self::ZeroEventSafetyGate(error) => Some(error),
        }
    }
}

impl From<ReleaseDecisionError> for CommercialReleaseAcceptanceError {
    fn from(error: ReleaseDecisionError) -> Self {
        Self::ReleaseEvidence(error)
    }
}

impl From<ZeroEventSafetyGateError> for CommercialReleaseAcceptanceError {
    fn from(error: ZeroEventSafetyGateError) -> Self {
        Self::ZeroEventSafetyGate(error)
    }
}

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

/// Combine mandatory benchmark evidence with mandatory quantitative zero-event safety policy.
///
/// The benchmark evaluator retains product failures, evidence insufficiency, and explicit buyer
/// limitations. The quantitative gate is then applied as a mandatory acceptance condition: an
/// otherwise accepted benchmark report becomes `Inconclusive` when any declared zero-event
/// requirement is missing or statistically insufficient. A known benchmark failure remains
/// `Rejected`. Invalid inputs from either evidence boundary are returned with their original typed
/// source error instead of being converted to success.
pub fn decide_commercial_release_with_zero_event_safety<I>(
    evidence: I,
    declared_limitations: &[DeclaredLimitation],
    observations: &[ZeroEventSafetyObservation],
    requirements: &[ZeroEventSafetyRequirement],
) -> Result<CommercialReleaseAcceptanceReport, CommercialReleaseAcceptanceError>
where
    I: IntoIterator<Item = BenchmarkSuiteEvidence>,
{
    let benchmark_report = decide_release_with_classified_benchmark_evidence(
        evidence,
        declared_limitations,
        observations,
    )?;
    let zero_event_safety_gate_report =
        evaluate_zero_event_safety_gate(requirements, observations)?;

    let decision = match benchmark_report.decision() {
        ReleaseDecision::Rejected => ReleaseDecision::Rejected,
        ReleaseDecision::Inconclusive => ReleaseDecision::Inconclusive,
        ReleaseDecision::Accepted | ReleaseDecision::AcceptedWithDeclaredLimitations => {
            match zero_event_safety_gate_report.decision() {
                ZeroEventSafetyGateDecision::Satisfied => benchmark_report.decision(),
                ZeroEventSafetyGateDecision::Inconclusive => ReleaseDecision::Inconclusive,
            }
        }
    };

    Ok(CommercialReleaseAcceptanceReport {
        benchmark_report,
        zero_event_safety_gate_report,
        decision,
    })
}
