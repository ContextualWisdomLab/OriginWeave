//! Credential-safe evidence and provenance value objects for OriginWeave.
//!
//! Network capture is represented without bodies or metadata values. Higher
//! layers may persist explicitly approved bodies separately under bounded
//! retention policy while retaining these redacted records for audit and replay.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod extraction_schema;
mod sensitive_access;

pub use extraction_schema::{
    ExtractionCardinality, ExtractionField, ExtractionSchema, ExtractionSchemaError,
    ExtractionSourceChannel, ExtractionValueType, MAX_EXTRACTION_FIELD_COUNT,
    MAX_EXTRACTION_IDENTIFIER_BYTES,
};
pub use sensitive_access::{
    MAX_SENSITIVE_FIELD_COUNT, MAX_SENSITIVE_IDENTIFIER_BYTES, SensitiveAccessClass,
    SensitiveAccessEvidence, SensitiveAccessEvidenceInput, SensitiveAccessOutcome,
    SensitiveEvidenceError,
};

use std::collections::BTreeMap;

use originweave_core::Origin;

const REDACTED: &str = "[REDACTED]";

/// Maximum encoded request-path size retained in one network evidence record.
pub const MAX_PATH_BYTES: usize = 4_096;
/// Maximum number of header fields accepted in one network evidence record.
pub const MAX_HEADER_COUNT: usize = 128;
/// Maximum number of query fields accepted in one network evidence record.
pub const MAX_QUERY_FIELD_COUNT: usize = 128;
/// Maximum encoded field-name size accepted for headers or query parameters.
pub const MAX_METADATA_NAME_BYTES: usize = 256;
/// Maximum field-value size inspected before the value is discarded.
pub const MAX_METADATA_VALUE_BYTES: usize = 8_192;
/// Maximum source URL or source-locator size retained in provenance metadata.
pub const MAX_PROVENANCE_TEXT_BYTES: usize = 8_192;

/// An HTTP method recorded for network evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HttpMethod {
    /// Retrieve a representation.
    Get,
    /// Submit a new representation.
    Post,
    /// Replace a representation.
    Put,
    /// Partially update a representation.
    Patch,
    /// Delete a representation.
    Delete,
    /// Retrieve headers without a response body.
    Head,
    /// Discover supported request semantics.
    Options,
}

/// An evidence channel that supplied a provenance assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvidenceSourceKind {
    /// A structured browser network response.
    NetworkResponse,
    /// A node or attribute in the document tree.
    DomTree,
    /// A computed node in the accessibility tree.
    AccessibilityTree,
    /// A screenshot or other visual capture.
    VisualCapture,
    /// Embedded metadata such as JSON-LD, RDFa, or Microdata.
    StructuredData,
}

/// The deterministic verification state of one provenance assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerificationResult {
    /// Independent validation confirmed the assertion.
    Verified,
    /// The assertion has not yet completed independent validation.
    Unverified,
    /// Independent validation rejected the assertion.
    Rejected,
}

/// A validation error in an evidence object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceError {
    /// A network path was empty, ambiguous, or contained unsafe delimiters.
    InvalidPath,
    /// A bounded collection, path, name, value, URL, or locator exceeded its limit.
    LimitExceeded,
    /// A source locator was empty.
    EmptyLocator,
    /// A source digest was not a lowercase SHA-256 identifier.
    InvalidHash,
    /// A source URL was missing or contained unsafe characters.
    InvalidSourceUrl,
}

/// A redacted network observation suitable for audit logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkEvidence {
    method: HttpMethod,
    origin: Origin,
    path: String,
    headers: BTreeMap<String, String>,
    query: BTreeMap<String, String>,
}

impl NetworkEvidence {
    /// Capture one bounded network request while discarding every metadata value.
    pub fn capture(
        method: HttpMethod,
        origin: Origin,
        path: &str,
        headers: BTreeMap<String, String>,
        query: BTreeMap<String, String>,
    ) -> Result<Self, EvidenceError> {
        validate_path(path)?;
        validate_metadata(&headers, MAX_HEADER_COUNT)?;
        validate_metadata(&query, MAX_QUERY_FIELD_COUNT)?;
        Ok(Self {
            method,
            origin,
            path: path.to_owned(),
            headers: redact_all_values(headers),
            query: redact_all_values(query),
        })
    }

    /// Return the captured HTTP method.
    #[must_use]
    pub const fn method(&self) -> HttpMethod {
        self.method
    }

    /// Return the normalized request origin.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the request path without query or fragment data.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Return bounded header names with every value redacted.
    #[must_use]
    pub const fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }

    /// Return bounded query field names with every value redacted.
    #[must_use]
    pub const fn query(&self) -> &BTreeMap<String, String> {
        &self.query
    }
}

