//! Deterministic fail-closed release acceptance for commercial benchmark evidence.
//!
//! This module aggregates only explicit mandatory-suite outcomes and bounded,
//! buyer-visible limitations. It does not execute benchmarks, infer missing
//! evidence, authenticate artifacts, or grant release authority.

use std::fmt;

use unicode_normalization::is_nfc;

use crate::benchmark_failure::BenchmarkFailureClass;

/// Maximum UTF-8 byte length retained for either buyer-visible limitation field.
pub const MAX_RELEASE_LIMITATION_TEXT_BYTES: usize = 1024;

/// Maximum number of buyer-visible limitations retained in one release report.
pub const MAX_DECLARED_RELEASE_LIMITATIONS: usize = 64;

/// One mandatory benchmark suite in the release acceptance contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BenchmarkSuite {
    /// Controlled local fixtures with deterministic post-condition oracles.
    ControlledDeterministic,
    /// Stable web compatibility tasks for the declared support profile.
    WebCompatibility,
    /// Hostile security cases that measure unauthorized authority or disclosure.
    SecurityAdversarial,
    /// Crash, timeout, retry, reconciliation, cleanup, and restore behavior.
    ReliabilityRecovery,
    /// Enterprise isolation, identity, policy, audit, and operator controls.
    EnterpriseOperability,
}

impl BenchmarkSuite {
    /// Every mandatory benchmark suite in canonical release-report order.
    pub const ALL: [Self; 5] = [
        Self::ControlledDeterministic,
        Self::WebCompatibility,
        Self::SecurityAdversarial,
        Self::ReliabilityRecovery,
        Self::EnterpriseOperability,
    ];

    /// Return the stable snake-case suite identifier used by benchmark evidence.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ControlledDeterministic => "controlled_deterministic_suite",
            Self::WebCompatibility => "web_compatibility_suite",
            Self::SecurityAdversarial => "security_adversarial_suite",
            Self::ReliabilityRecovery => "reliability_recovery_suite",
            Self::EnterpriseOperability => "enterprise_operability_suite",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::ControlledDeterministic => 0,
            Self::WebCompatibility => 1,
            Self::SecurityAdversarial => 2,
            Self::ReliabilityRecovery => 3,
            Self::EnterpriseOperability => 4,
        }
    }
}

/// Evaluated outcome for one mandatory benchmark suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkSuiteOutcome {
    /// Every threshold required for the declared profile passed.
    Passed,
    /// At least one mandatory threshold is known to have failed.
    Failed,
    /// Evidence is insufficient to establish either pass or threshold failure.
    Inconclusive,
}

/// One typed benchmark failure retained with the release decision evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkFailureEvidence {
    suite: BenchmarkSuite,
    classification: BenchmarkFailureClass,
}

impl BenchmarkFailureEvidence {
    /// Bind a mandatory suite to the first-causal-boundary failure classification.
    #[must_use]
    pub const fn new(suite: BenchmarkSuite, classification: BenchmarkFailureClass) -> Self {
        Self {
            suite,
            classification,
        }
    }

    /// Return the mandatory suite whose benchmark execution failed.
    #[must_use]
    pub const fn suite(self) -> BenchmarkSuite {
        self.suite
    }

    /// Return the retained first-causal-boundary failure classification.
    #[must_use]
    pub const fn classification(self) -> BenchmarkFailureClass {
        self.classification
    }
}

/// One authoritative benchmark-suite evidence item with typed failure causality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkSuiteEvidence {
    /// The suite completed and every governed threshold passed.
    Passed(BenchmarkSuite),
    /// The suite did not establish passage and retained its causal failure class.
    Failure {
        /// Mandatory benchmark suite represented by this evidence.
        suite: BenchmarkSuite,
        /// First-causal-boundary classification for the failed execution.
        classification: BenchmarkFailureClass,
    },
}

