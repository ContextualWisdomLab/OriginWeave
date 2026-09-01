use std::{error::Error, fmt};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, PolicyContext, RiskClass,
    SemanticNodeActionBinding, SemanticNodeActionTargetError, SemanticNodeObservation,
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

    /// Revalidate registry-owned browser authority and invoke one dispatch callback in the same call.
    ///
    /// The registry must be the trusted adapter's current authority registry for the action that is
    /// about to be dispatched. The callback is never invoked if the retained node binding is stale,
    /// retired, forged, or belongs to another registry. A successful callback invocation does not
    /// authenticate the adapter, grant destination, secret, or approval authority, or prove the
    /// action's post-condition; those remain separate execution boundaries.
    pub fn dispatch_if_current<R, F>(
        &self,
        registry: &BrowserAuthorityRegistry,
        dispatch: F,
    ) -> Result<R, BrowserRegistryError>
    where
        F: FnOnce(&SemanticNodeActionBinding) -> R,
    {
        self.validate_current(registry)
            .map(|()| dispatch(&self.binding))
    }

    /// Revalidate browser authority and one fresh semantic observation before dispatch.
    ///
    /// The caller must supply the trusted adapter's current authority registry and obtain
    /// `current_observation` from that same current browser boundary immediately before the side
    /// effect. Registry authority is checked first, followed by exact semantic node identity,
    /// advertised action, and enabled-state evidence. The callback is invoked only after both
    /// independent validations succeed. Adapter success remains distinct from post-condition proof.
    pub fn dispatch_if_current_observation<R, F>(
        &self,
        registry: &BrowserAuthorityRegistry,
        current_observation: &SemanticNodeObservation,
        dispatch: F,
    ) -> Result<R, SemanticNodeDispatchValidationError>
    where
        F: FnOnce(&SemanticNodeActionBinding) -> R,
    {
        self.validate_current(registry)
            .map_err(SemanticNodeDispatchValidationError::BrowserAuthority)?;
        self.binding
            .target()
            .validate_current_observation(current_observation)
            .map_err(SemanticNodeDispatchValidationError::SemanticState)?;
        Ok(dispatch(&self.binding))
    }
}

/// A fail-closed dispatch validation failure before any adapter callback can run.
#[derive(Debug)]
pub enum SemanticNodeDispatchValidationError {
    /// The registry no longer recognizes the exact browser-owned node binding.
    BrowserAuthority(BrowserRegistryError),
    /// The fresh semantic observation no longer matches the authorized target state.
    SemanticState(SemanticNodeActionTargetError),
}

impl fmt::Display for SemanticNodeDispatchValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrowserAuthority(_) => formatter
                .write_str("semantic node dispatch browser authority revalidation failed"),
            Self::SemanticState(_) => formatter
                .write_str("semantic node dispatch semantic-state revalidation failed"),
        }
    }
}

impl Error for SemanticNodeDispatchValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BrowserAuthority(source) => Some(source),
            Self::SemanticState(source) => Some(source),
        }
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
