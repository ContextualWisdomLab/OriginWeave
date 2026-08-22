use std::fmt;

use crate::{
    BrowserAuthorityRegistry, BrowserRegistryError, NodeActionKind, ObservedNodeHandle,
    SemanticNodeObservation,
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

    /// Revalidate this target against current registry-owned browser authority before later use.
    ///
    /// The exact node binding must still be live in `registry`; a caller cannot revive a retired
    /// or stale target merely by presenting a self-consistent session/context/origin/epoch tuple.
    pub fn validate_current(
        &self,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<(), BrowserRegistryError> {
        registry.validate_node_handle(&self.handle)
    }
}

/// A bounded validation failure when deriving one semantic node action target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticNodeActionTargetError {
    /// The requested action was not advertised by the semantic observation.
    UnsupportedAction,
    /// The observation reported the target disabled for an interactive action.
    NodeNotEnabled,
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
        }
    }
}

impl std::error::Error for SemanticNodeActionTargetError {}
