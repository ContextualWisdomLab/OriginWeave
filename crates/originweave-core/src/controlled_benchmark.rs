//! Deterministic acceptance policy for the controlled benchmark suite.
//!
//! This module evaluates already-collected, credential-free trial evidence.
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

/// Unicode security profile applied only to benchmark-owned evidence identity.
///
/// This profile adopts Unicode 18.0.0 `Default_Ignorable_Code_Point` without
/// tailored exceptions. Browser-issued protocol identifiers are outside this
/// grammar and remain lossless at their own bounded-context boundary.
pub const CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE: &str =
    "unicode-18.0.0-default-ignorable-exclusion";

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
/// Native messaging is an extension capability, so a profile that claims native
/// messaging while omitting Manifest V3 support is structurally invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlledBenchmarkSupportProfile {
    /// Whether the release claims Manifest V3 extension support.
    pub manifest_v3: bool,
    /// Whether the release claims native-messaging host support.
    pub native_messaging: bool,
}

/// Exact reproducibility context required to compare one controlled benchmark run.
///
/// These identity strings are supplied by the benchmark runner and remain
/// credential-free. This evaluator does not authenticate them; durable evidence
/// must bind them to signed execution artifacts before release authority can rely
/// on the run. The expected and observed contexts must match byte-for-byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlledBenchmarkRunContext<'a> {
    /// Exact OriginWeave source or signed-release revision identity.
    pub originweave_revision: &'a str,
    /// Exact Chromium revision used by the browser lane.
    pub chromium_revision: &'a str,
    /// Exact operating-system image identity.
    pub os_image: &'a str,
    /// Exact hardware profile used for the run.
    pub hardware_profile: &'a str,
    /// Exact browser protocol-adapter generation set.
    pub protocol_adapters: &'a str,
    /// Exact model/provider route, or an explicit deterministic no-model marker.
    pub model_provider: &'a str,
    /// Exact prompt/reasoning or deterministic-oracle configuration identity.
    pub reasoning_configuration: &'a str,
    /// Exact fixture or corpus version identity.
    pub fixture_corpus_version: &'a str,
    /// Exact deterministic random-seed set identity.
    pub random_seed_set: &'a str,
}

impl<'a> ControlledBenchmarkRunContext<'a> {
    /// Returns every reproducibility identity in the single stable comparison order.
    ///
    /// Keeping the field name beside its value lets fail-closed validation report the
    /// first causal boundary without maintaining a second hand-written comparison map.
    fn fields(self) -> [(&'static str, &'a str); 9] {
        [
            ("originweave_revision", self.originweave_revision),
            ("chromium_revision", self.chromium_revision),
            ("os_image", self.os_image),
            ("hardware_profile", self.hardware_profile),
            ("protocol_adapters", self.protocol_adapters),
            ("model_provider", self.model_provider),
            ("reasoning_configuration", self.reasoning_configuration),
            ("fixture_corpus_version", self.fixture_corpus_version),
            ("random_seed_set", self.random_seed_set),
        ]
    }
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

/// Canonical aggregate evidence derived from controlled-benchmark trials.
///
/// Release-level suite authority does not accept this aggregate directly. Use
/// [`aggregate_controlled_benchmark_trials`] for case-level diagnostics or pass
/// raw [`ControlledBenchmarkTrialEvidence`] through [`ControlledBenchmarkCaseTrials`]
/// to [`evaluate_controlled_benchmark_suite`].
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
    /// Unauthorized side-effect events observed across all represented trials.
    ///
    /// This is an event count rather than a per-trial outcome count: one trial
    /// can expose more than one unauthorized side effect. A nonzero event count
    /// requires at least one represented trial.
    pub unauthorized_side_effects: u32,
}

/// One canonical controlled-benchmark trial observation.
///
/// Trial ordinals are one-based slots in the fixed deterministic trial budget.
/// This value records runner-reported outcomes but does not authenticate the
/// runner, corpus, build identity, or provenance assertion; those remain duties
/// of the benchmark runner and durable evidence pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlledBenchmarkTrialEvidence {
    /// One-based canonical trial slot within the deterministic case budget.
    pub trial_ordinal: u32,
    /// Whether the typed browser action completed successfully.
    pub action_succeeded: bool,
    /// Whether the observed post-condition exactly matched the expected state.
    pub exact_post_condition: bool,
    /// Whether this trial carries the complete required provenance assertion.
    pub provenance_complete: bool,
    /// Unauthorized side-effect events observed during this trial.
    pub unauthorized_side_effects: u32,
}

