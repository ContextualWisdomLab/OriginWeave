//! Extension-policy contracts for OriginWeave's Chromium compatibility boundary.
//!
//! This crate owns validated native-messaging host-manifest semantics. Stable identity and
//! request value objects remain in `originweave-core`; this context depends inward on those
//! contracts and does not grant process, browser-action, secret, or Agent authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub use originweave_core::{
    ExtensionId, NativeMessagingAccessRequest, NativeMessagingHostName, NativeMessagingHostNameError,
};

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
