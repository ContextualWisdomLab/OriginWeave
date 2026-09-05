use std::fmt;

use crate::{
    ActionRequest, AdmittedNodeAuthorityError, AdmittedNodeHandle, BrowserAuthorityRegistry,
};

/// A node-local typed action retained by an authorized semantic binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeActionKind {
    /// Activate the node using browser-native click semantics.
    Click,
    /// Insert bounded non-secret text using browser-native input semantics.
    TypeText,
    /// Select one option using browser-native selection semantics.
    SelectOption,
    /// Set a checkable control to an explicit checked state.
    SetChecked,
    /// Scroll the node into the viewport without activating it.
    ScrollIntoView,
}

/// One registry-issued browser node and local node action explicitly paired with the business
/// action request they would serve.
///
/// The binding prevents a caller from independently validating one current browser node, selecting
/// a different browser-local action at dispatch, and combining that side effect with a separately
/// authorized business intent. It deliberately does not authorize policy, map the node-local action
/// to a business risk class, grant a destination, resolve secrets, or execute browser I/O. The later
/// typed adapter boundary still revalidates registry provenance and the exact admitted wire node
/// immediately before I/O.
#[derive(Debug)]
pub struct SemanticNodeActionBinding {
    handle: AdmittedNodeHandle,
    node_action: NodeActionKind,
    request: ActionRequest,
}

impl SemanticNodeActionBinding {
    /// Bind one registry-issued admitted node and exact node-local action to a business request from
    /// the same source origin.
    pub fn new(
        handle: AdmittedNodeHandle,
        node_action: NodeActionKind,
        request: ActionRequest,
    ) -> Result<Self, SemanticNodeActionBindingError> {
        if handle.origin() != request.source_origin() {
            return Err(SemanticNodeActionBindingError::SourceOriginMismatch);
        }
        Ok(Self {
            handle,
            node_action,
            request,
        })
    }

    /// Return the exact registry-issued node retained for later immediate-use authority checks.
    #[must_use]
    pub const fn handle(&self) -> &AdmittedNodeHandle {
        &self.handle
    }

    /// Return the exact browser-local node action retained with the authorized business intent.
    #[must_use]
    pub const fn node_action(&self) -> NodeActionKind {
        self.node_action
    }

    /// Return the independently classified business action request.
    #[must_use]
    pub const fn request(&self) -> &ActionRequest {
        &self.request
    }

    /// Revalidate the exact retained admitted-node authority against the trusted current registry.
    ///
    /// This preserves opaque registry-instance provenance and current session/context/origin/document
    /// state. It does not validate the final adapter-local wire identifier or execute browser I/O;
    /// the typed adapter command constructor must still perform that immediate-use check.
    pub fn validate_current(
        &self,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<(), AdmittedNodeAuthorityError> {
        registry.validate_admitted_node_handle(&self.handle)
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