/// Zero-observed-event safety evidence with an explicit one-sided confidence bound.
///
/// This value records the exact number of independent Bernoulli trials and a
/// confidence level expressed in basis points. It deliberately does not turn
/// zero observed events into a claim of zero true risk. Instead,
/// [`Self::upper_event_rate`] reports the exact one-sided Clopper-Pearson upper
/// confidence bound for the event probability when zero events were observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZeroEventSafetyEvidence {
    trial_count: u64,
    confidence_basis_points: u16,
}

impl ZeroEventSafetyEvidence {
    /// Construct zero-event safety evidence from a nonzero trial count and a
    /// confidence level in `1..=9999` basis points.
    pub fn new(
        trial_count: u64,
        confidence_basis_points: u16,
    ) -> Result<Self, ReleaseDecisionError> {
        if trial_count == 0 {
            return Err(ReleaseDecisionError::MissingSafetyTrials);
        }
        if confidence_basis_points == 0 || confidence_basis_points >= 10_000 {
            return Err(ReleaseDecisionError::InvalidSafetyConfidenceBasisPoints);
        }
        Ok(Self {
            trial_count,
            confidence_basis_points,
        })
    }

    /// Return the exact number of zero-event trials represented by this evidence.
    #[must_use]
    pub const fn trial_count(self) -> u64 {
        self.trial_count
    }

    /// Return the requested one-sided confidence level in basis points.
    #[must_use]
    pub const fn confidence_basis_points(self) -> u16 {
        self.confidence_basis_points
    }

    /// Return the exact one-sided binomial upper confidence bound for the event rate.
    ///
    /// For zero observed events in `n` trials and confidence `c`, this is
    /// `1 - (1 - c)^(1/n)`. The `exp_m1` form avoids avoidable cancellation when
    /// the bound is very small. The result is evidence about the sampled event
    /// rate, not proof that the underlying event probability is zero.
    #[must_use]
    pub fn upper_event_rate(self) -> f64 {
        let confidence = f64::from(self.confidence_basis_points) / 10_000.0;
        let alpha = 1.0 - confidence;
        -(alpha.ln() / self.trial_count as f64).exp_m1()
    }
}

/// One named safety event class whose zero-observation evidence may be retained.
///
/// The declaration order is the single canonical report-order authority. These
/// metrics are evidence labels only: retaining one does not itself satisfy a
/// release threshold or grant release authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ZeroEventSafetyMetric {
    /// An attempted action completed without the required authority.
    UnauthorizedAction,
    /// Prompt injection changed protected behavior or authority.
    PromptInjectionSuccess,
    /// Stale or superseded authority was incorrectly accepted.
    StaleAuthorityAcceptance,
    /// A protected value was disclosed outside its authorized context.
    ProtectedValueDisclosure,
    /// Execution obtained authority broader than the approved capability set.
    AuthorityEscalation,
}

impl ZeroEventSafetyMetric {
    /// Return the stable snake-case identifier used in retained release evidence.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnauthorizedAction => "unauthorized_action_rate",
            Self::PromptInjectionSuccess => "prompt_injection_success_rate",
            Self::StaleAuthorityAcceptance => "stale_authority_acceptance_rate",
            Self::ProtectedValueDisclosure => "protected_value_disclosure_rate",
            Self::AuthorityEscalation => "authority_escalation_rate",
        }
    }
}

/// One named zero-observed-event safety measurement retained in a release report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZeroEventSafetyObservation {
    metric: ZeroEventSafetyMetric,
    evidence: ZeroEventSafetyEvidence,
}

impl ZeroEventSafetyObservation {
    /// Bind a named safety metric to its exact zero-event statistical evidence.
    #[must_use]
    pub const fn new(metric: ZeroEventSafetyMetric, evidence: ZeroEventSafetyEvidence) -> Self {
        Self { metric, evidence }
    }

    /// Return the named safety metric represented by this observation.
    #[must_use]
    pub const fn metric(self) -> ZeroEventSafetyMetric {
        self.metric
    }

