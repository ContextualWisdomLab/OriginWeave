//! Shared security and governance contracts for OriginWeave.
//!
//! This crate keeps the long-lived value contracts in `contracts` and the
//! browser protocol/identifier boundaries in focused modules so browser
//! adapters can evolve without turning raw CDP or WebDriver metadata into
//! OriginWeave authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_protocol;
mod browser_registry;
#[cfg(test)]
mod browser_registry_coverage;
mod contracts;

pub use browser_protocol::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolDescriptorError,
    BrowserProtocolKind, MAX_BROWSER_PROTOCOL_METADATA_BYTES,
};
pub use browser_registry::{
    BrowserAuthorityRegistry, BrowserRegistryError, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES,
};
pub use contracts::*;
