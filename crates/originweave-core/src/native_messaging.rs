//! Explicit Chrome native-messaging host authority without ambient Agent authority.

use std::fmt;

use crate::ExtensionId;

const MAX_NATIVE_MESSAGING_HOST_NAME_BYTES: usize = 256;

/// A canonical Chrome native-messaging host name admitted to OriginWeave policy.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NativeMessagingHostName {
    canonical: String,
}

impl NativeMessagingHostName {
    /// Parse the host name syntax accepted by Chrome native-messaging manifests.
    ///
    /// Host names are exact identities rather than display labels: only lowercase
    /// ASCII alphanumeric characters, underscores, and dots are accepted. Dots
    /// cannot lead, trail, or appear consecutively.
    pub fn parse(input: &str) -> Result<Self, NativeMessagingHostNameError> {
        if input.is_empty()
            || input.len() > MAX_NATIVE_MESSAGING_HOST_NAME_BYTES
            || input.starts_with('.')
            || input.ends_with('.')
            || input.contains("..")
            || !input.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'.'
            })
        {
            return Err(NativeMessagingHostNameError::InvalidHostName);
        }
        Ok(Self {
            canonical: input.to_owned(),
        })
    }

    /// Return the validated native-messaging host name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
}

/// A validation error for a Chrome native-messaging host name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingHostNameError {
    /// The value violated Chrome's native-messaging host-name syntax.
    InvalidHostName,
}

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

/// One explicit host-managed allow-list entry for a Chromium extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMessagingHostGrant {
    extension_id: ExtensionId,
    host_name: NativeMessagingHostName,
}

impl NativeMessagingHostGrant {
    /// Build one exact extension-to-native-host allow-list entry.
    #[must_use]
    pub const fn new(extension_id: ExtensionId, host_name: NativeMessagingHostName) -> Self {
        Self {
            extension_id,
            host_name,
        }
    }

    /// Return the extension identity granted native-messaging access.
    #[must_use]
    pub const fn extension_id(&self) -> &ExtensionId {
        &self.extension_id
    }

    /// Return the exact native-messaging host identity in this grant.
    #[must_use]
    pub const fn host_name(&self) -> &NativeMessagingHostName {
        &self.host_name
    }
}

/// One extension request to connect to an exact native-messaging host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMessagingAccessRequest {
    extension_id: ExtensionId,
    host_name: NativeMessagingHostName,
}

impl NativeMessagingAccessRequest {
    /// Build one native-messaging access request without granting process authority.
    #[must_use]
    pub const fn new(extension_id: ExtensionId, host_name: NativeMessagingHostName) -> Self {
        Self {
            extension_id,
            host_name,
        }
    }

    /// Return the extension identity requesting native-messaging access.
    #[must_use]
    pub const fn extension_id(&self) -> &ExtensionId {
        &self.extension_id
    }

    /// Return the exact native-messaging host identity requested.
    #[must_use]
    pub const fn host_name(&self) -> &NativeMessagingHostName {
        &self.host_name
    }
}

/// Result of evaluating native-messaging access against one explicit host grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingAccessDecision {
    /// The exact extension identity and native host name are explicitly granted.
    Allow,
    /// No explicit host-managed native-messaging grant was supplied.
    DenyMissingGrant,
    /// The request belongs to a different extension identity.
    DenyExtensionMismatch,
    /// The request names a different native-messaging host.
    DenyHostMismatch,
}

/// Evaluate one exact native-messaging request without minting Agent authority.
///
/// This deterministic primitive models one entry in the native host's explicit
/// extension allow-list. It deliberately does not launch a process, resolve a
/// host path, parse messages, or convert Chrome's `nativeMessaging` permission
/// into an OriginWeave Agent capability. Those remain separate adapter and policy
/// boundaries.
#[must_use]
pub fn evaluate_native_messaging_access(
    request: &NativeMessagingAccessRequest,
    grant: Option<&NativeMessagingHostGrant>,
) -> NativeMessagingAccessDecision {
    let Some(grant) = grant else {
        return NativeMessagingAccessDecision::DenyMissingGrant;
    };
    if request.extension_id != grant.extension_id {
        return NativeMessagingAccessDecision::DenyExtensionMismatch;
    }
    if request.host_name != grant.host_name {
        return NativeMessagingAccessDecision::DenyHostMismatch;
    }
    NativeMessagingAccessDecision::Allow
}
