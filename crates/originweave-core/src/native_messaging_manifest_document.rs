//! Bounded pre-parser ingress for one Chrome native-messaging host manifest document.
//!
//! This module deliberately stops before JSON parsing. It bounds untrusted document bytes and
//! validates UTF-8 before a future manifest parser can allocate from or interpret structured
//! fields. Admission here does not prove that the document is valid JSON, installed by Chrome,
//! authenticated by the operating system, or safe to use as process or Agent authority.

use std::fmt;

/// Maximum UTF-8 byte length accepted for one native-messaging host manifest document.
///
/// Chrome does not define this OriginWeave-specific 64 KiB safety budget. The limit exists to
/// bound allocation and parser input before any JSON or authority-bearing field processing.
pub const MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES: usize = 64 * 1024;

/// A bounded UTF-8 native-messaging host manifest document awaiting structured parsing.
///
/// Possessing this value proves only that the original byte document was non-empty, within the
/// OriginWeave ingress budget, and valid UTF-8. It is not a validated host manifest and carries
/// no installation, origin, executable, process, or Agent authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMessagingManifestDocument {
    text: String,
}

impl NativeMessagingManifestDocument {
    /// Admit one untrusted manifest document before JSON parsing.
    ///
    /// The byte-size check runs before UTF-8 decoding or allocation of the stored `String` so
    /// oversized input cannot force unbounded parser or text-storage work. Empty input and
    /// invalid UTF-8 fail closed.
    pub fn parse(bytes: &[u8]) -> Result<Self, NativeMessagingManifestDocumentError> {
        if bytes.is_empty() {
            return Err(NativeMessagingManifestDocumentError::EmptyDocument);
        }
        if bytes.len() > MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES {
            return Err(NativeMessagingManifestDocumentError::DocumentTooLarge);
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_error| NativeMessagingManifestDocumentError::InvalidUtf8)?;
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
        }
    }
}

impl std::error::Error for NativeMessagingManifestDocumentError {}
