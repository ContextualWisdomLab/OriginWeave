//! Deterministic, fail-closed policy evaluation for OriginWeave actions.
//!
//! The evaluator performs no I/O and never executes a browser action. Callers
//! must present a complete [`originweave_core::PolicyContext`] and may execute
//! only an explicit [`Decision::Allow`].

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod break_glass;
mod model_disclosure;
mod model_fallback;
mod model_output;
mod model_route;
mod sensitive_data;

pub use break_glass::{
    BreakGlassActorBinding, BreakGlassApprovalEvidence, BreakGlassValidityPolicy,
    SensitiveBreakGlassDecision, SensitiveBreakGlassRequest, SensitiveBreakGlassScope,
    evaluate_sensitive_break_glass,
};
pub use model_disclosure::{
    ModelDisclosureAlternative, ModelDisclosureDecision, ModelDisclosureNecessity,
    evaluate_full_field_model_disclosure,
};
pub use model_fallback::{
    ModelFallbackDecision, ModelFallbackRequest, ModelFallbackScope, ModelRouteAvailability,
    ModelRouteAvailabilityEvidence, evaluate_model_fallback,
};
pub use model_output::{
    ModelOutputDecision, ModelOutputRequest, ModelOutputScope, ModelOutputValidation,
    evaluate_model_output,
};
pub use model_route::{
    ModelInvocationDecision, ModelInvocationRequest, ModelInvocationScope, ModelRouteDecision,
    ModelRouteRequest, ModelRouteScope, evaluate_model_invocation, evaluate_model_route,
};
pub use sensitive_data::{
    DataClassification, DisclosureDecision, DisclosureScope, HandleRevocationReason,
    HandleUseDecision, HandleUseRequest, SensitiveDataAuthority, SensitiveDataRequest,
    SensitiveHandleUseReservation, SensitiveHandleUseState, SensitiveValueHandleScope,
    evaluate_disclosure, evaluate_handle_use,
};

use originweave_core::{
    ActionRequest, ApprovalEvidence, ApprovalScope, Capability, ExecutionPurpose,
    InstructionSource, PolicyContext, RiskClass, RobotsDecision, SecretDelivery, SessionMode,
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