/// Raw trial evidence for one stable controlled-benchmark case identity.
///
/// The suite evaluator owns aggregate derivation from these trials. Keeping the
/// case identity and trial observations together prevents callers from minting a
/// passing release-level suite result by supplying fabricated aggregate counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlledBenchmarkCaseTrials {
    /// Stable case identity bound to the controlled registry version.
    pub case_id: ControlledBenchmarkCaseId,
    /// Raw canonical trial observations for this case.
    pub trials: Vec<ControlledBenchmarkTrialEvidence>,
}

/// Invalid trial-level evidence supplied to the canonical aggregate boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledBenchmarkTrialAggregationError {
    /// A trial ordinal lies outside the one-based canonical trial budget.
    InvalidTrialOrdinal {
        /// Invalid ordinal supplied by the runner.
        observed: u32,
        /// Highest accepted canonical trial ordinal.
        maximum: u32,
    },
    /// More than one observation claims the same canonical trial slot.
    DuplicateTrialOrdinal {
        /// Duplicated canonical trial ordinal.
        trial_ordinal: u32,
    },
    /// Summing unauthorized side-effect events exceeded the representable count.
    UnauthorizedSideEffectCountOverflow,
}

impl fmt::Display for ControlledBenchmarkTrialAggregationError {
    /// Formats the first malformed-trial boundary without discarding its exact ordinal.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTrialOrdinal { observed, maximum } => write!(
                formatter,
                "controlled benchmark trial ordinal {observed} is outside the canonical range 1..={maximum}"
            ),
            Self::DuplicateTrialOrdinal { trial_ordinal } => write!(
                formatter,
                "controlled benchmark trial ordinal {trial_ordinal} is duplicated"
            ),
            Self::UnauthorizedSideEffectCountOverflow => formatter
                .write_str("controlled benchmark unauthorized side-effect event count overflowed"),
        }
    }
}

impl std::error::Error for ControlledBenchmarkTrialAggregationError {}

/// Derive canonical aggregate evidence from individual controlled-benchmark trials.
///
/// Each accepted trial must claim one unique ordinal in the fixed one-based trial
/// budget. Aggregate counters are derived here rather than accepted from callers,
/// so duplicated or out-of-budget trial slots cannot inflate a case toward the
/// passing threshold. This function validates evidence shape only: it does not
/// authenticate the runner, corpus/build identity, outcome truth, or provenance
/// assertion. Those remain fail-closed responsibilities of the runner and durable
/// evidence pipeline.
///
/// # Errors
///
/// Returns [`ControlledBenchmarkTrialAggregationError`] when a trial ordinal is
/// outside the canonical budget, a canonical slot is duplicated, or unauthorized
/// side-effect event counts cannot be represented without overflow.
pub fn aggregate_controlled_benchmark_trials(
    trials: &[ControlledBenchmarkTrialEvidence],
) -> Result<ControlledBenchmarkCaseEvidence, ControlledBenchmarkTrialAggregationError> {
    let mut observed_ordinals = BTreeSet::new();
    let mut total_trials = 0u32;
    let mut successful_trials = 0u32;
    let mut exact_post_condition_trials = 0u32;
    let mut provenance_complete_trials = 0u32;
    let mut unauthorized_side_effects = 0u32;

    for trial in trials {
        if !(1..=CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS).contains(&trial.trial_ordinal) {
            return Err(
                ControlledBenchmarkTrialAggregationError::InvalidTrialOrdinal {
                    observed: trial.trial_ordinal,
                    maximum: CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS,
                },
            );
        }
        if !observed_ordinals.insert(trial.trial_ordinal) {
            return Err(
                ControlledBenchmarkTrialAggregationError::DuplicateTrialOrdinal {
                    trial_ordinal: trial.trial_ordinal,
                },
            );
        }

        total_trials += 1;
        if trial.action_succeeded {
            successful_trials += 1;
        }
        if trial.exact_post_condition {
            exact_post_condition_trials += 1;
        }
        if trial.provenance_complete {
            provenance_complete_trials += 1;
        }
        unauthorized_side_effects = unauthorized_side_effects
            .checked_add(trial.unauthorized_side_effects)
            .ok_or(ControlledBenchmarkTrialAggregationError::UnauthorizedSideEffectCountOverflow)?;
    }

    Ok(ControlledBenchmarkCaseEvidence {
        total_trials,
        successful_trials,
        exact_post_condition_trials,
        provenance_complete_trials,
        unauthorized_side_effects,
    })
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
    /// A per-trial outcome counter claims more observations than the trial count.
    CounterExceedsTrialCount {
        /// Name of the invalid aggregate counter.
        counter: &'static str,
        /// Observation count claimed by the invalid counter.
        observed: u32,
        /// Total number of represented trials.
        total_trials: u32,
    },
    /// An event counter claims observations while the evidence represents no trial.
    EventWithoutTrial {
        /// Name of the invalid aggregate event counter.
        counter: &'static str,
        /// Event count claimed without a represented trial.
        observed: u32,
    },
}

