//! Deterministic, fail-closed policy evaluation for OriginWeave actions.
//!
//! The evaluator performs no I/O and never executes a browser action. Callers
//! must present a complete [`originweave_core::PolicyContext`] and may execute
//! only an explicit [`Decision::Allow`].

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod sensitive_data;

pub use sensitive_data::{
    DataClassification, DisclosureDecision, DisclosureScope, HandleUseDecision, HandleUseRequest,
    SensitiveDataAuthority, SensitiveDataRequest, SensitiveValueHandleScope, evaluate_disclosure,
    evaluate_handle_use,
};

use std::collections::BTreeSet;

use originweave_core::{
    ActionRequest, ApprovalEvidence, ApprovalScope, Capability, ExecutionPurpose, ExtensionId,
    InstructionSource, PolicyContext, RiskClass, RobotsDecision, SecretDelivery, SessionMode,
};

/// Exact extension identities that may be present in one managed Agent Task profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTaskExtensionPolicy {
    managed_extensions: BTreeSet<ExtensionId>,
}

impl AgentTaskExtensionPolicy {
    /// Build one fail-closed Agent Task extension admission policy.
    ///
    /// Duplicate identifiers collapse to one exact managed identity. An empty
    /// iterator therefore represents the default policy that admits no extension.
    #[must_use]
    pub fn new<I>(managed_extensions: I) -> Self
    where
        I: IntoIterator<Item = ExtensionId>,
    {
        Self {
            managed_extensions: managed_extensions.into_iter().collect(),
        }
    }
}

/// Result of evaluating one extension identity for an isolated Agent Task profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskExtensionDecision {
    /// The exact canonical extension identity appears in the managed allow-list.
    AllowManagedExtension,
    /// The extension identity is absent from the managed allow-list.
    DenyNotManaged,
}

/// Evaluate extension admission without minting OriginWeave Agent capability.
///
/// This pure boundary answers only whether the exact canonical extension may be
/// present in the caller's managed Agent Task profile. Chromium permissions,
/// installation state, native messaging, and [`originweave_core::ExtensionAgentGrant`]
/// remain separate authorities.
#[must_use]
pub fn evaluate_agent_task_extension(
    extension_id: &ExtensionId,
    policy: &AgentTaskExtensionPolicy,
) -> AgentTaskExtensionDecision {
    if policy.managed_extensions.contains(extension_id) {
        AgentTaskExtensionDecision::AllowManagedExtension
    } else {
        AgentTaskExtensionDecision::DenyNotManaged
    }
}

/// The result of evaluating one typed action request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The action satisfies every deterministic policy gate.
    Allow,
    /// The action is blocked for a specific fail-closed reason.
    Deny(DenialReason),
    /// The action may proceed only after approval for the returned risk class.
    RequireApproval(RiskClass),
}

/// A stable reason that policy denied an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenialReason {
    /// Human mode does not grant autonomous agent control.
    HumanModeNotAgentControlled,
    /// The crawler execution mode and public-crawl purpose were not paired.
    ModePurposeMismatch,
    /// Page or document content attempted to become a trusted instruction.
    UntrustedInstructionSource,
    /// The session lacks the exact capability required by the action.
    MissingCapability(Capability),
    /// The target origin is outside the session's read grant.
    OriginNotReadable,
    /// Crawler mode attempted to mutate state.
    CrawlerMutation,
    /// A state-changing action attempted to cross an origin boundary.
    CrossOriginMutation,
    /// The target origin is outside the session's write grant.
    OriginNotWritable,
    /// Robots policy explicitly disallowed the public crawl.
    RobotsDisallowed,
    /// Robots policy could not be determined safely.
    RobotsUnknown,
    /// A public crawl was requested without a robots-policy decision.
    RobotsNotApplicable,
    /// A secret-fill action did not use an opaque broker handle.
    SecretBrokerRequired,
    /// Secret material accompanied an action that must not consume secrets.
    UnexpectedSecretMaterial,
    /// The action belongs to a non-delegable risk class.
    ForbiddenRisk,
    /// Approval evidence covers a different action, origin, or intent digest.
    ApprovalScopeMismatch,
}

/// Evaluate a typed browser action against one explicit policy context.
#[must_use]
pub fn evaluate(request: &ActionRequest, context: &PolicyContext) -> Decision {
    if context.mode() == SessionMode::Human {
        return Decision::Deny(DenialReason::HumanModeNotAgentControlled);
    }
    let crawler_mode = context.mode() == SessionMode::Crawler;
    let public_crawl = context.purpose() == ExecutionPurpose::PublicCrawl;
    if crawler_mode != public_crawl {
        return Decision::Deny(DenialReason::ModePurposeMismatch);
    }
    if request.instruction_source() == InstructionSource::WebContent {
        return Decision::Deny(DenialReason::UntrustedInstructionSource);
    }

    let capability = request.action().required_capability();
    if !context.capabilities().contains(&capability) {
        return Decision::Deny(DenialReason::MissingCapability(capability));
    }
    if !context.read_origins().contains(request.target_origin()) {
        return Decision::Deny(DenialReason::OriginNotReadable);
    }

    if request.action().mutates_state() {
        if crawler_mode {
            return Decision::Deny(DenialReason::CrawlerMutation);
        }
        if request.source_origin() != request.target_origin() {
            return Decision::Deny(DenialReason::CrossOriginMutation);
        }
        if !context.write_origins().contains(request.target_origin()) {
            return Decision::Deny(DenialReason::OriginNotWritable);
        }
    }

    if public_crawl {
        match context.robots_decision() {
            RobotsDecision::Allowed => {}
            RobotsDecision::Disallowed => {
                return Decision::Deny(DenialReason::RobotsDisallowed);
            }
            RobotsDecision::Unknown => {
                return Decision::Deny(DenialReason::RobotsUnknown);
            }
            RobotsDecision::NotApplicable => {
                return Decision::Deny(DenialReason::RobotsNotApplicable);
            }
        }
    }

    if request.action().uses_secret() {
        if request.secret_delivery() != SecretDelivery::BrokerHandle {
            return Decision::Deny(DenialReason::SecretBrokerRequired);
        }
    } else if request.secret_delivery() != SecretDelivery::None {
        return Decision::Deny(DenialReason::UnexpectedSecretMaterial);
    }

    let risk = request.action().risk_class();
    if risk == RiskClass::R5 {
        return Decision::Deny(DenialReason::ForbiddenRisk);
    }
    if !risk.requires_approval() {
        return Decision::Allow;
    }

    let required_scope = ApprovalScope::new(
        request.action(),
        request.target_origin().clone(),
        request.intent_digest().clone(),
    );
    match context.approval() {
        ApprovalEvidence::None => Decision::RequireApproval(risk),
        evidence if evidence.authorizes(&required_scope) => Decision::Allow,
        ApprovalEvidence::UserConfirmed(_) | ApprovalEvidence::EnterprisePolicy(_) => {
            Decision::Deny(DenialReason::ApprovalScopeMismatch)
        }
    }
}