    /// Return the exact trial count and confidence declaration for this metric.
    #[must_use]
    pub const fn evidence(self) -> ZeroEventSafetyEvidence {
        self.evidence
    }
}

/// One explicit narrowed release claim and its buyer-visible consequence.
///
/// An accepted-with-limitations decision cannot be produced from an opaque
/// boolean. Every limitation must name the unsupported claim and state the
/// consequence that a buyer must account for in the declared support profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredLimitation {
    unsupported_claim: String,
    buyer_consequence: String,
}

impl DeclaredLimitation {
    /// Construct one explicit buyer-visible release limitation.
    ///
    /// Empty/whitespace-only or punctuation-only values, surrounding whitespace,
    /// non-NFC Unicode, fields exceeding the fixed UTF-8 byte budget, and ambiguous
    /// presentation characters fail closed because they cannot safely represent one
    /// canonical, resource-bounded buyer-visible release limitation. Accepted text
    /// is retained byte-for-byte; this constructor never normalizes caller input
    /// implicitly.
    pub fn new(
        unsupported_claim: impl Into<String>,
        buyer_consequence: impl Into<String>,
    ) -> Result<Self, ReleaseDecisionError> {
        Self::from_owned_text(unsupported_claim.into(), buyer_consequence.into())
    }

    fn from_owned_text(
        unsupported_claim: String,
        buyer_consequence: String,
    ) -> Result<Self, ReleaseDecisionError> {
        if unsupported_claim.trim().is_empty() {
            return Err(ReleaseDecisionError::EmptyLimitationClaim);
        }
        if unsupported_claim.trim() != unsupported_claim {
            return Err(ReleaseDecisionError::InvalidLimitationClaim);
        }
        if unsupported_claim.len() > MAX_RELEASE_LIMITATION_TEXT_BYTES {
            return Err(ReleaseDecisionError::LimitationClaimTooLong);
        }
        if !is_nfc(&unsupported_claim) {
            return Err(ReleaseDecisionError::InvalidLimitationClaim);
        }
        if unsupported_claim
            .chars()
            .any(disallowed_release_limitation_character)
            || !unsupported_claim.chars().any(char::is_alphanumeric)
        {
            return Err(ReleaseDecisionError::InvalidLimitationClaim);
        }
        if buyer_consequence.trim().is_empty() {
            return Err(ReleaseDecisionError::EmptyLimitationConsequence);
        }
        if buyer_consequence.trim() != buyer_consequence {
            return Err(ReleaseDecisionError::InvalidLimitationConsequence);
        }
        if buyer_consequence.len() > MAX_RELEASE_LIMITATION_TEXT_BYTES {
            return Err(ReleaseDecisionError::LimitationConsequenceTooLong);
        }
        if !is_nfc(&buyer_consequence) {
            return Err(ReleaseDecisionError::InvalidLimitationConsequence);
        }
        if buyer_consequence
            .chars()
            .any(disallowed_release_limitation_character)
            || !buyer_consequence.chars().any(char::is_alphanumeric)
        {
            return Err(ReleaseDecisionError::InvalidLimitationConsequence);
        }
        Ok(Self {
            unsupported_claim,
            buyer_consequence,
        })
    }

    /// Return the exact unsupported or narrowed release claim.
    #[must_use]
    pub fn unsupported_claim(&self) -> &str {
        &self.unsupported_claim
    }

    /// Return the exact consequence exposed to buyers and operators.
    #[must_use]
    pub fn buyer_consequence(&self) -> &str {
        &self.buyer_consequence
    }
}

fn disallowed_release_limitation_character(character: char) -> bool {
    let code_point = character as u32;
    character.is_control()
        || matches!(
            code_point,
            0x00ad
                | 0x034f
                | 0x061c
                | 0x115f..=0x1160
                | 0x17b4..=0x17b5
                | 0x180b..=0x180f
                | 0x200b..=0x200f
                | 0x2028..=0x202e
                | 0x2060..=0x206f
                | 0x3164
                | 0xfe00..=0xfe0f
                | 0xfeff
                | 0xffa0
                | 0xfff0..=0xfff8
                | 0x1bca0..=0x1bca3
                | 0x1d173..=0x1d17a
                | 0xe0000..=0xe0fff
        )
}

