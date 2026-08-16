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

/// Maximum UTF-8 byte length accepted for one declared native-host executable path.
///
/// This 32 KiB value is an OriginWeave allocation safety budget, not a Chrome or operating-
/// system path-validity limit. Runtime adapters remain responsible for platform-native path
/// resolution, canonicalization, ownership, and executable identity checks.
pub const MAX_NATIVE_MESSAGING_EXECUTABLE_PATH_BYTES: usize = 32 * 1024;

/// Operating-system path semantics used by one native-messaging host manifest.
///
/// Chrome requires absolute native-host paths on Linux and macOS, while Windows also allows
/// paths relative to the manifest directory. OriginWeave records the platform explicitly so
/// later runtime adapters cannot reinterpret a validated path under different semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingHostPlatform {
    /// Linux native-messaging host-manifest semantics.
    Linux,
    /// macOS native-messaging host-manifest semantics.
    MacOs,
    /// Windows native-messaging host-manifest semantics.
    Windows,
}

/// Validated authority-bearing fields from one Chrome native-messaging host manifest.
///
/// The record contains the exact host identity, declared executable-path text and platform,
/// plus exact Chromium extension identities named by the manifest's `allowed_origins`.
/// Possessing this value is not proof of manifest installation, path canonicalization,
/// executable existence or ownership, process identity, message provenance, or Agent
/// authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMessagingHostManifest {
    host_name: NativeMessagingHostName,
    platform: NativeMessagingHostPlatform,
    executable_path: String,
    allowed_extensions: BTreeSet<ExtensionId>,
}

impl NativeMessagingHostManifest {
    /// Validate the authority-bearing host-manifest fields without widening them.
    ///
    /// `interface_type` must be exactly `stdio`. Linux and macOS executable paths must be
    /// absolute, matching Chrome's native-messaging contract; Windows relative paths remain
    /// relative and must be resolved by a trusted runtime adapter against the authenticated
    /// manifest directory. Empty paths, embedded NUL bytes, and paths exceeding the
    /// OriginWeave allocation budget are rejected before storage on every platform. Every
    /// allowed origin must be exactly `chrome-extension://<canonical-extension-id>/`;
    /// alternate schemes, wildcards, suffix paths, query strings, fragments, and
    /// non-canonical extension identities are rejected rather than normalized. The raw list
    /// is bounded before deduplication.
    pub fn parse(
        host_name: NativeMessagingHostName,
        platform: NativeMessagingHostPlatform,
        executable_path: &str,
        interface_type: &str,
        allowed_origins: &[&str],
    ) -> Result<Self, NativeMessagingHostManifestError> {
        if interface_type != "stdio" {
            return Err(NativeMessagingHostManifestError::UnsupportedInterfaceType);
        }
        validate_executable_path(platform, executable_path)?;
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
            platform,
            executable_path: executable_path.to_owned(),
            allowed_extensions,
        })
    }

    /// Return the exact native-messaging host identity declared by the manifest.
    #[must_use]
    pub const fn host_name(&self) -> &NativeMessagingHostName {
        &self.host_name
    }

    /// Return the platform whose path semantics were used to validate the manifest.
    #[must_use]
    pub const fn platform(&self) -> NativeMessagingHostPlatform {
        self.platform
    }

    /// Return the exact executable-path text declared by the manifest.
    ///
    /// Windows relative paths are intentionally not resolved here because safe resolution
    /// requires the authenticated manifest location. The returned path therefore carries no
    /// filesystem-existence, canonicalization, ownership, or process-identity claim.
    #[must_use]
    pub fn executable_path(&self) -> &str {
        &self.executable_path
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

fn validate_executable_path(
    platform: NativeMessagingHostPlatform,
    executable_path: &str,
) -> Result<(), NativeMessagingHostManifestError> {
    if executable_path.is_empty() || executable_path.contains('\0') {
        return Err(NativeMessagingHostManifestError::InvalidExecutablePath);
    }
    if executable_path.len() > MAX_NATIVE_MESSAGING_EXECUTABLE_PATH_BYTES {
        return Err(NativeMessagingHostManifestError::ExecutablePathTooLong);
    }
    if platform != NativeMessagingHostPlatform::Windows && !executable_path.starts_with('/') {
        return Err(NativeMessagingHostManifestError::RelativeExecutablePathUnsupported);
    }
    Ok(())
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
    /// The manifest executable-path text was empty or contained an embedded NUL byte.
    InvalidExecutablePath,
    /// The manifest executable-path text exceeded the OriginWeave allocation safety budget.
    ExecutablePathTooLong,
    /// A non-Windows manifest used a relative executable path.
    RelativeExecutablePathUnsupported,
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
            Self::InvalidExecutablePath => formatter
                .write_str("native messaging host manifest contains an invalid executable path"),
            Self::ExecutablePathTooLong => formatter.write_str(
                "native messaging host manifest executable path exceeds the OriginWeave safety budget",
            ),
            Self::RelativeExecutablePathUnsupported => formatter
                .write_str("native messaging host executable path must be absolute on this platform"),
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
