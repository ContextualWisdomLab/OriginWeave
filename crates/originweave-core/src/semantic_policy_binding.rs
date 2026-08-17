use std::fmt;

use crate::{ActionRequest, SemanticNodeActionTarget};

/// One semantic browser target paired with the explicit policy request that governs its use.
///
/// The binding proves only that the policy request starts from the same canonical browser origin
/// that produced the semantic node. It deliberately does not infer an [`crate::ActionKind`], risk
/// class, capability, approval, instruction trust, secret delivery, or target origin from
/// node-local observation evidence. Cross-origin targets therefore remain explicit policy input
/// for the policy engine to evaluate rather than being silently rewritten here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticNodePolicyBinding {
    target: SemanticNodeActionTarget,
    request: ActionRequest,
}

impl SemanticNodePolicyBinding {
    /// Bind one semantic target to an explicit policy request from the same browser origin.
    pub fn new(
        target: SemanticNodeActionTarget,
        request: ActionRequest,
    ) -> Result<Self, SemanticNodePolicyBindingError> {
        if target.handle().origin() != request.source_origin() {
            return Err(SemanticNodePolicyBindingError::SourceOriginMismatch);
        }
        Ok(Self { target, request })
    }

    /// Return the exact authority-bound semantic target supplied by the caller.
    #[must_use]
    pub const fn target(&self) -> &SemanticNodeActionTarget {
        &self.target
    }

    /// Return the complete explicit policy request supplied by the caller.
    #[must_use]
    pub const fn request(&self) -> &ActionRequest {
        &self.request
    }
}

/// A fail-closed validation error while pairing semantic target evidence with policy input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticNodePolicyBindingError {
    /// The policy request claims a browser source origin different from the observed node origin.
    SourceOriginMismatch,
}

impl fmt::Display for SemanticNodePolicyBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceOriginMismatch => formatter
                .write_str("semantic node origin does not match the policy request source origin"),
        }
    }
}

impl std::error::Error for SemanticNodePolicyBindingError {}