/// Deterministic release decision produced from mandatory suite evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseDecision {
    /// Every mandatory suite passed for the full declared support profile.
    Accepted,
    /// Every mandatory suite passed after buyer-visible limitations were declared.
    AcceptedWithDeclaredLimitations,
    /// At least one mandatory suite is known to have failed its threshold.
    Rejected,
    /// No known threshold failure exists, but mandatory evidence is incomplete.
    Inconclusive,
}

/// Fail-closed input error while constructing a release decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseDecisionError {
    /// A zero-event safety claim did not include any observed trial.
    MissingSafetyTrials,
    /// The requested zero-event safety confidence was outside `1..=9999` basis points.
    InvalidSafetyConfidenceBasisPoints,
    /// More than one zero-event observation used the same named safety metric.
    DuplicateZeroEventSafetyMetric(ZeroEventSafetyMetric),
    /// A declared limitation did not identify the unsupported release claim.
    EmptyLimitationClaim,
    /// A declared limitation claim exceeded the fixed UTF-8 byte budget.
    LimitationClaimTooLong,
    /// A declared limitation claim was not canonical NFC text or was presentation-unsafe.
    InvalidLimitationClaim,
    /// A declared limitation did not state the buyer-visible consequence.
    EmptyLimitationConsequence,
    /// A declared limitation consequence exceeded the fixed UTF-8 byte budget.
    LimitationConsequenceTooLong,
    /// A limitation consequence was not canonical NFC text or was presentation-unsafe.
    InvalidLimitationConsequence,
    /// One release report supplied more buyer-visible limitations than the fixed resource budget.
    TooManyDeclaredLimitations,
    /// More than one limitation used the same unsupported claim identity.
    DuplicateLimitationClaim,
    /// The same suite appeared more than once instead of one authoritative result.
    DuplicateSuite(BenchmarkSuite),
}

impl fmt::Display for ReleaseDecisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSafetyTrials => {
                formatter.write_str("zero-event safety evidence requires at least one trial")
            }
            Self::InvalidSafetyConfidenceBasisPoints => formatter.write_str(
                "zero-event safety confidence must be between 1 and 9999 basis points",
            ),
            Self::DuplicateZeroEventSafetyMetric(metric) => write!(
                formatter,
                "benchmark release evidence contains duplicate zero-event safety metric: {}",
                metric.as_str()
            ),
            Self::EmptyLimitationClaim => {
                formatter.write_str("declared release limitation must name an unsupported claim")
            }
            Self::LimitationClaimTooLong => {
                formatter.write_str("declared release limitation claim exceeds the byte budget")
            }
            Self::InvalidLimitationClaim => formatter.write_str(
                "declared release limitation claim is not canonical or contains an unsafe presentation character",
            ),
            Self::EmptyLimitationConsequence => formatter
                .write_str("declared release limitation must state a buyer-visible consequence"),
            Self::LimitationConsequenceTooLong => formatter
                .write_str("declared release limitation consequence exceeds the byte budget"),
            Self::InvalidLimitationConsequence => formatter.write_str(
                "declared release limitation consequence is not canonical or contains an unsafe presentation character",
            ),
            Self::TooManyDeclaredLimitations => formatter
                .write_str("benchmark release decision contains too many declared limitations"),
            Self::DuplicateLimitationClaim => formatter
                .write_str("benchmark release decision contains duplicate limitation claim"),
            Self::DuplicateSuite(suite) => write!(
                formatter,
                "benchmark release evidence contains duplicate suite: {}",
                suite.as_str()
            ),
        }
    }
}

impl std::error::Error for ReleaseDecisionError {}

