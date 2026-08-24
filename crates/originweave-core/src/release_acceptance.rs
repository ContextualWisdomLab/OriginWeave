//! Deterministic fail-closed release acceptance for commercial benchmark evidence.
//!
//! This module aggregates only explicit mandatory-suite outcomes and bounded,
//! buyer-visible limitations. It does not execute benchmarks, infer missing
//! evidence, authenticate artifacts, or grant release authority.

use std::fmt;

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
    /// Empty/whitespace-only values, surrounding whitespace, fields exceeding the
    /// fixed UTF-8 byte budget, and ambiguous presentation characters fail closed
    /// because they cannot safely represent one canonical, resource-bounded
    /// buyer-visible release limitation.
    pub fn new(
        unsupported_claim: impl Into<String>,
        buyer_consequence: impl Into<String>,
    ) -> Result<Self, ReleaseDecisionError> {
        let unsupported_claim = unsupported_claim.into();
        if unsupported_claim.trim().is_empty() {
            return Err(ReleaseDecisionError::EmptyLimitationClaim);
        }
        if unsupported_claim.trim() != unsupported_claim {
            return Err(ReleaseDecisionError::InvalidLimitationClaim);
        }
        if unsupported_claim.len() > MAX_RELEASE_LIMITATION_TEXT_BYTES {
            return Err(ReleaseDecisionError::LimitationClaimTooLong);
        }
        if unsupported_claim
            .chars()
            .any(disallowed_release_limitation_character)
        {
            return Err(ReleaseDecisionError::InvalidLimitationClaim);
        }
        let buyer_consequence = buyer_consequence.into();
        if buyer_consequence.trim().is_empty() {
            return Err(ReleaseDecisionError::EmptyLimitationConsequence);
        }
        if buyer_consequence.trim() != buyer_consequence {
            return Err(ReleaseDecisionError::InvalidLimitationConsequence);
        }
        if buyer_consequence.len() > MAX_RELEASE_LIMITATION_TEXT_BYTES {
            return Err(ReleaseDecisionError::LimitationConsequenceTooLong);
        }
        if buyer_consequence
            .chars()
            .any(disallowed_release_limitation_character)
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
    /// A declared limitation did not identify the unsupported release claim.
    EmptyLimitationClaim,
    /// A declared limitation claim exceeded the fixed UTF-8 byte budget.
    LimitationClaimTooLong,
    /// A declared limitation claim contained an unsafe presentation character.
    InvalidLimitationClaim,
    /// A declared limitation did not state the buyer-visible consequence.
    EmptyLimitationConsequence,
    /// A declared limitation consequence exceeded the fixed UTF-8 byte budget.
    LimitationConsequenceTooLong,
    /// A declared limitation consequence contained an unsafe presentation character.
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
            Self::EmptyLimitationClaim => {
                formatter.write_str("declared release limitation must name an unsupported claim")
            }
            Self::LimitationClaimTooLong => {
                formatter.write_str("declared release limitation claim exceeds the byte budget")
            }
            Self::InvalidLimitationClaim => formatter.write_str(
                "declared release limitation claim contains an unsafe presentation character",
            ),
            Self::EmptyLimitationConsequence => formatter
                .write_str("declared release limitation must state a buyer-visible consequence"),
            Self::LimitationConsequenceTooLong => formatter
                .write_str("declared release limitation consequence exceeds the byte budget"),
            Self::InvalidLimitationConsequence => formatter.write_str(
                "declared release limitation consequence contains an unsafe presentation character",
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
    declared_limitations: Vec<DeclaredLimitation>,
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

    /// Return the exact buyer-visible limitations retained with this decision.
    #[must_use]
    pub fn declared_limitations(&self) -> &[DeclaredLimitation] {
        &self.declared_limitations
    }
}

/// Produce one deterministic release decision from mandatory suite outcomes.
///
/// Duplicate suite evidence, duplicate buyer-visible limitation claim identities,
/// and excessive declared-limitation cardinality fail closed rather than selecting
/// or retaining ambiguous or attacker-controlled release metadata. A known
/// mandatory-threshold failure is always rejected, even when other suites are
/// missing or inconclusive; all such evidence gaps remain in the returned report.
/// Without a known failure, missing or inconclusive evidence is never promoted to
/// acceptance. Accepted-with-limitations requires at least one validated
/// [`DeclaredLimitation`], so the decision cannot be detached from the exact
/// narrowed claim and buyer-visible consequence.
pub fn decide_release<I>(
    results: I,
    declared_limitations: &[DeclaredLimitation],
) -> Result<ReleaseDecisionReport, ReleaseDecisionError>
where
    I: IntoIterator<Item = (BenchmarkSuite, BenchmarkSuiteOutcome)>,
{
    if declared_limitations.len() > MAX_DECLARED_RELEASE_LIMITATIONS {
        return Err(ReleaseDecisionError::TooManyDeclaredLimitations);
    }

    let mut limitation_claims = std::collections::BTreeSet::new();
    for limitation in declared_limitations {
        if !limitation_claims.insert(limitation.unsupported_claim()) {
            return Err(ReleaseDecisionError::DuplicateLimitationClaim);
        }
    }

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
        declared_limitations: declared_limitations.to_vec(),
    })
}
