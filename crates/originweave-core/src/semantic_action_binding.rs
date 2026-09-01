use std::fmt;

use crate::{ActionRequest, AdmittedNodeHandle};

/// One registry-issued browser node explicitly paired with the business action request it would serve.
///
/// The binding prevents a caller from independently validating one current browser node and a
/// different business intent and then combining them as if they originated from the same document.
/// It deliberately does not authorize policy, map browser-local input to a business risk class,
/// grant a destination, resolve secrets, or execute browser I/O. The later typed command boundary
/// still revalidates registry provenance and the exact admitted wire node immediately before I/O.
#[derive(Debug)]
pub struct SemanticNodeActionBinding {
    handle: AdmittedNodeHandle,
    request: ActionRequest,
}

impl SemanticNodeActionBinding {
    /// Bind one registry-issued admitted node to a business request from the same source origin.
    pub fn new(
        handle: AdmittedNodeHandle,
        request: ActionRequest,
    ) -> Result<Self, SemanticNodeActionBindingError> {
        if handle.origin() != request.source_origin() {
            return Err(SemanticNodeActionBindingError::SourceOriginMismatch);
        }
        Ok(Self { handle, request })
    }

    /// Return the exact registry-issued node retained for later immediate-use authority checks.
    #[must_use]
    pub const fn handle(&self) -> &AdmittedNodeHandle {
        &self.handle
    }

    /// Return the independently classified business action request.
    #[must_use]
    pub const fn request(&self) -> &ActionRequest {
        &self.request
    }
}

/// A bounded failure to pair admitted browser-node authority with a business action request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticNodeActionBindingError {
    /// The request claims a different source document origin than the admitted node.
    SourceOriginMismatch,
}

impl fmt::Display for SemanticNodeActionBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceOriginMismatch => formatter
                .write_str("admitted node origin does not match action request source origin"),
        }
    }
}

impl std::error::Error for SemanticNodeActionBindingError {}
