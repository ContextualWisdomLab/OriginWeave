use std::fmt;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, PolicyContext, RiskClass,
    SemanticNodeActionBinding,
};

use crate::{Decision, DenialReason, evaluate};

/// A semantic-node action that the deterministic action policy explicitly allowed.
///
/// Construction evaluates the exact [`SemanticNodeActionBinding`] request through the ordinary
/// OriginWeave action policy. This value does not grant browser authority, approval, destination
/// authority, secret access, or execution success. Callers must still revalidate the bound browser
/// authority immediately before dispatch and satisfy every later execution boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Return the exact semantic-node target and business request that policy allowed together.
    #[must_use]
    pub const fn binding(&self) -> &SemanticNodeActionBinding {
        &self.binding
    }

    /// Revalidate registry-owned browser authority immediately before dispatch.
    ///
    /// The exact node binding retained by the policy-authorized action must still be live in the
    /// supplied registry. Caller-presented session/context/origin/epoch tuples cannot revive a
    /// retired, stale, forged, or cross-registry node target.
    pub fn validate_current(
        &self,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<(), BrowserRegistryError> {
        self.binding.validate_current(registry)
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
