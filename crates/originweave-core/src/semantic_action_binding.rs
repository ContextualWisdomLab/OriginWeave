use std::fmt;

use crate::{
    ActionRequest, BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeHandleError, Origin,
    SemanticNodeActionTarget,
};

/// One semantic node target explicitly paired with the business action request it would serve.
///
/// The binding prevents independently validated browser-node authority and business intent from
/// being combined across different source documents. It does not grant policy authority, map a
/// node-local action to business risk, authorize a destination, or execute browser input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticNodeActionBinding {
    target: SemanticNodeActionTarget,
    request: ActionRequest,
}

impl SemanticNodeActionBinding {
    /// Bind a semantic node target to a business request from the same current document origin.
    pub fn new(
        target: SemanticNodeActionTarget,
        request: ActionRequest,
    ) -> Result<Self, SemanticNodeActionBindingError> {
        if target.handle().origin() != request.source_origin() {
            return Err(SemanticNodeActionBindingError::SourceOriginMismatch);
        }
        Ok(Self { target, request })
    }

    /// Return the exact authority-bound semantic node target.
    #[must_use]
    pub const fn target(&self) -> &SemanticNodeActionTarget {
        &self.target
    }

    /// Return the independently classified business action request.
    #[must_use]
    pub const fn request(&self) -> &ActionRequest {
        &self.request
    }

    /// Revalidate exact browser authority immediately before a later dispatch boundary.
    pub fn validate_current(
        &self,
        current_session: BrowserSessionId,
        current_context: BrowsingContextId,
        current_origin: &Origin,
        current_epoch: DocumentEpoch,
    ) -> Result<(), NodeHandleError> {
        self.target.validate_current(
            current_session,
            current_context,
            current_origin,
            current_epoch,
        )
    }
}

/// A bounded failure to pair browser-node authority with the requested business action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticNodeActionBindingError {
    /// The business request belongs to a different source origin than the observed node.
    SourceOriginMismatch,
}

impl fmt::Display for SemanticNodeActionBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceOriginMismatch => formatter
                .write_str("semantic node origin does not match action request source origin"),
        }
    }
}

impl std::error::Error for SemanticNodeActionBindingError {}
