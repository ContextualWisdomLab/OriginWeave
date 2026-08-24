//! Shared security and governance contracts for OriginWeave.
//!
//! The historical core contracts remain source-compatible while adapter-specific
//! boundaries can live in focused modules without changing their authority model.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod contracts;

pub use contracts::*;

/// Stateless MCP routing validation that maps only explicit tools to typed actions.
pub mod mcp;

mod native_messaging;
pub use native_messaging::*;

mod native_messaging_manifest;
pub use native_messaging_manifest::{
    MAX_NATIVE_MESSAGING_ALLOWED_ORIGINS, MAX_NATIVE_MESSAGING_EXECUTABLE_PATH_BYTES,
    NativeMessagingHostManifest, NativeMessagingHostManifestAccessDecision,
    NativeMessagingHostManifestError, NativeMessagingHostPlatform,
};

mod native_messaging_manifest_document;
pub use native_messaging_manifest_document::{
    MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES, NativeMessagingManifestDocument,
    NativeMessagingManifestDocumentError, NativeMessagingManifestParseError,
};