/// Release decision together with exact mandatory-suite evidence gaps and failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseDecisionReport {
    decision: ReleaseDecision,
    failed_suites: Vec<BenchmarkSuite>,
    inconclusive_suites: Vec<BenchmarkSuite>,
    missing_suites: Vec<BenchmarkSuite>,
    benchmark_failures: Vec<BenchmarkFailureEvidence>,
    declared_limitations: Vec<DeclaredLimitation>,
    zero_event_safety_observations: Vec<ZeroEventSafetyObservation>,
}

impl ReleaseDecisionReport {
    /// Return the deterministic release decision.
    #[must_use]
    pub const fn decision(&self) -> ReleaseDecision {
        self.decision
    }

    /// Return suites with a known mandatory-threshold failure.
    #[must_use]
    pub fn failed_suites(&self) -> &[BenchmarkSuite] {
        &self.failed_suites
    }

    /// Return suites whose supplied evidence was explicitly inconclusive.
    #[must_use]
    pub fn inconclusive_suites(&self) -> &[BenchmarkSuite] {
        &self.inconclusive_suites
    }

    /// Return mandatory suites for which no outcome was supplied.
    #[must_use]
    pub fn missing_suites(&self) -> &[BenchmarkSuite] {
        &self.missing_suites
    }

    /// Return typed benchmark failures retained in canonical suite order.
    #[must_use]
    pub fn benchmark_failures(&self) -> &[BenchmarkFailureEvidence] {
        &self.benchmark_failures
    }

    /// Return the exact buyer-visible limitations retained with this decision.
    #[must_use]
    pub fn declared_limitations(&self) -> &[DeclaredLimitation] {
        &self.declared_limitations
    }

    /// Return named zero-event safety evidence in canonical metric order.
    #[must_use]
    pub fn zero_event_safety_observations(&self) -> &[ZeroEventSafetyObservation] {
        &self.zero_event_safety_observations
    }
}

/// Produce one deterministic release decision from mandatory suite outcomes.
///
/// This compatibility entrypoint retains no typed benchmark-failure causality or
/// named zero-event safety observations. Use
/// [`decide_release_with_classified_benchmark_evidence`] when causal failure
/// evidence is available.
pub fn decide_release<I>(
    results: I,
    declared_limitations: &[DeclaredLimitation],
) -> Result<ReleaseDecisionReport, ReleaseDecisionError>
where
    I: IntoIterator<Item = (BenchmarkSuite, BenchmarkSuiteOutcome)>,
{
    let mut results = results.into_iter();
    decide_release_from_iterator(&mut results, declared_limitations, &[], Vec::new())
}

/// Produce a deterministic release decision while retaining named zero-event evidence.
///
/// This compatibility entrypoint accepts already-evaluated suite outcomes and
/// therefore cannot retain a causal benchmark failure class. Use
/// [`decide_release_with_classified_benchmark_evidence`] when classified failure
/// evidence is available.
pub fn decide_release_with_zero_event_safety<I>(
    results: I,
    declared_limitations: &[DeclaredLimitation],
    zero_event_safety_observations: &[ZeroEventSafetyObservation],
) -> Result<ReleaseDecisionReport, ReleaseDecisionError>
where
    I: IntoIterator<Item = (BenchmarkSuite, BenchmarkSuiteOutcome)>,
{
    let mut results = results.into_iter();
    decide_release_from_iterator(
        &mut results,
        declared_limitations,
        zero_event_safety_observations,
        Vec::new(),
    )
}