impl fmt::Display for ControlledBenchmarkError {
    /// Formats malformed aggregate evidence while preserving the offending counter.
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
            Self::EventWithoutTrial { counter, observed } => write!(
                formatter,
                "controlled benchmark event counter {counter} reports {observed} events with no represented trials"
            ),
        }
    }
}

impl std::error::Error for ControlledBenchmarkError {}

/// Structurally or semantically invalid controlled-suite case evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlledBenchmarkSuiteError {
    /// The declared support profile describes an impossible dependency combination.
    InvalidSupportProfile,
    /// Evidence belongs to a different controlled deterministic registry version.
    RegistryVersionMismatch,
    /// A required or observed reproducibility-context field is blank.
    InvalidRunContext {
        /// Name of the invalid reproducibility-context field.
        field: &'static str,
    },
    /// A required or observed reproducibility-context field has surrounding whitespace.
    NonCanonicalRunContext {
        /// Name of the non-canonical reproducibility-context field.
        field: &'static str,
    },
    /// A required or observed reproducibility-context field contains a disallowed rendering control or Unicode 18.0.0 default-ignorable scalar.
    ControlCharacterRunContext {
        /// Name of the invalid reproducibility-context field.
        field: &'static str,
    },
    /// Observed execution context does not match the required reproducibility context.
    RunContextMismatch {
        /// Name of the mismatched reproducibility-context field.
        field: &'static str,
    },
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
    /// Trial evidence for one case is malformed or non-canonical.
    InvalidTrialEvidence {
        /// Case whose raw trial evidence failed canonical aggregation.
        case_id: ControlledBenchmarkCaseId,
        /// Trial aggregation error preserving the first causal boundary.
        source: ControlledBenchmarkTrialAggregationError,
    },
}

impl fmt::Display for ControlledBenchmarkSuiteError {
    /// Formats suite-admission errors without normalizing case or context identity.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSupportProfile => formatter.write_str(
                "controlled benchmark support profile cannot claim native messaging without Manifest V3 extension support",
            ),
            Self::RegistryVersionMismatch => formatter.write_str(
                "controlled benchmark suite evidence registry version does not match the required registry",
            ),
            Self::InvalidRunContext { field } => write!(
                formatter,
                "controlled benchmark run context field {field} is blank"
            ),
            Self::NonCanonicalRunContext { field } => write!(
                formatter,
                "controlled benchmark run context field {field} contains non-canonical surrounding whitespace"
            ),
            Self::ControlCharacterRunContext { field } => write!(
                formatter,
                "controlled benchmark run context field {field} contains a disallowed C0/C1 control, Unicode line/paragraph separator, bidirectional formatting character, or Unicode 18.0.0 Default_Ignorable_Code_Point"
            ),
            Self::RunContextMismatch { field } => write!(
                formatter,
                "controlled benchmark run context field {field} does not match the required reproducibility context"
            ),
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
            Self::InvalidTrialEvidence { case_id, source } => write!(
                formatter,
                "controlled benchmark suite case {} has invalid trial evidence: {source}",
                case_id.as_str()
            ),
        }
    }
}

impl std::error::Error for ControlledBenchmarkSuiteError {
    /// Exposes only the underlying trial-shape error; other variants are causal roots.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidTrialEvidence { source, .. } => Some(source),
            Self::InvalidSupportProfile
            | Self::RegistryVersionMismatch
            | Self::InvalidRunContext { .. }
            | Self::NonCanonicalRunContext { .. }
            | Self::ControlCharacterRunContext { .. }
            | Self::RunContextMismatch { .. }
            | Self::DuplicateCase { .. }
            | Self::UnexpectedConditionalCase { .. } => None,
        }
    }
}

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
/// trial count, when a per-trial outcome counter exceeds `total_trials`, or when
/// an event counter reports observations despite `total_trials` being zero.
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

    if evidence.total_trials == 0 && evidence.unauthorized_side_effects != 0 {
        return Err(ControlledBenchmarkError::EventWithoutTrial {
            counter: "unauthorized_side_effects",
            observed: evidence.unauthorized_side_effects,
        });
    }

    Ok(evaluate_valid_controlled_benchmark_case(evidence))
}

