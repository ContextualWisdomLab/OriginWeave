//! Shared security and governance contracts for OriginWeave.
//!
//! This crate keeps the long-lived value contracts in `contracts`, the
//! browser protocol/identifier boundaries and extension authority in focused modules so browser
//! adapters can evolve without turning raw CDP or WebDriver metadata into
//! OriginWeave authority.
//!
//! Raw adapter-local node identifiers must not be mintable through the public
//! registry API. Public callers must enter through the reviewed semantic-node
//! admission path instead:
//!
//! ```compile_fail
//! use originweave_core::{BrowserAuthorityRegistry, Origin};
//!
//! let mut registry = BrowserAuthorityRegistry::new();
//! let session = registry.register_session("webdriver-session")?;
//! let context = registry.register_context(session, "top-level-context")?;
//! let origin = Origin::parse("http://127.0.0.1:43127")?;
//! let _handle = registry.bind_node(session, context, &origin, "backend-node-17")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_authority_registry;
mod browser_protocol;
mod browser_protocol_dispatch;
mod browser_protocol_operation;
mod browser_registry;
#[cfg(test)]
mod browser_registry_coverage;
mod contract_errors;
mod contracts;
mod extension_authority;
mod webdriver_bidi_command;
mod webdriver_bidi_error_code;
mod webdriver_bidi_response_document;
mod webdriver_bidi_response_document_correlation;
mod webdriver_bidi_response_envelope;
mod webdriver_bidi_result;
mod webdriver_bidi_websocket_connect_target;
mod webdriver_bidi_websocket_endpoint;

pub use browser_authority_registry::BrowserAuthorityRegistry;
pub use browser_protocol::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability,
    BrowserProtocolCapabilityRequirementError, BrowserProtocolDescriptorError, BrowserProtocolKind,
    BrowserProtocolRuntimeRequirementError, BrowserProtocolUseValidationError,
    BrowserProtocolVersionRequirementError, MAX_BROWSER_PROTOCOL_METADATA_BYTES,
    OriginWeaveProtocolVersion, OriginWeaveProtocolVersionParseError, ValidatedBrowserProtocolUse,
};
pub use browser_protocol_dispatch::{
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserContextProtocolDispatchError,
    BrowserProtocolRuntimeMetadata,
};
pub use browser_protocol_operation::{
    BrowserProtocolOperation, MAX_BROWSER_ACCESSIBILITY_QUERY_NAME_BYTES,
    MAX_BROWSER_ACCESSIBILITY_QUERY_NODE_COUNT, MAX_BROWSER_ACCESSIBILITY_QUERY_ROLE_BYTES,
    WEBDRIVER_BIDI_LOCATE_NODES_METHOD, WEBDRIVER_BIDI_NODE_REMOTE_VALUE_TYPE,
    WEBDRIVER_BIDI_QUERY_INCLUDE_SHADOW_TREE, WEBDRIVER_BIDI_QUERY_MAX_DOM_DEPTH,
    WEBDRIVER_BIDI_QUERY_MAX_OBJECT_DEPTH, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiAccessibilityQueryError, WebDriverBiDiLocateNodesAdmissionError,
    WebDriverBiDiQueryNodesAdmissionError, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiRemoteNodeReferenceError,
};
pub(crate) use browser_registry::contains_disallowed_protocol_text;
pub use browser_registry::{
    BrowserRegistryError, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES,
    UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS,
};
pub use contracts::{
    ActionIntentDigest, ActionIntentDigestError, ActionKind, ActionRequest, ApprovalEvidence,
    ApprovalScope, BrowserSessionId, BrowsingContextId, Capability, DocumentEpoch,
    ExecutionPurpose, ExtensionAgentCapability, ExtensionId, ExtensionIdError, InstructionSource,
    NodeHandleError, ObservedNodeHandle, Origin, OriginError, PolicyContext, RiskClass,
    RobotsDecision, SecretDelivery, SessionMode,
};
pub use extension_authority::{
    AgentTaskId, AgentTaskIdError, ExtensionAccessDecision, ExtensionAccessRequest,
    ExtensionAgentGrant, evaluate_extension_access,
};
pub use webdriver_bidi_command::{
    CorrelatedWebDriverBiDiLocateNodesResponse, MAX_WEBDRIVER_BIDI_COMMAND_ID,
    ValidatedWebDriverBiDiLocateNodesResponse, WebDriverBiDiCommandResponseKind,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiLocateNodesCommandError,
    WebDriverBiDiLocateNodesResponseCorrelationError,
    WebDriverBiDiLocateNodesResponseEnvelopeError,
};
pub use webdriver_bidi_error_code::WebDriverBiDiErrorCode;
pub use webdriver_bidi_response_document::{
    BoundedWebDriverBiDiResponseDocument, MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES,
    WebDriverBiDiResponseDocumentAdmissionError,
};
pub use webdriver_bidi_response_document_correlation::WebDriverBiDiLocateNodesResponseDocumentError;
pub use webdriver_bidi_response_envelope::{
    MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH, MAX_WEBDRIVER_BIDI_RESPONSE_TOP_LEVEL_FIELDS,
    ParsedWebDriverBiDiCommandResponseEnvelope, WebDriverBiDiResponseEnvelopeParseError,
};
pub use webdriver_bidi_result::{
    ValidatedWebDriverBiDiLocateNodesResult, WebDriverBiDiLocateNodesResultAdmissionError,
};
pub use webdriver_bidi_websocket_connect_target::{
    VerifiedWebDriverBiDiSocketPeer, WebDriverBiDiSocketPeerVerificationError,
    WebDriverBiDiWebSocketConnectTarget, WebDriverBiDiWebSocketConnectTargetError,
};
pub use webdriver_bidi_websocket_endpoint::{
    CorrelatedWebDriverBiDiWebSocketEndpoint, MAX_WEBDRIVER_BIDI_WEBSOCKET_ENDPOINT_BYTES,
    WebDriverBiDiWebSocketEndpoint, WebDriverBiDiWebSocketEndpointAdmissionError,
    WebDriverBiDiWebSocketEndpointCorrelationError,
};
