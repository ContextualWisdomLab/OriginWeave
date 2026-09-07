use std::fmt;

use originweave_core::{
    AdmittedNodeAuthorityError, BrowserAuthorityRegistry, PolicyContext, RiskClass,
    SemanticNodeActionBinding,
};

use crate::{Decision, DenialReason, evaluate};

/// A semantic-node action that the deterministic action policy explicitly allowed.
///
/// Construction evaluates the exact [`SemanticNodeActionBinding`] request through the ordinary
/// OriginWeave action policy. The retained binding keeps the registry-issued node handle, exact
/// node-local action, and business request together, but this value does not grant current browser
/// authority, approval, destination authority, secret access, or execution success. The trusted
/// browser adapter must still revalidate its current registry-owned node authority immediately
/// before any side effect.
#[derive(Debug)]
pub struct PolicyAuthorizedSemanticNodeAction {
    binding: SemanticNodeActionBinding,
}

impl PolicyAuthorizedSemanticNodeAction {
    /// Evaluate the exact bound business request and retain it only after explicit policy allow.
    pub fn authorize(
        binding: SemanticNodeActionBinding,
        context: &PolicyContext,
    ) -> Result<Self, SemanticNodePolicyAuthorizationError> {
        match evaluate(binding.request(), context) {
            Decision::Allow => Ok(Self { binding }),
            Decision::Deny(reason) => Err(SemanticNodePolicyAuthorizationError::Denied(reason)),
            Decision::RequireApproval(risk) => {
                Err(SemanticNodePolicyAuthorizationError::ApprovalRequired(risk))
            }
        }
    }

    /// Return the exact registry-issued node, node-local action, and business request policy allowed.
    ///
    /// Possessing this binding proves only the deterministic policy decision made at construction
    /// time. Callers must not infer that the browser document or node authority is still current.
    #[must_use]
    pub const fn binding(&self) -> &SemanticNodeActionBinding {
        &self.binding
    }

    /// Revalidate the registry-owned browser authority immediately before later dispatch.
    ///
    /// The exact node binding retained by this policy-authorized action must still be live in the
    /// trusted adapter's current [`BrowserAuthorityRegistry`]. A caller-presented tuple cannot revive
    /// a retired, stale, forged, or cross-registry node authority. This check does not execute the
    /// browser action or prove its post-condition; those remain separate execution boundaries.
    pub fn validate_current(
        &self,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<(), AdmittedNodeAuthorityError> {
        self.binding.validate_current(registry)
    }

    /// Revalidate browser authority and invoke one adapter callback in the same call boundary.
    ///
    /// The callback is never invoked when the retained registry-issued node authority is stale,
    /// retired, forged, or belongs to another registry. Callback completion remains only adapter
    /// execution evidence: it does not grant destination, secret, or approval authority and does not
    /// prove the browser action's post-condition.
    pub fn dispatch_if_current<R, F>(
        &self,
        registry: &BrowserAuthorityRegistry,
        dispatch: F,
    ) -> Result<R, AdmittedNodeAuthorityError>
    where
        F: FnOnce(&SemanticNodeActionBinding) -> R,
    {
        self.validate_current(registry)
            .map(|()| dispatch(&self.binding))
    }
}

/// A fail-closed outcome that did not produce a policy-authorized semantic-node action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticNodePolicyAuthorizationError {
    /// Deterministic policy denied the exact business action request.
    Denied(DenialReason),
    /// Deterministic policy requires approval for the returned risk class before authorization.
    ApprovalRequired(RiskClass),
}

impl fmt::Display for SemanticNodePolicyAuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Denied(reason) => write!(
                formatter,
                "semantic node action denied by deterministic policy: {}",
                denial_reason_message(reason)
            ),
            Self::ApprovalRequired(risk) => write!(
                formatter,
                "semantic node action requires {risk:?} approval before policy authorization"
            ),
        }
    }
}

impl std::error::Error for SemanticNodePolicyAuthorizationError {}

fn denial_reason_message(reason: &DenialReason) -> &'static str {
    match reason {
        DenialReason::HumanModeNotAgentControlled => "human mode is not agent controlled",
        DenialReason::ModePurposeMismatch => "execution mode and purpose mismatch",
        DenialReason::UntrustedInstructionSource => "untrusted instruction source",
        DenialReason::MissingCapability(_) => "required capability is missing",
        DenialReason::OriginNotReadable => "target origin is not readable",
        DenialReason::CrawlerMutation => "crawler mutation is forbidden",
        DenialReason::CrossOriginMutation => "cross-origin mutation is forbidden",
        DenialReason::OriginNotWritable => "target origin is not writable",
        DenialReason::RobotsDisallowed => "robots policy disallows the crawl",
        DenialReason::RobotsUnknown => "robots policy is unknown",
        DenialReason::RobotsNotApplicable => "robots policy was not evaluated",
        DenialReason::SecretBrokerRequired => "secret broker handle is required",
        DenialReason::UnexpectedSecretMaterial => "unexpected secret material",
        DenialReason::ForbiddenRisk => "risk class is not delegable",
        DenialReason::ApprovalScopeMismatch => "approval scope does not match",
    }
}