/// Classifies evidence after structural counters have already been validated.
///
/// Keeping this branch free of additional admission logic lets suite evaluation reuse
/// the exact same threshold decision only after it derives canonical raw-trial evidence.
fn evaluate_valid_controlled_benchmark_case(
    evidence: ControlledBenchmarkCaseEvidence,
) -> ControlledBenchmarkCaseOutcome {
    if evidence.successful_trials < evidence.total_trials
        || evidence.exact_post_condition_trials < evidence.total_trials
        || evidence.provenance_complete_trials < evidence.total_trials
        || evidence.unauthorized_side_effects != 0
    {
        return ControlledBenchmarkCaseOutcome::Failed;
    }

    if evidence.total_trials < CONTROLLED_DETERMINISTIC_REQUIRED_TRIALS {
        return ControlledBenchmarkCaseOutcome::Inconclusive;
    }

    ControlledBenchmarkCaseOutcome::Passed
}

/// Evaluate raw controlled-suite evidence only when execution context is reproducible.
///
/// Every required identity in the expected and observed contexts must be nonblank,
/// free of surrounding whitespace, and free of C0/C1 controls, Unicode line/paragraph
/// separators, bidirectional formatting characters, and Unicode 18.0.0
/// `Default_Ignorable_Code_Point` scalars before the observed context is compared
/// byte-for-byte with the expected context. This does not authenticate either context;
/// it is a fail-closed comparison boundary for a benchmark runner or durable evidence
/// pipeline that performs that authentication.
///
/// # Errors
///
/// Returns [`ControlledBenchmarkSuiteError::InvalidRunContext`] for a blank
/// required or observed identity, [`ControlledBenchmarkSuiteError::NonCanonicalRunContext`]
/// for surrounding whitespace, [`ControlledBenchmarkSuiteError::ControlCharacterRunContext`]
/// for disallowed rendering controls or Unicode 18.0.0 default-ignorables, and
/// [`ControlledBenchmarkSuiteError::RunContextMismatch`] for the first mismatched
/// identity. After context validation, all errors from
/// [`evaluate_controlled_benchmark_suite`] are preserved unchanged.
pub fn evaluate_controlled_benchmark_suite_for_run(
    expected_context: ControlledBenchmarkRunContext<'_>,
    observed_context: ControlledBenchmarkRunContext<'_>,
    registry_version: &str,
    profile: ControlledBenchmarkSupportProfile,
    cases: &[ControlledBenchmarkCaseTrials],
) -> Result<BenchmarkSuiteOutcome, ControlledBenchmarkSuiteError> {
    for ((field, expected_value), (_, observed_value)) in expected_context
        .fields()
        .into_iter()
        .zip(observed_context.fields())
    {
        validate_run_context_field(field, expected_value)?;
        validate_run_context_field(field, observed_value)?;
        if expected_value != observed_value {
            return Err(ControlledBenchmarkSuiteError::RunContextMismatch { field });
        }
    }

    evaluate_controlled_benchmark_suite(registry_version, profile, cases)
}

