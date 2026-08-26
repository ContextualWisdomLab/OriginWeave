//! Shared security and governance contracts for OriginWeave.
//!
//! This crate keeps the long-lived value contracts in `contracts`, the
//! browser protocol/identifier boundaries and extension authority in focused
//! modules so browser adapters can evolve without turning raw CDP or WebDriver
//! metadata into OriginWeave authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_protocol;
mod browser_registry;
#[cfg(test)]
mod browser_registry_coverage;
mod contract_errors;
mod contracts;
mod extension_authority;

pub use browser_protocol::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolDescriptorError,
    BrowserProtocolKind, MAX_BROWSER_PROTOCOL_METADATA_BYTES,
};
pub use browser_registry::{
    BrowserAuthorityRegistry, BrowserRegistryError, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES,
    ObservedNodeHandle as RegistryObservedNodeHandle,
};
pub use contracts::{
    ActionIntentDigest, ActionIntentDigestError, ActionKind, ActionRequest, ApprovalEvidence,
    ApprovalScope, BrowserSessionId, BrowsingContextId, Capability, DocumentEpoch,
    ExecutionPurpose, ExtensionAccessDecision, ExtensionAccessRequest, ExtensionAgentCapability,
    ExtensionAgentGrant, ExtensionId, ExtensionIdError, InstructionSource, NodeHandleError,
    ObservedNodeHandle, Origin, OriginError, PolicyContext, RiskClass, RobotsDecision,
    SecretDelivery, SessionMode, evaluate_extension_access,
};
pub use extension_authority::{
    AgentTaskId, AgentTaskIdError, ExtensionAccessDecision as AuthorityExtensionAccessDecision,
    ExtensionAccessRequest as AuthorityExtensionAccessRequest,
    ExtensionAgentGrant as AuthorityExtensionAgentGrant,
    evaluate_extension_access as evaluate_extension_authority_access,
};
