//! Shared security and governance contracts for OriginWeave.
//!
//! The historical core contracts remain source-compatible while adapter-specific
//! boundaries can live in focused modules without changing their authority model.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod contracts;

pub use contracts::{
    ActionIntentDigest, ActionIntentDigestError, ActionKind, ActionRequest, AgentTaskId,
    AgentTaskIdError, ApprovalEvidence, ApprovalScope, BrowserAuthorityRegistry,
    BrowserRegistryError, BrowserSessionId, BrowsingContextId, Capability, DocumentEpoch,
    ExecutionPurpose, ExtensionAgentCapability, ExtensionId, ExtensionIdError, InstructionSource,
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, NodeHandleError, Origin, OriginError, PolicyContext,
    RegistryObservedNodeHandle as ObservedNodeHandle, RiskClass, RobotsDecision, SecretDelivery,
    SessionMode,
    AuthorityExtensionAccessDecision as ExtensionAccessDecision,
    AuthorityExtensionAccessRequest as ExtensionAccessRequest,
    AuthorityExtensionAgentGrant as ExtensionAgentGrant,
    evaluate_extension_authority_access as evaluate_extension_access,
};

/// Stateless MCP routing validation that maps only explicit tools to typed actions.
pub mod mcp;
