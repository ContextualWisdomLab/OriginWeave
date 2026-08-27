//! Deterministic acceptance policy for the controlled benchmark suite.
//!
//! This module evaluates already-collected, credential-free aggregate evidence.
//! It does not execute a browser, authenticate evidence, select or license a
//! corpus, or establish provenance by itself; those responsibilities belong to
//! the benchmark runner and evidence pipeline. Individual case outcomes remain
//! deliberately distinct from release-level suite outcomes. Only the complete,
//! versioned required-case registry for the declared support profile can produce
//! a [`crate::release_acceptance::BenchmarkSuiteOutcome`] for this one suite.

use crate::release_acceptance::BenchmarkSuiteOutcome;
use std::collections::BTreeSet;
use std::fmt;

/// Canonical number of trials required for one deterministic controlled case.
pub const CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS: u32 = 100;

/// Version of the authoritative controlled deterministic case registry.
///
/// Changing required case identity, required membership, or conditional-support
/// semantics requires a new registry version rather than silently changing the
/// meaning of retained benchmark evidence.
pub const CONTROLLED_DETERMINISTIC_REGISTRY_VERSION: &str = "controlled-deterministic-v1";

/// Stable case identities in the controlled deterministic benchmark registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlledBenchmarkCaseId {
    /// Semantic locate/type/click/submit behavior.
    SemanticInteraction,
    /// Same-document post-condition observation.
    SameDocumentPostCondition,
    /// Navigation post-condition observation.
    NavigationPostCondition,
    /// Structured extraction from DOM and accessibility channels.
    DomAccessibilityExtraction,
    /// Structured JSON-LD extraction.
    JsonLdExtraction,
    /// Structured table extraction.
    TableExtraction,
    /// Bounded structured network-response extraction.
    BoundedNetworkExtraction,
    /// Iframe interaction and authority isolation.
    IframeInteraction,
    /// Shadow-DOM interaction and authority isolation.
    ShadowDomInteraction,
    /// Approved file download behavior.
    ApprovedDownload,
    /// Approved file upload behavior.
    ApprovedUpload,
    /// Approval-required reversible action behavior.
    ApprovalRequiredReversibleAction,
    /// Secret-handle form fill without model disclosure.
    SecretHandleFormFill,
    /// Redirect and origin-transition authority behavior.
    RedirectOriginTransition,
    /// Dynamic mutation and stale-node invalidation.
    DynamicMutationStaleNode,
    /// Session checkpoint, cancellation, and resume behavior.
    SessionCheckpointCancelResume,
    /// Browser crash and task-owned process/profile cleanup.
    BrowserCrashCleanup,
    /// WARC/PROV capture and offline replay behavior.
    WarcProvReplay,
    /// Manifest V3 isolation when the release declares MV3 support.
    ManifestV3Isolation,
    /// Native-messaging isolation when the release declares native-host support.
    NativeMessagingIsolation,
}

impl ControlledBenchmarkCaseId {
    /// Ordered authoritative registry for the controlled deterministic suite.
    pub const ALL: [Self; 20] = [
        Self::SemanticInteraction,
        Self::SameDocumentPostCondition,
        Self::NavigationPostCondition,
        Self::DomAccessibilityExtraction,
        Self::JsonLdExtraction,
        Self::TableExtraction,
        Self::BoundedNetworkExtraction,
        Self::IframeInteraction,
        Self::ShadowDomInteraction,
        Self::ApprovedDownload,
        Self::ApprovedUpload,
        Self::ApprovalRequiredReversibleAction,
        Self::SecretHandleFormFill,
        Self::RedirectOriginTransition,
        Self::DynamicMutationStaleNode,
        Self::SessionCheckpointCancelResume,
        Self::BrowserCrashCleanup,
        Self::WarcProvReplay,
        Self::ManifestV3Isolation,
        Self::NativeMessagingIsolation,
    ];

    /// Stable external identifier bound to this registry version.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SemanticInteraction => "semantic_interaction",
            Self::SameDocumentPostCondition => "same_document_post_condition",
            Self::NavigationPostCondition => "navigation_post_condition",
            Self::DomAccessibilityExtraction => "dom_accessibility_extraction",
            Self::JsonLdExtraction => "json_ld_extraction",
            Self::TableExtraction => "table_extraction",
            Self::BoundedNetworkExtraction => "bounded_network_extraction",
            Self::IframeInteraction => "iframe_interaction",
            Self::ShadowDomInteraction => "shadow_dom_interaction",
            Self::ApprovedDownload => "approved_download",
            Self::ApprovedUpload => "approved_upload",
            Self::ApprovalRequiredReversibleAction => "approval_required_reversible_action",
            Self::SecretHandleFormFill => "secret_handle_form_fill",
            Self::RedirectOriginTransition => "redirect_origin_transition",
            Self::DynamicMutationStaleNode => "dynamic_mutation_stale_node",
            Self::SessionCheckpointCancelResume => "session_checkpoint_cancel_resume",
            Self::BrowserCrashCleanup => "browser_crash_cleanup",
            Self::WarcProvReplay => "warc_prov_replay",
            Self::ManifestV3Isolation => "manifest_v3_isolation",
            Self::NativeMessagingIsolation => "native_messaging_isolation",
        }
    }

    /// Whether this case is required for the explicitly declared support profile.
    #[must_use]
    pub const fn required_for(self, profile: ControlledBenchmarkSupportProfile) -> bool {
        match self {
            Self::ManifestV3Isolation => profile.manifest_v3,
            Self::NativeMessagingIsolation => profile.native_messaging,
            _ => true,
        }
    }
}

