//! Credential-safe evidence and provenance value objects for OriginWeave.
//!
//! Network capture is represented without bodies or metadata values. Higher
//! layers may persist explicitly approved bodies separately under bounded
//! retention policy while retaining these redacted records for audit and replay.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod action_outcome;
mod sensitive_access;

pub use action_outcome::{
    PostConditionKind, PostConditionObservation, VerifiedActionOutcomeError,
    VerifiedActionOutcomeEvidence,
};
pub use sensitive_access::{
    MAX_SENSITIVE_FIELD_COUNT, MAX_SENSITIVE_IDENTIFIER_BYTES, SensitiveAccessClass,
    SensitiveAccessEvidence, SensitiveAccessEvidenceInput, SensitiveAccessOutcome,
    SensitiveEvidenceError,
};

use std::collections::BTreeMap;

use originweave_core::{ObservedNodeHandle, Origin};

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
/// Maximum byte length of one structured extracted-field identifier.
pub const MAX_STRUCTURED_FIELD_NAME_BYTES: usize = 128;

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
    source_origin: Origin,
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
        let Some(source_origin) = parse_source_origin(source_url) else {
            return Err(EvidenceError::InvalidSourceUrl);
        };
        if source_locator.is_empty() {
            return Err(EvidenceError::EmptyLocator);
        }
        if !valid_sha256(source_hash) {
            return Err(EvidenceError::InvalidHash);
        }
        Ok(Self {
            source_url: source_url.to_owned(),
            source_origin,
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

    pub(crate) const fn source_origin(&self) -> &Origin {
        &self.source_origin
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

fn parse_source_origin(source_url: &str) -> Option<Origin> {
    if source_url.is_empty()
        || source_url
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || source_url.contains(['?', '#', '\\'])
    {
        return None;
    }
    let (scheme, remainder) = source_url.split_once("://")?;
    let authority_end = match remainder.find('/') {
        Some(index) => index,
        None => remainder.len(),
    };
    let authority = &remainder[..authority_end];
    let origin_text = format!("{scheme}://{authority}");
    let origin = Origin::parse(&origin_text).ok()?;
    let path = &remainder[authority_end..];
    if !path.is_empty() && validate_path(path).is_err() {
        return None;
    }
    Some(origin)
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

/// A credential-safe proof bundle for one extracted structured value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredValueEvidence {
    field_name: String,
    value_hash: String,
    source_node: ObservedNodeHandle,
    node_provenance: ProvenanceRecord,
    network_provenance: ProvenanceRecord,
}

impl StructuredValueEvidence {
    /// Bind one structured field digest to exact node and network provenance.
    pub fn new(
        field_name: &str,
        value_hash: &str,
        source_node: ObservedNodeHandle,
        node_provenance: ProvenanceRecord,
        network_provenance: ProvenanceRecord,
    ) -> Result<Self, StructuredValueEvidenceError> {
        if !valid_structured_field_name(field_name) {
            return Err(StructuredValueEvidenceError::InvalidFieldName);
        }
        if !valid_sha256(value_hash) {
            return Err(StructuredValueEvidenceError::InvalidValueHash);
        }
        if node_provenance.verification_result() != VerificationResult::Verified {
            return Err(StructuredValueEvidenceError::NodeProvenanceNotVerified);
        }
        if !matches!(
            node_provenance.source_kind(),
            EvidenceSourceKind::DomTree | EvidenceSourceKind::AccessibilityTree
        ) {
            return Err(StructuredValueEvidenceError::NodeProvenanceKindMismatch);
        }
        if network_provenance.verification_result() != VerificationResult::Verified {
            return Err(StructuredValueEvidenceError::NetworkProvenanceNotVerified);
        }
        if network_provenance.source_kind() != EvidenceSourceKind::NetworkResponse {
            return Err(StructuredValueEvidenceError::NetworkProvenanceKindMismatch);
        }
        if node_provenance.source_origin() != source_node.origin()
            || network_provenance.source_origin() != source_node.origin()
        {
            return Err(StructuredValueEvidenceError::SourceOriginMismatch);
        }
        Ok(Self {
            field_name: field_name.to_owned(),
            value_hash: value_hash.to_owned(),
            source_node,
            node_provenance,
            network_provenance,
        })
    }

    /// Return the bounded structured field identifier.
    #[must_use]
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Return the lowercase SHA-256 digest of the canonical extracted value bytes.
    #[must_use]
    pub fn value_hash(&self) -> &str {
        &self.value_hash
    }

    /// Return the exact OriginWeave-owned source node.
    #[must_use]
    pub const fn source_node(&self) -> &ObservedNodeHandle {
        &self.source_node
    }

    /// Return the independently verified DOM/accessibility provenance for the source node.
    #[must_use]
    pub const fn node_provenance(&self) -> &ProvenanceRecord {
        &self.node_provenance
    }

    /// Return the independently verified network provenance associated with the value.
    #[must_use]
    pub const fn network_provenance(&self) -> &ProvenanceRecord {
        &self.network_provenance
    }
}

/// A fail-closed reason why structured extraction evidence could not be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredValueEvidenceError {
    /// The structured field identifier was empty, oversized, or contained unsupported bytes.
    InvalidFieldName,
    /// The value digest was not a canonical lowercase SHA-256 identifier.
    InvalidValueHash,
    /// The node provenance did not carry independent verified status.
    NodeProvenanceNotVerified,
    /// The node provenance was not a DOM or accessibility observation.
    NodeProvenanceKindMismatch,
    /// The network provenance did not carry independent verified status.
    NetworkProvenanceNotVerified,
    /// The network provenance was not a structured network-response observation.
    NetworkProvenanceKindMismatch,
    /// Node or network provenance belonged to a different canonical origin than the source node.
    SourceOriginMismatch,
}

impl std::fmt::Display for StructuredValueEvidenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidFieldName => "invalid structured field name",
            Self::InvalidValueHash => "invalid structured value hash",
            Self::NodeProvenanceNotVerified => "node provenance is not verified",
            Self::NodeProvenanceKindMismatch => {
                "node provenance is not DOM or accessibility evidence"
            }
            Self::NetworkProvenanceNotVerified => "network provenance is not verified",
            Self::NetworkProvenanceKindMismatch => {
                "network provenance is not network-response evidence"
            }
            Self::SourceOriginMismatch => "provenance origin does not match source node origin",
        })
    }
}

impl std::error::Error for StructuredValueEvidenceError {}

fn valid_structured_field_name(field_name: &str) -> bool {
    if field_name.is_empty() || field_name.len() > MAX_STRUCTURED_FIELD_NAME_BYTES {
        return false;
    }
    field_name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        && field_name.bytes().any(|byte| byte.is_ascii_alphanumeric())
}