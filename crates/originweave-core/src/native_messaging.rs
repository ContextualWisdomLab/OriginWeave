//! Explicit Chrome native-messaging host authority without ambient Agent authority.

use std::fmt;

use crate::ExtensionId;

const MAX_NATIVE_MESSAGING_HOST_NAME_BYTES: usize = 256;
const HOST_TO_BROWSER_NATIVE_MESSAGING_LIMIT: usize = 1_048_576;
const BROWSER_TO_HOST_NATIVE_MESSAGING_LIMIT: usize = 67_108_864;

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

/// Direction of one Chrome native-messaging frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingFrameDirection {
    /// A frame written by a native host for delivery to the browser.
    HostToBrowser,
    /// A frame written by the browser for delivery to a native host.
    BrowserToHost,
}

/// Failure to encode or decode a bounded Chrome native-messaging frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingFrameError {
    /// Fewer than four bytes were available for the native-endian length prefix.
    MissingLengthPrefix,
    /// The advertised or supplied payload exceeds the limit for its direction.
    PayloadTooLarge,
    /// The complete frame length differs from the advertised payload length.
    LengthMismatch,
    /// The framed payload is not valid UTF-8 text.
    InvalidUtf8Payload,
}

impl fmt::Display for NativeMessagingFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLengthPrefix => {
                formatter.write_str("native messaging frame is missing its 32-bit length prefix")
            }
            Self::PayloadTooLarge => {
                formatter.write_str("native messaging payload exceeds the direction-specific limit")
            }
            Self::LengthMismatch => {
                formatter.write_str("native messaging frame length does not match its prefix")
            }
            Self::InvalidUtf8Payload => {
                formatter.write_str("native messaging payload is not valid UTF-8")
            }
        }
    }
}

impl std::error::Error for NativeMessagingFrameError {}

/// Return Chrome's payload ceiling for one native-messaging direction.
#[must_use]
pub const fn native_messaging_payload_limit(direction: NativeMessagingFrameDirection) -> usize {
    match direction {
        NativeMessagingFrameDirection::HostToBrowser => HOST_TO_BROWSER_NATIVE_MESSAGING_LIMIT,
        NativeMessagingFrameDirection::BrowserToHost => BROWSER_TO_HOST_NATIVE_MESSAGING_LIMIT,
    }
}

/// Encode one complete native-messaging frame with a native-endian 32-bit length prefix.
///
/// The payload is rejected before allocation when it exceeds the direction-specific
/// Chrome limit. The returned bytes are framing only and carry no trust or Agent authority.
pub fn encode_native_messaging_frame(
    direction: NativeMessagingFrameDirection,
    payload: &[u8],
) -> Result<Vec<u8>, NativeMessagingFrameError> {
    if payload.len() > native_messaging_payload_limit(direction) {
        return Err(NativeMessagingFrameError::PayloadTooLarge);
    }

    let payload_length = payload.len() as u32;
    let mut frame = Vec::with_capacity(payload.len() + 4);
    frame.extend_from_slice(&payload_length.to_ne_bytes());
    frame.extend_from_slice(payload);
    Ok(frame)
}

/// Decode one complete bounded native-messaging frame without allocating its payload.
///
/// Oversized advertised lengths are rejected before payload slicing. The frame must
/// contain exactly the advertised payload bytes; truncation and trailing data fail closed.
pub fn decode_native_messaging_frame(
    direction: NativeMessagingFrameDirection,
    frame: &[u8],
) -> Result<&[u8], NativeMessagingFrameError> {
    if frame.len() < 4 {
        return Err(NativeMessagingFrameError::MissingLengthPrefix);
    }

    let advertised_length = u32::from_ne_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
    if advertised_length > native_messaging_payload_limit(direction) {
        return Err(NativeMessagingFrameError::PayloadTooLarge);
    }
    if frame.len() != advertised_length + 4 {
        return Err(NativeMessagingFrameError::LengthMismatch);
    }

    Ok(&frame[4..])
}

/// Decode one bounded native-messaging frame and validate its payload as UTF-8 text.
///
/// This validates only framing and UTF-8 encoding. JSON syntax, message provenance, and
/// any Agent authority remain separate fail-closed boundaries for a later adapter.
pub fn decode_native_messaging_text_frame(
    direction: NativeMessagingFrameDirection,
    frame: &[u8],
) -> Result<&str, NativeMessagingFrameError> {
    let payload = decode_native_messaging_frame(direction, frame)?;
    std::str::from_utf8(payload).map_err(|_error| NativeMessagingFrameError::InvalidUtf8Payload)
}
