//! Bounded pre-parser ingress for one Chrome native-messaging host manifest document.
//!
//! This module deliberately stops before JSON parsing. It bounds untrusted document bytes,
//! validates UTF-8, and requires one outer object-shaped envelope using only JSON whitespace
//! before a future manifest parser can allocate from or interpret structured fields. Admission
//! here does not prove that the document is valid JSON, installed by Chrome, authenticated by
//! the operating system, or safe to use as process or Agent authority.

use std::fmt;

/// Maximum UTF-8 byte length accepted for one native-messaging host manifest document.
///
/// Chrome does not define this OriginWeave-specific 64 KiB safety budget. The limit exists to
/// bound allocation and parser input before any JSON or authority-bearing field processing.
pub const MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES: usize = 64 * 1024;

/// A bounded UTF-8 native-messaging host manifest document awaiting structured parsing.
///
/// Possessing this value proves only that the original byte document was non-empty, within the
/// OriginWeave ingress budget, valid UTF-8, and wrapped by one outer object boundary after JSON
/// whitespace is ignored. It is not proof of valid JSON and is not a validated host manifest;
/// it carries no installation, origin, executable, process, or Agent authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMessagingManifestDocument {
    text: String,
}

impl NativeMessagingManifestDocument {
    /// Admit one untrusted manifest document before JSON parsing.
    ///
    /// The byte-size check runs before UTF-8 decoding or allocation of the stored `String` so
    /// oversized input cannot force unbounded parser or text-storage work. Empty input, invalid
    /// UTF-8, and documents whose first and last non-JSON-whitespace characters are not `{` and
    /// `}` fail closed. The object-envelope check is only a cheap ingress guard; a later
    /// structured parser must still prove complete JSON syntax and field semantics.
    pub fn parse(bytes: &[u8]) -> Result<Self, NativeMessagingManifestDocumentError> {
        if bytes.is_empty() {
            return Err(NativeMessagingManifestDocumentError::EmptyDocument);
        }
        if bytes.len() > MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES {
            return Err(NativeMessagingManifestDocumentError::DocumentTooLarge);
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_error| NativeMessagingManifestDocumentError::InvalidUtf8)?;
        let trimmed = text.trim_matches(|character| matches!(character, ' ' | '\t' | '\r' | '\n'));
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return Err(NativeMessagingManifestDocumentError::InvalidObjectBoundary);
        }
        Ok(Self {
            text: text.to_owned(),
        })
    }

    /// Return the exact validated UTF-8 text without interpreting JSON fields.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// Failure to admit a native-messaging host manifest document at the pre-parser boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingManifestDocumentError {
    /// The manifest document contained zero bytes.
    EmptyDocument,
    /// The manifest document exceeded the OriginWeave pre-parser safety budget.
    DocumentTooLarge,
    /// The manifest document was not valid UTF-8.
    InvalidUtf8,
    /// The document did not have one outer object boundary after JSON whitespace was removed.
    InvalidObjectBoundary,
}

impl fmt::Display for NativeMessagingManifestDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDocument => {
                formatter.write_str("native messaging host manifest document is empty")
            }
            Self::DocumentTooLarge => formatter.write_str(
                "native messaging host manifest document exceeds the OriginWeave safety budget",
            ),
            Self::InvalidUtf8 => {
                formatter.write_str("native messaging host manifest document is not valid UTF-8")
            }
            Self::InvalidObjectBoundary => formatter.write_str(
                "native messaging host manifest document must have one outer JSON object boundary",
            ),
        }
    }
}

impl std::error::Error for NativeMessagingManifestDocumentError {}