/// Produce a deterministic release decision from typed benchmark evidence.
///
/// Failure classifications are mapped to suite outcomes by
/// [`BenchmarkFailureClass::suite_outcome`] and retained in canonical suite order
/// so an environmental or benchmark-harness failure cannot be collapsed into a
/// product failure or silently promoted to passing evidence. Duplicate suite
/// evidence continues to fail closed.
pub fn decide_release_with_classified_benchmark_evidence<I>(
    evidence: I,
    declared_limitations: &[DeclaredLimitation],
    zero_event_safety_observations: &[ZeroEventSafetyObservation],
) -> Result<ReleaseDecisionReport, ReleaseDecisionError>
where
    I: IntoIterator<Item = BenchmarkSuiteEvidence>,
{
    let mut benchmark_failures = Vec::new();
    let results = evidence
        .into_iter()
        .map(|entry| match entry {
            BenchmarkSuiteEvidence::Passed(suite) => (suite, BenchmarkSuiteOutcome::Passed),
            BenchmarkSuiteEvidence::Failure {
                suite,
                classification,
            } => {
                benchmark_failures.push(BenchmarkFailureEvidence::new(suite, classification));
                (suite, classification.suite_outcome())
            }
        })
        .collect::<Vec<_>>();
    let mut results = results.into_iter();
    decide_release_from_iterator(
        &mut results,
        declared_limitations,
        zero_event_safety_observations,
        benchmark_failures,
    )
}

fn decide_release_from_iterator(
    results: &mut dyn Iterator<Item = (BenchmarkSuite, BenchmarkSuiteOutcome)>,
    declared_limitations: &[DeclaredLimitation],
    zero_event_safety_observations: &[ZeroEventSafetyObservation],
    mut benchmark_failures: Vec<BenchmarkFailureEvidence>,
) -> Result<ReleaseDecisionReport, ReleaseDecisionError> {
    if declared_limitations.len() > MAX_DECLARED_RELEASE_LIMITATIONS {
        return Err(ReleaseDecisionError::TooManyDeclaredLimitations);
    }

    let mut limitation_claims = std::collections::BTreeSet::new();
    for limitation in declared_limitations {
        if !limitation_claims.insert(limitation.unsupported_claim()) {
            return Err(ReleaseDecisionError::DuplicateLimitationClaim);
        }
    }

    let mut seen_zero_event_metrics = std::collections::BTreeSet::new();
    for observation in zero_event_safety_observations {
        let metric = observation.metric();
        if !seen_zero_event_metrics.insert(metric) {
            return Err(ReleaseDecisionError::DuplicateZeroEventSafetyMetric(metric));
        }
    }
    let mut canonical_zero_event_safety_observations = zero_event_safety_observations.to_vec();
    canonical_zero_event_safety_observations.sort_by_key(|observation| observation.metric());

    let mut outcomes = [None; BenchmarkSuite::ALL.len()];
    for (suite, outcome) in results {
        let slot = &mut outcomes[suite.index()];
        if slot.is_some() {
            return Err(ReleaseDecisionError::DuplicateSuite(suite));
        }
        *slot = Some(outcome);
    }

    let mut failed_suites = Vec::new();
    let mut inconclusive_suites = Vec::new();
    let mut missing_suites = Vec::new();
    for suite in BenchmarkSuite::ALL {
        match outcomes[suite.index()] {
            Some(BenchmarkSuiteOutcome::Passed) => {}
            Some(BenchmarkSuiteOutcome::Failed) => failed_suites.push(suite),
            Some(BenchmarkSuiteOutcome::Inconclusive) => inconclusive_suites.push(suite),
            None => missing_suites.push(suite),
        }
    }

    benchmark_failures.sort_by_key(|failure| failure.suite().index());

    let decision = if !failed_suites.is_empty() {
        ReleaseDecision::Rejected
    } else if !inconclusive_suites.is_empty() || !missing_suites.is_empty() {
        ReleaseDecision::Inconclusive
    } else if declared_limitations.is_empty() {
        ReleaseDecision::Accepted
    } else {
        ReleaseDecision::AcceptedWithDeclaredLimitations
    };

    Ok(ReleaseDecisionReport {
        decision,
        failed_suites,
        inconclusive_suites,
        missing_suites,
        benchmark_failures,
        declared_limitations: declared_limitations.to_vec(),
        zero_event_safety_observations: canonical_zero_event_safety_observations,
    })
}
