//! Shared security and governance contracts for OriginWeave.
//!
//! The crate keeps deterministic authority contracts independent from browser-engine
//! integration so the browser shell, headless runtime, MCP adapter, and enterprise
//! policy service can compose them without ambient authority inheritance.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod legacy_contracts;
pub use legacy_contracts::*;

mod native_messaging_manifest;
pub use native_messaging_manifest::{
    MAX_NATIVE_MESSAGING_ALLOWED_ORIGINS, NativeMessagingHostManifest,
    NativeMessagingHostManifestAccessDecision, NativeMessagingHostManifestError,
};
