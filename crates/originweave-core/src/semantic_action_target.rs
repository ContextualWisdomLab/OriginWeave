use std::fmt;

use crate::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeActionKind, NodeHandleError,
    ObservedNodeHandle, Origin, SemanticNodeObservation,
};

/// One node-local action bound to the exact browser authority that produced its observation.
///
/// This value narrows descriptive observation evidence into a stale-checkable action target. It
/// does not grant policy authority, classify business risk, or execute browser input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticNodeActionTarget {
    handle: ObservedNodeHandle,
    action: NodeActionKind,
}

impl SemanticNodeActionTarget {
    /// Construct a target only when the observation advertises a currently coherent node action.
    pub fn from_observation(
        observation: &SemanticNodeObservation,
        action: NodeActionKind,
    ) -> Result<Self, SemanticNodeActionTargetError> {
        if !observation.supported_actions().contains(&action) {
            return Err(SemanticNodeActionTargetError::UnsupportedAction);
        }
        if action != NodeActionKind::ScrollIntoView && !observation.is_enabled() {
            return Err(SemanticNodeActionTargetError::NodeNotEnabled);
        }
        Ok(Self {
            handle: observation.handle().clone(),
            action,
        })
    }

    /// Return the exact OriginWeave-owned node handle retained by this target.
    #[must_use]
    pub const fn handle(&self) -> &ObservedNodeHandle {
        &self.handle
    }

    /// Return the descriptive node-local action selected from the observation.
    #[must_use]
    pub const fn action(&self) -> NodeActionKind {
        self.action
    }

    /// Revalidate session, context, origin, and document authority immediately before later use.
    pub fn validate_current(
        &self,
        current_session: BrowserSessionId,
        current_context: BrowsingContextId,
        current_origin: &Origin,
        current_epoch: DocumentEpoch,
    ) -> Result<(), NodeHandleError> {
        self.handle.validate_current(
            current_session,
            current_context,
            current_origin,
            current_epoch,
        )
    }

    /// Revalidate this target against one freshly observed exact semantic node.
    ///
    /// The caller is responsible for obtaining the current observation from a trusted adapter
    /// immediately before use. This check prevents an older target from ignoring changed node
    /// identity, supported-action, or enabled-state evidence.
    pub fn validate_current_observation(
        &self,
        current_observation: &SemanticNodeObservation,
    ) -> Result<(), SemanticNodeActionTargetError> {
        if current_observation.handle() != &self.handle {
            return Err(SemanticNodeActionTargetError::ObservationAuthorityMismatch);
        }
        if !current_observation.supported_actions().contains(&self.action) {
            return Err(SemanticNodeActionTargetError::UnsupportedAction);
        }
        if self.action != NodeActionKind::ScrollIntoView && !current_observation.is_enabled() {
            return Err(SemanticNodeActionTargetError::NodeNotEnabled);
        }
        Ok(())
    }
}

/// A bounded validation failure when deriving or revalidating one semantic node action target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticNodeActionTargetError {
    /// The requested action was not advertised by the semantic observation.
    UnsupportedAction,
    /// The observation reported the target disabled for an interactive action.
    NodeNotEnabled,
    /// The current observation describes a different OriginWeave-owned node authority.
    ObservationAuthorityMismatch,
}

impl fmt::Display for SemanticNodeActionTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedAction => {
                formatter.write_str("semantic node action is not advertised by the observation")
            }
            Self::NodeNotEnabled => {
                formatter.write_str("semantic node is not enabled for the requested action")
            }
            Self::ObservationAuthorityMismatch => {
                formatter.write_str("current semantic observation does not match the action target")
            }
        }
    }
}

impl std::error::Error for SemanticNodeActionTargetError {}