/// Conditional release surfaces that alter required controlled-suite evidence.
///
/// A false field means that surface is outside the declared support profile; it
/// does not convert evidence for that surface into a passing or skipped case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlledBenchmarkSupportProfile {
    /// Whether the release claims Manifest V3 extension support.
    pub manifest_v3: bool,
    /// Whether the release claims native-messaging host support.
    pub native_messaging: bool,
}

/// Evaluated threshold outcome for one controlled benchmark case.
///
/// This type is deliberately distinct from
/// [`crate::release_acceptance::BenchmarkSuiteOutcome`] so a single case result
/// cannot be supplied directly as release-level suite evidence. The suite-level
/// evaluator below constructs release acceptance evidence only after validating
/// the registry's complete required case set for the declared support profile.
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

/// Structurally invalid controlled-suite case evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledBenchmarkSuiteError {
    /// One stable case identity appears more than once in the supplied suite evidence.
    DuplicateCase {
        /// Duplicated case identity.
        case_id: ControlledBenchmarkCaseId,
    },
    /// Evidence was supplied for a conditional surface the release does not claim.
    UnexpectedConditionalCase {
        /// Conditional case outside the declared support profile.
        case_id: ControlledBenchmarkCaseId,
    },
}

impl fmt::Display for ControlledBenchmarkSuiteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCase { case_id } => write!(
                formatter,
                "controlled benchmark suite contains duplicate case {}",
                case_id.as_str()
            ),
            Self::UnexpectedConditionalCase { case_id } => write!(
                formatter,
                "controlled benchmark suite contains {} evidence outside the declared support profile",
                case_id.as_str()
            ),
        }
    }
}

impl std::error::Error for ControlledBenchmarkSuiteError {}

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

/// Aggregate the authoritative controlled case registry into one suite outcome.
///
/// Structural evidence is validated before semantic outcomes are considered, so
/// duplicate case identities or evidence for an undeclared conditional surface
/// fail closed instead of being hidden by an unrelated case failure. A known
/// failure in any required case yields [`BenchmarkSuiteOutcome::Failed`]. Missing
/// or inconclusive required evidence yields [`BenchmarkSuiteOutcome::Inconclusive`].
/// Only one passing outcome for every case required by the declared support
/// profile yields [`BenchmarkSuiteOutcome::Passed`]. This function establishes
/// evidence only for the controlled deterministic suite; release acceptance still
/// requires the other mandatory benchmark suites independently.
///
/// # Errors
///
/// Returns [`ControlledBenchmarkSuiteError`] for duplicate case identities or
/// evidence for a conditional case outside the declared support profile.
pub fn evaluate_controlled_benchmark_suite(
    profile: ControlledBenchmarkSupportProfile,
    cases: &[(ControlledBenchmarkCaseId, ControlledBenchmarkCaseOutcome)],
) -> Result<BenchmarkSuiteOutcome, ControlledBenchmarkSuiteError> {
    let mut observed = BTreeSet::new();
    let mut any_failed = false;
    let mut any_inconclusive = false;

    for &(case_id, outcome) in cases {
        if !case_id.required_for(profile) {
            return Err(ControlledBenchmarkSuiteError::UnexpectedConditionalCase { case_id });
        }
        if !observed.insert(case_id) {
            return Err(ControlledBenchmarkSuiteError::DuplicateCase { case_id });
        }

        match outcome {
            ControlledBenchmarkCaseOutcome::Passed => {}
            ControlledBenchmarkCaseOutcome::Failed => any_failed = true,
            ControlledBenchmarkCaseOutcome::Inconclusive => any_inconclusive = true,
        }
    }

    let missing_required = ControlledBenchmarkCaseId::ALL
        .into_iter()
        .any(|case_id| case_id.required_for(profile) && !observed.contains(&case_id));

    if any_failed {
        return Ok(BenchmarkSuiteOutcome::Failed);
    }
    if missing_required || any_inconclusive {
        return Ok(BenchmarkSuiteOutcome::Inconclusive);
    }

    Ok(BenchmarkSuiteOutcome::Passed)
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
