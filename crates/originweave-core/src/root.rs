//! Shared security and governance contracts for OriginWeave.
//!
//! The historical core contracts remain source-compatible while adapter-specific
//! boundaries can live in focused modules without changing their authority model.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod core_contracts;
use core_contracts as contracts;

pub use core_contracts::{
    ActionIntentDigest, ActionIntentDigestError, ActionKind, ActionRequest, AgentTaskId,
    AgentTaskIdError, ApprovalEvidence, ApprovalScope,
    AuthorityExtensionAccessDecision as ExtensionAccessDecision,
    AuthorityExtensionAccessRequest as ExtensionAccessRequest,
    AuthorityExtensionAgentGrant as ExtensionAgentGrant, BrowserAuthorityRegistry,
    BrowserRegistryError, BrowserSessionId, BrowsingContextId, Capability, DocumentEpoch,
    ExecutionPurpose, ExtensionAgentCapability, ExtensionId, ExtensionIdError, InstructionSource,
    MAX_ACCESSIBLE_NAME_BYTES, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, MAX_SEMANTIC_CHILDREN,
    MAX_SEMANTIC_ROLE_BYTES, MAX_VISIBLE_TEXT_BYTES, NodeActionKind, NodeHandleError,
    ObservationChannel, Origin, OriginError, PolicyContext,
    RegistryObservedNodeHandle as ObservedNodeHandle, RiskClass, RobotsDecision, SecretDelivery,
    SemanticNodeObservation, SemanticNodeObservationError, SemanticNodeObservationInput,
    SessionMode, evaluate_extension_authority_access as evaluate_extension_access,
};

/// Stateless MCP routing validation that maps only explicit tools to typed actions.
pub mod mcp;