/// Evaluate raw trial evidence for the authoritative controlled case registry.
///
/// The caller must bind the supplied evidence to the exact current
/// [`CONTROLLED_DETERMINISTIC_REGISTRY_VERSION`]. A mismatched registry version
/// fails closed before any case evidence can influence the suite outcome. The
/// declared support profile is also validated before evidence admission: native
/// messaging cannot be claimed without the Manifest V3 extension surface that
/// owns that capability. The release-level suite boundary accepts raw trial
/// observations grouped in [`ControlledBenchmarkCaseTrials`] and derives each
/// aggregate itself with [`aggregate_controlled_benchmark_trials`]. Callers cannot
/// mint suite authority by supplying aggregate counters or precomputed passing
/// case outcomes. Structural registry checks run before threshold evaluation:
/// duplicate identities or evidence for an undeclared conditional surface fail
/// closed first. A known failure in any required case yields
/// [`BenchmarkSuiteOutcome::Failed`]. Missing or inconclusive required evidence
/// yields [`BenchmarkSuiteOutcome::Inconclusive`]. Only trial evidence that derives
/// one passing outcome for every case required by the declared support profile
/// yields [`BenchmarkSuiteOutcome::Passed`]. This function establishes evidence
/// only for the controlled deterministic suite; release acceptance still requires
/// the other mandatory benchmark suites independently.
///
/// # Errors
///
/// Returns [`ControlledBenchmarkSuiteError`] for an invalid support-profile
/// dependency combination, registry-version mismatch, duplicate case identities,
/// evidence for a conditional case outside the declared support profile, or
/// malformed trial evidence.
pub fn evaluate_controlled_benchmark_suite(
    registry_version: &str,
    profile: ControlledBenchmarkSupportProfile,
    cases: &[ControlledBenchmarkCaseTrials],
) -> Result<BenchmarkSuiteOutcome, ControlledBenchmarkSuiteError> {
    if profile.native_messaging && !profile.manifest_v3 {
        return Err(ControlledBenchmarkSuiteError::InvalidSupportProfile);
    }

    if registry_version != CONTROLLED_DETERMINISTIC_REGISTRY_VERSION {
        return Err(ControlledBenchmarkSuiteError::RegistryVersionMismatch);
    }

    let mut observed = BTreeSet::new();

    for case in cases {
        let case_id = case.case_id;
        if !case_id.required_for(profile) {
            return Err(ControlledBenchmarkSuiteError::UnexpectedConditionalCase { case_id });
        }
        if !observed.insert(case_id) {
            return Err(ControlledBenchmarkSuiteError::DuplicateCase { case_id });
        }
    }

    let mut any_failed = false;
    let mut any_inconclusive = false;
    for case in cases {
        let case_id = case.case_id;
        let evidence = aggregate_controlled_benchmark_trials(&case.trials).map_err(|source| {
            ControlledBenchmarkSuiteError::InvalidTrialEvidence { case_id, source }
        })?;
        let outcome = evaluate_valid_controlled_benchmark_case(evidence);

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

/// Rejects benchmark-owned run identities that would make evidence boundaries ambiguous.
///
/// The controlled benchmark owns these labels, so surrounding whitespace, C0/C1
/// controls, Unicode line/paragraph separators, bidirectional formatting controls,
/// and Unicode 18.0.0 `Default_Ignorable_Code_Point` scalars are invalid here even
/// though browser-issued protocol identifiers are preserved losslessly at their own
/// bounded-context boundary.
fn validate_run_context_field(
    field: &'static str,
    value: &str,
) -> Result<(), ControlledBenchmarkSuiteError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ControlledBenchmarkSuiteError::InvalidRunContext { field });
    }
    if trimmed != value {
        return Err(ControlledBenchmarkSuiteError::NonCanonicalRunContext { field });
    }
    if value.chars().any(|character| {
        character.is_control()
            || is_unicode_line_separator(character)
            || is_bidi_control(character)
            || is_unicode_18_default_ignorable(character)
    }) {
        return Err(ControlledBenchmarkSuiteError::ControlCharacterRunContext { field });
    }
    Ok(())
}

/// Identifies Unicode separators that create a new rendered line or paragraph.
///
/// `char::is_control` covers General Category `Cc`, not `Zl`/`Zp`. Allowing these
/// separators inside benchmark-owned evidence identity would therefore let one
/// byte-exact identity render as multiple log or report records.
fn is_unicode_line_separator(character: char) -> bool {
    matches!(character, '\u{2028}' | '\u{2029}')
}

/// Identifies the Unicode `Bidi_Control` set without rejecting ordinary RTL scripts.
///
/// These format characters can reorder the visual presentation of otherwise byte-exact
/// benchmark identity strings. The benchmark owns this metadata grammar, so admitting
/// them would make logs, reports, and signed-summary review ambiguous even when storage
/// preserves the original scalar sequence.
fn is_bidi_control(character: char) -> bool {
    matches!(
        character,
        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
    )
}

/// Identifies Unicode 18.0.0 `Default_Ignorable_Code_Point` exactly.
///
/// The compressed ranges below are the 4,174 scalars published in Unicode 18.0.0
/// `DerivedCoreProperties.txt`. They are intentionally version-pinned instead of
/// delegated to a moving Unicode library so a toolchain or dependency upgrade cannot
/// silently change benchmark evidence identity admission. This profile has no tailored
/// ZWJ/ZWNJ, variation-selector, tag, or script-specific exception.
fn is_unicode_18_default_ignorable(character: char) -> bool {
    matches!(
        character,
        '\u{00ad}'
            | '\u{034f}'
            | '\u{061c}'
            | '\u{115f}'..='\u{1160}'
            | '\u{17b4}'..='\u{17b5}'
            | '\u{180b}'..='\u{180f}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{3164}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{feff}'
            | '\u{ffa0}'
            | '\u{fff0}'..='\u{fff8}'
            | '\u{1bca0}'..='\u{1bca3}'
            | '\u{1d173}'..='\u{1d17a}'
            | '\u{e0000}'..='\u{e0fff}'
    )
}

/// Verifies that a per-trial outcome counter cannot claim more observations than trials.
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
