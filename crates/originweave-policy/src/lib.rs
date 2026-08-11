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

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, ApprovalScope, BrowserSessionId,
    BrowsingContextId, Capability, ExecutionPurpose, ExtensionAccessDecision, ExtensionAccessRequest,
    ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId, InstructionSource, Origin,
    PolicyContext, RiskClass, RobotsDecision, SecretDelivery, SessionMode, evaluate_extension_access,
};

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

/// Result of composing exact extension proposal authority with ordinary action policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionProposalDecision {
    /// The extension/session/context lacks exact typed-action proposal authority.
    ExtensionAccessDenied(ExtensionAccessDecision),
    /// Proposal authority was present; this is the unchanged ordinary action-policy result.
    ActionPolicy(Decision),
}

/// A typed action proposal derived from raw extension-produced message content.
///
/// This value intentionally has no instruction-source field. Raw extension messages are untrusted
/// observations regardless of the extension's Chrome permissions or OriginWeave proposal grant.
/// A separate trusted adapter must authenticate independent human or enterprise-policy provenance
/// before using any path that can construct a trusted [`ActionRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionMessageActionProposal {
    action: ActionKind,
    source_origin: Origin,
    target_origin: Origin,
    secret_delivery: SecretDelivery,
    intent_digest: ActionIntentDigest,
}

impl ExtensionMessageActionProposal {
    /// Construct one raw extension-message action proposal without granting instruction trust.
    #[must_use]
    pub const fn new(
        action: ActionKind,
        source_origin: Origin,
        target_origin: Origin,
        secret_delivery: SecretDelivery,
        intent_digest: ActionIntentDigest,
    ) -> Self {
        Self {
            action,
            source_origin,
            target_origin,
            secret_delivery,
            intent_digest,
        }
    }
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

/// Evaluate one extension-originated typed-action proposal without promoting transport authority.
///
/// This function checks the exact extension, browser-session, browsing-context and
/// [`ExtensionAgentCapability::ProposeTypedAction`] grant before ordinary action policy. When
/// extension access is allowed, the caller-supplied [`ActionRequest`] is evaluated unchanged, so
/// its instruction source, capability, origin, secret-delivery and approval requirements cannot be
/// minted or rewritten by extension transport. This function does not parse extension messages,
/// execute browser input, resolve secrets, or claim an action post-condition.
#[must_use]
pub fn evaluate_extension_action_proposal(
    extension_id: &ExtensionId,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    grant: Option<&ExtensionAgentGrant>,
    request: &ActionRequest,
    context: &PolicyContext,
) -> ExtensionProposalDecision {
    let access_request = ExtensionAccessRequest::new(
        extension_id.clone(),
        browser_session,
        browsing_context,
        ExtensionAgentCapability::ProposeTypedAction,
    );
    let access = evaluate_extension_access(&access_request, grant);
    if access != ExtensionAccessDecision::Allow {
        return ExtensionProposalDecision::ExtensionAccessDenied(access);
    }
    ExtensionProposalDecision::ActionPolicy(evaluate(request, context))
}

/// Evaluate a raw extension-message proposal as untrusted web content.
///
/// Exact extension/session/context proposal authority is still checked first by
/// [`evaluate_extension_action_proposal`]. The proposal is then converted internally into an
/// [`ActionRequest`] whose instruction source is always [`InstructionSource::WebContent`]. The
/// extension therefore cannot select human or enterprise instruction trust from message content.
/// This boundary does not authenticate an independently trusted human/policy source, parse Chrome
/// messages, execute input, resolve secrets, or verify action success.
#[must_use]
pub fn evaluate_extension_message_action_proposal(
    extension_id: &ExtensionId,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    grant: Option<&ExtensionAgentGrant>,
    proposal: &ExtensionMessageActionProposal,
    context: &PolicyContext,
) -> ExtensionProposalDecision {
    let request = ActionRequest::new(
        proposal.action,
        proposal.source_origin.clone(),
        proposal.target_origin.clone(),
        InstructionSource::WebContent,
        proposal.secret_delivery,
        proposal.intent_digest.clone(),
    );
    evaluate_extension_action_proposal(
        extension_id,
        browser_session,
        browsing_context,
        grant,
        &request,
        context,
    )
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
