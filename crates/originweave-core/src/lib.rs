//! Shared security and governance contracts for OriginWeave.
//!
//! This crate keeps the long-lived value contracts in `contracts` and the
//! browser protocol/identifier boundaries in focused modules so browser
//! adapters can evolve without turning raw CDP or WebDriver metadata into
//! OriginWeave authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_protocol;
mod browser_protocol_dispatch;
mod browser_protocol_operation;
mod browser_registry;
#[cfg(test)]
mod browser_registry_coverage;
mod contracts;

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
    BrowserAuthorityRegistry, BrowserRegistryError, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES,
    UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS,
};
pub use contracts::*;
