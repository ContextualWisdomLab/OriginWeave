use std::fmt;

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeHandleError, Origin, PolicyContext,
    RiskClass, SemanticNodeActionBinding, SemanticNodeActionTargetError, SemanticNodeObservation,
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

    /// Revalidate browser session, context, origin, and document epoch immediately before dispatch.
    pub fn validate_current(
        &self,
        current_session: BrowserSessionId,
        current_context: BrowsingContextId,
        current_origin: &Origin,
        current_epoch: DocumentEpoch,
    ) -> Result<(), NodeHandleError> {
        self.binding.validate_current(
            current_session,
            current_context,
            current_origin,
            current_epoch,
        )
    }

    /// Revalidate exact browser authority and immediately invoke one adapter dispatch callback.
    ///
    /// The supplied session, context, origin, and document epoch must be trusted adapter state
    /// sampled for the action that is about to be dispatched. The callback is never invoked when
    /// that state no longer matches the semantic-node binding. A successful callback invocation
    /// does not authenticate the adapter, grant destination, secret, or approval authority, or
    /// prove the action's post-condition; those remain separate execution boundaries.
    pub fn dispatch_if_current<R, F>(
        &self,
        current_session: BrowserSessionId,
        current_context: BrowsingContextId,
        current_origin: &Origin,
        current_epoch: DocumentEpoch,
        dispatch: F,
    ) -> Result<R, NodeHandleError>
    where
        F: FnOnce(&SemanticNodeActionBinding) -> R,
    {
        self.validate_current(
            current_session,
            current_context,
            current_origin,
            current_epoch,
        )?;
        Ok(dispatch(&self.binding))
    }

    /// Revalidate one fresh semantic observation and immediately invoke the dispatch callback.
    ///
    /// The caller must obtain `current_observation` from a trusted browser adapter immediately
    /// before the side effect. The callback is not invoked when the observation describes a
    /// different OriginWeave-owned node, no longer advertises the selected node-local action, or
    /// reports the node disabled for an action that requires enabled state. This method does not
    /// obtain or authenticate the observation and does not prove execution success.
    pub fn dispatch_if_current_observation<R, F>(
        &self,
        current_observation: &SemanticNodeObservation,
        dispatch: F,
    ) -> Result<R, SemanticNodeActionTargetError>
    where
        F: FnOnce(&SemanticNodeActionBinding) -> R,
    {
        self.binding
            .target()
            .validate_current_observation(current_observation)?;
        Ok(dispatch(&self.binding))
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
