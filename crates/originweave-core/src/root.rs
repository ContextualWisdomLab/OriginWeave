//! Shared security and governance contracts for OriginWeave.
//!
//! The crate keeps deterministic authority contracts independent from browser-engine
//! integration so the browser shell, headless runtime, MCP adapter, and enterprise
//! policy service can compose them without ambient authority inheritance.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;

#[path = "lib.rs"]
mod legacy_contracts;
pub use legacy_contracts::*;

impl fmt::Display for NativeMessagingHostNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHostName => formatter.write_str(
                "native-messaging host name violates the reviewed Chrome identity syntax",
            ),
        }
    }
}

impl std::error::Error for NativeMessagingHostNameError {}

mod native_messaging_manifest;
pub use native_messaging_manifest::{
    MAX_NATIVE_MESSAGING_ALLOWED_ORIGINS, MAX_NATIVE_MESSAGING_EXECUTABLE_PATH_BYTES,
    NativeMessagingHostManifest, NativeMessagingHostManifestAccessDecision,
    NativeMessagingHostManifestError, NativeMessagingHostPlatform,
};
