//! Shared security and governance contracts for OriginWeave.
//!
//! This crate keeps the long-lived value contracts in `contracts`, the
//! protocol-identifier registry in a focused module, and bounded semantic
//! observations in a separate authority-preserving module.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_registry;
#[cfg(test)]
mod browser_registry_coverage;
mod contract_errors;
mod contracts;
mod semantic_observation;

pub use browser_registry::{
    BrowserAuthorityRegistry, BrowserRegistryError, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES,
};
pub use contracts::{
    ActionIntentDigest, ActionIntentDigestError, ActionKind, ActionRequest, ApprovalEvidence,
    ApprovalScope, BrowserSessionId, BrowsingContextId, Capability, DocumentEpoch,
    ExecutionPurpose, ExtensionAccessDecision, ExtensionAccessRequest, ExtensionAgentCapability,
    ExtensionAgentGrant, ExtensionId, ExtensionIdError, InstructionSource, NodeHandleError,
    ObservedNodeHandle, Origin, OriginError, PolicyContext, RiskClass, RobotsDecision,
    SecretDelivery, SessionMode, evaluate_extension_access,
};
pub use semantic_observation::{
    MAX_ACCESSIBLE_NAME_BYTES, MAX_SEMANTIC_CHILDREN, MAX_SEMANTIC_ROLE_BYTES,
    MAX_VISIBLE_TEXT_BYTES, NodeActionKind, ObservationChannel, SemanticNodeObservation,
    SemanticNodeObservationError, SemanticNodeObservationInput, SemanticNodeQuery,
    SemanticNodeQueryError,
};
