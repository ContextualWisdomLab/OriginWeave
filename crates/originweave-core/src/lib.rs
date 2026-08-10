//! Shared security and governance contracts for OriginWeave.
//!
//! This crate keeps the long-lived value contracts in `contracts`, the
//! protocol-identifier registry in a focused module, and bounded semantic
//! observations in separate authority-preserving modules.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_registry;
#[cfg(test)]
mod browser_registry_coverage;
mod contracts;
mod semantic_action_target;
mod semantic_observation;

pub use browser_registry::{
    BrowserAuthorityRegistry, BrowserRegistryError, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES,
};
pub use contracts::*;
pub use semantic_action_target::{SemanticNodeActionTarget, SemanticNodeActionTargetError};
pub use semantic_observation::{
    MAX_ACCESSIBLE_NAME_BYTES, MAX_SEMANTIC_CHILDREN, MAX_SEMANTIC_ROLE_BYTES,
    MAX_VISIBLE_TEXT_BYTES, NodeActionKind, ObservationChannel, SemanticNodeObservation,
    SemanticNodeObservationError, SemanticNodeObservationInput, SemanticNodeQuery,
    SemanticNodeQueryError,
};