fn validate_metadata(
    values: &BTreeMap<String, String>,
    maximum_count: usize,
) -> Result<(), EvidenceError> {
    if values.len() > maximum_count
        || values.iter().any(|(name, value)| {
            name.is_empty()
                || name.len() > MAX_METADATA_NAME_BYTES
                || value.len() > MAX_METADATA_VALUE_BYTES
                || name
                    .chars()
                    .any(|character| character.is_control() || character.is_whitespace())
        })
    {
        return Err(EvidenceError::LimitExceeded);
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<(), EvidenceError> {
    if path.len() > MAX_PATH_BYTES {
        return Err(EvidenceError::LimitExceeded);
    }
    if path.is_empty()
        || !path.starts_with('/')
        || path.chars().any(|character| {
            character.is_control()
                || character.is_whitespace()
                || matches!(character, '?' | '#' | '\\')
        })
    {
        return Err(EvidenceError::InvalidPath);
    }

    let bytes = path.as_bytes();
    let mut segment = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'/' {
            if segment == b"." || segment == b".." {
                return Err(EvidenceError::InvalidPath);
            }
            segment.clear();
            index += 1;
            continue;
        }
        if byte == b'%' {
            if index + 2 >= bytes.len() {
                return Err(EvidenceError::InvalidPath);
            }
            let Some(high) = hexadecimal_value(bytes[index + 1]) else {
                return Err(EvidenceError::InvalidPath);
            };
            let Some(low) = hexadecimal_value(bytes[index + 2]) else {
                return Err(EvidenceError::InvalidPath);
            };
            let decoded = high * 16 + low;
            if decoded.is_ascii_control()
                || decoded.is_ascii_whitespace()
                || matches!(decoded, b'/' | b'\\' | b'?' | b'#')
            {
                return Err(EvidenceError::InvalidPath);
            }
            segment.push(decoded);
            index += 3;
            continue;
        }
        segment.push(byte);
        index += 1;
    }
    if segment == b"." || segment == b".." {
        return Err(EvidenceError::InvalidPath);
    }
    Ok(())
}

const fn hexadecimal_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn redact_all_values(values: BTreeMap<String, String>) -> BTreeMap<String, String> {
    values
        .into_keys()
        .map(|name| (name, REDACTED.to_owned()))
        .collect()
}

/// A provenance pointer from an extracted assertion to its exact evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceRecord {
    source_url: String,
    source_locator: String,
    source_hash: String,
    source_kind: EvidenceSourceKind,
    verification_result: VerificationResult,
}

impl ProvenanceRecord {
    /// Validate and create one provenance record.
    pub fn new(
        source_url: &str,
        source_locator: &str,
        source_hash: &str,
        source_kind: EvidenceSourceKind,
        verification_result: VerificationResult,
    ) -> Result<Self, EvidenceError> {
        if source_url.len() > MAX_PROVENANCE_TEXT_BYTES
            || source_locator.len() > MAX_PROVENANCE_TEXT_BYTES
        {
            return Err(EvidenceError::LimitExceeded);
        }
        if !valid_source_url(source_url) {
            return Err(EvidenceError::InvalidSourceUrl);
        }
        if source_locator.is_empty() {
            return Err(EvidenceError::EmptyLocator);
        }
        if !valid_sha256(source_hash) {
            return Err(EvidenceError::InvalidHash);
        }
        Ok(Self {
            source_url: source_url.to_owned(),
            source_locator: source_locator.to_owned(),
            source_hash: source_hash.to_owned(),
            source_kind,
            verification_result,
        })
    }

    /// Return the source URL captured at observation time.
    #[must_use]
    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    /// Return the channel-specific evidence locator.
    #[must_use]
    pub fn source_locator(&self) -> &str {
        &self.source_locator
    }

    /// Return the lowercase `sha256:` digest of the source artifact.
    #[must_use]
    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    /// Return the evidence channel that supplied the assertion.
    #[must_use]
    pub const fn source_kind(&self) -> EvidenceSourceKind {
        self.source_kind
    }

    /// Return the independent verification result.
    #[must_use]
    pub const fn verification_result(&self) -> VerificationResult {
        self.verification_result
    }
}

fn valid_source_url(source_url: &str) -> bool {
    if source_url.is_empty()
        || source_url
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || source_url.contains(['?', '#', '\\'])
    {
        return false;
    }
    let Some((scheme, remainder)) = source_url.split_once("://") else {
        return false;
    };
    let authority_end = remainder.find('/').unwrap_or(remainder.len());
    let authority = &remainder[..authority_end];
    let origin_text = format!("{scheme}://{authority}");
    if Origin::parse(&origin_text).is_err() {
        return false;
    }
    let path = &remainder[authority_end..];
    path.is_empty() || validate_path(path).is_ok()
}

fn valid_sha256(source_hash: &str) -> bool {
    let Some(hex_digest) = source_hash.strip_prefix("sha256:") else {
        return false;
    };
    hex_digest.len() == 64
        && hex_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
