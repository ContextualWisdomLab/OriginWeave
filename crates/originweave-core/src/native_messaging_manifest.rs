//! Deterministic authority extracted from one validated Chrome native-messaging host manifest.
//!
//! This module validates caller-supplied manifest fields only. It does not prove that a
//! manifest is installed, that an executable path is owned by a trusted principal, or that
//! any process attached to stdio is the host named by the manifest. Runtime adapters must
//! establish those boundaries independently before composing this evidence with process
//! authority.

use std::collections::BTreeSet;
use std::fmt;

use crate::{ExtensionId, NativeMessagingAccessRequest, NativeMessagingHostName};

/// Maximum number of raw `allowed_origins` entries accepted from one host manifest.
///
/// Chrome does not define this OriginWeave-specific safety budget. The limit bounds work
/// before duplicate origins are collapsed and therefore prevents a syntactically valid
/// manifest from turning policy admission into unbounded allocation or comparison work.
pub const MAX_NATIVE_MESSAGING_ALLOWED_ORIGINS: usize = 256;

/// Validated authority-bearing fields from one Chrome native-messaging host manifest.
///
/// The record contains only the exact host identity and exact Chromium extension identities
/// named by the manifest's `allowed_origins`. Possessing this value is not proof of manifest
/// installation, executable ownership, process identity, message provenance, or Agent
/// authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMessagingHostManifest {
    host_name: NativeMessagingHostName,
    allowed_extensions: BTreeSet<ExtensionId>,
}

impl NativeMessagingHostManifest {
    /// Validate the authority-bearing host-manifest fields without widening them.
    ///
    /// `interface_type` must be exactly `stdio`. Every allowed origin must be exactly
    /// `chrome-extension://<canonical-extension-id>/`; alternate schemes, wildcards,
    /// suffix paths, query strings, fragments, and non-canonical extension identities are
    /// rejected rather than normalized. The raw list is bounded before deduplication.
    pub fn parse(
        host_name: NativeMessagingHostName,
        interface_type: &str,
        allowed_origins: &[&str],
    ) -> Result<Self, NativeMessagingHostManifestError> {
        if interface_type != "stdio" {
            return Err(NativeMessagingHostManifestError::UnsupportedInterfaceType);
        }
        if allowed_origins.is_empty() {
            return Err(NativeMessagingHostManifestError::MissingAllowedOrigin);
        }
        if allowed_origins.len() > MAX_NATIVE_MESSAGING_ALLOWED_ORIGINS {
            return Err(NativeMessagingHostManifestError::TooManyAllowedOrigins);
        }

        let mut allowed_extensions = BTreeSet::new();
        for origin in allowed_origins {
            allowed_extensions.insert(parse_extension_origin(origin)?);
        }

        Ok(Self {
            host_name,
            allowed_extensions,
        })
    }

    /// Return the exact native-messaging host identity declared by the manifest.
    #[must_use]
    pub const fn host_name(&self) -> &NativeMessagingHostName {
        &self.host_name
    }

    /// Return the number of distinct exact extension identities explicitly allowed.
    #[must_use]
    pub fn allowed_extension_count(&self) -> usize {
        self.allowed_extensions.len()
    }

    /// Evaluate one native-messaging request against this exact manifest authority.
    ///
    /// Host identity is checked before extension membership. An `Allow` result means only
    /// that the already-validated manifest fields name the exact request; it does not mint
    /// Agent authority or attest the installed host process.
    #[must_use]
    pub fn evaluate(
        &self,
        request: &NativeMessagingAccessRequest,
    ) -> NativeMessagingHostManifestAccessDecision {
        if request.host_name() != &self.host_name {
            return NativeMessagingHostManifestAccessDecision::DenyHostMismatch;
        }
        if !self.allowed_extensions.contains(request.extension_id()) {
            return NativeMessagingHostManifestAccessDecision::DenyExtensionNotAllowed;
        }
        NativeMessagingHostManifestAccessDecision::Allow
    }
}

fn parse_extension_origin(origin: &str) -> Result<ExtensionId, NativeMessagingHostManifestError> {
    let Some(extension_text) = origin.strip_prefix("chrome-extension://") else {
        return Err(NativeMessagingHostManifestError::InvalidAllowedOrigin);
    };
    let Some(extension_text) = extension_text.strip_suffix('/') else {
        return Err(NativeMessagingHostManifestError::InvalidAllowedOrigin);
    };
    ExtensionId::parse(extension_text)
        .map_err(|_error| NativeMessagingHostManifestError::InvalidAllowedOrigin)
}

/// Result of matching one native-messaging request to validated host-manifest authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingHostManifestAccessDecision {
    /// The manifest names the exact requested host and explicitly allows the extension.
    Allow,
    /// The request names a different host from the validated manifest.
    DenyHostMismatch,
    /// The exact requesting extension is absent from the manifest allow-list.
    DenyExtensionNotAllowed,
}

/// Failure to validate authority-bearing fields from a native-messaging host manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingHostManifestError {
    /// The manifest interface type was not exactly Chrome's `stdio` value.
    UnsupportedInterfaceType,
    /// The manifest did not explicitly allow any extension origin.
    MissingAllowedOrigin,
    /// The raw allowed-origin list exceeded the OriginWeave admission safety budget.
    TooManyAllowedOrigins,
    /// An allowed origin was not one exact canonical Chromium extension origin.
    InvalidAllowedOrigin,
}

impl fmt::Display for NativeMessagingHostManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedInterfaceType => formatter
                .write_str("native messaging host manifest interface type must be stdio"),
            Self::MissingAllowedOrigin => formatter.write_str(
                "native messaging host manifest must allow at least one exact extension origin",
            ),
            Self::TooManyAllowedOrigins => formatter.write_str(
                "native messaging host manifest exceeds the OriginWeave allowed-origin safety budget",
            ),
            Self::InvalidAllowedOrigin => formatter
                .write_str("native messaging host manifest contains an invalid extension origin"),
        }
    }
}

impl std::error::Error for NativeMessagingHostManifestError {}
