use std::fmt;

use sha2::{Digest, Sha256};

use crate::{WarcPayloadCompleteness, WarcResourceRecord, WarcTruncationReason};

const ORIGINWEAVE_COMMIT_URL_PREFIX: &str =
    "https://github.com/ContextualWisdomLab/OriginWeave/commit/";
const WARC_RECORD_DIGEST_IRI: &str =
    "tag:contextualwisdomlab.github.io,2026:OriginWeave/warcRecordDigest";
const WARC_PAYLOAD_COMPLETENESS_IRI: &str =
    "tag:contextualwisdomlab.github.io,2026:OriginWeave/warcPayloadCompleteness";
const WARC_TRUNCATION_REASON_IRI: &str =
    "tag:contextualwisdomlab.github.io,2026:OriginWeave/warcTruncationReason";

/// Exact byte length accepted for a canonical Git SHA-1 software revision.
pub const MAX_PROV_SOFTWARE_COMMIT_SHA_BYTES: usize = 40;

/// A validation failure while constructing a deterministic WARC provenance bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarcProvBundleError {
    /// The software revision was not one canonical lower-case 40-byte Git SHA-1.
    InvalidSoftwareCommitSha,
    /// A bounded provenance field exceeded its allowed size.
    LimitExceeded,
}

impl fmt::Display for WarcProvBundleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSoftwareCommitSha => "invalid OriginWeave software commit SHA",
            Self::LimitExceeded => "WARC PROV bundle limit exceeded",
        })
    }
}

impl std::error::Error for WarcProvBundleError {}

/// A deterministic PROV-O JSON-LD projection over one validated WARC resource record.
///
/// The bundle contains identifiers, source and record hashes, source location, capture time,
/// explicit WARC payload completeness, and the exact OriginWeave software revision. It
/// deliberately does not retain or emit the WARC payload.
#[derive(Clone, PartialEq, Eq)]
pub struct WarcProvBundle {
    record_entity_id: String,
    source_entity_id: String,
    capture_activity_id: String,
    software_agent_id: String,
    software_commit_sha: String,
    source_url: String,
    source_hash: String,
    warc_date: String,
    block_digest: String,
    warc_record_digest: String,
    payload_completeness: WarcPayloadCompleteness,
}

impl fmt::Debug for WarcProvBundle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WarcProvBundle")
            .field("record_entity_id", &self.record_entity_id)
            .field("source_entity_id", &self.source_entity_id)
            .field("capture_activity_id", &self.capture_activity_id)
            .field("software_agent_id", &self.software_agent_id)
            .field("payload_completeness", &self.payload_completeness)
            .finish_non_exhaustive()
    }
}

impl WarcProvBundle {
    /// Construct a provenance bundle from one already-validated WARC resource record.
    ///
    /// `software_commit_sha` is an immutable canonical Git SHA-1 identifier. This constructor
    /// does not contact GitHub and does not treat the identifier as authentication or authority.
    pub fn new(
        record: &WarcResourceRecord,
        software_commit_sha: &str,
    ) -> Result<Self, WarcProvBundleError> {
        if software_commit_sha.len() > MAX_PROV_SOFTWARE_COMMIT_SHA_BYTES {
            return Err(WarcProvBundleError::LimitExceeded);
        }
        if !valid_software_commit_sha(software_commit_sha) {
            return Err(WarcProvBundleError::InvalidSoftwareCommitSha);
        }

        let record_entity_id = record.record_id().to_owned();
        let source_entity_id = format!("{}#source", record.record_id());
        let capture_activity_id = format!("{}#capture", record.record_id());
        let software_agent_id = format!("{ORIGINWEAVE_COMMIT_URL_PREFIX}{software_commit_sha}");
        let warc_record_digest = sha256_digest(&record.to_warc_bytes());

        Ok(Self {
            record_entity_id,
            source_entity_id,
            capture_activity_id,
            software_agent_id,
            software_commit_sha: software_commit_sha.to_owned(),
            source_url: record.provenance().source_url().to_owned(),
            source_hash: record.provenance().source_hash().to_owned(),
            warc_date: record.warc_date().to_owned(),
            block_digest: record.block_digest().to_owned(),
            warc_record_digest,
            payload_completeness: record.completeness(),
        })
    }

    /// Return the PROV entity identifier for the WARC record.
    #[must_use]
    pub fn record_entity_id(&self) -> &str {
        &self.record_entity_id
    }

    /// Return the PROV entity identifier for the independently verified source.
    #[must_use]
    pub fn source_entity_id(&self) -> &str {
        &self.source_entity_id
    }

    /// Return the PROV activity identifier for this capture.
    #[must_use]
    pub fn capture_activity_id(&self) -> &str {
        &self.capture_activity_id
    }

    /// Return the immutable OriginWeave commit URL used as the PROV software-agent identifier.
    #[must_use]
    pub fn software_agent_id(&self) -> &str {
        &self.software_agent_id
    }

    /// Return the canonical lower-case Git SHA-1 of the OriginWeave revision.
    #[must_use]
    pub fn software_commit_sha(&self) -> &str {
        &self.software_commit_sha
    }

    /// Serialize the bundle as deterministic compact W3C PROV-O JSON-LD.
    ///
    /// All interpolated values originate from the validated WARC record or the canonical
    /// lower-case software commit identifier, so no raw payload bytes enter this document. The
    /// payload block digest binds the retained resource bytes while `warcRecordDigest` binds the
    /// complete deterministic WARC serialization, including its headers. WARC payload completeness
    /// is retained as an OriginWeave-owned absolute-IRI attribute; truncated records also retain
    /// the exact WARC truncation token.
    #[must_use]
    pub fn to_json_ld(&self) -> String {
        let completeness_attributes =
            warc_payload_completeness_attributes(self.payload_completeness);
        format!(
            "{{\"@context\":{{\"prov\":\"http://www.w3.org/ns/prov#\",\"xsd\":\"http://www.w3.org/2001/XMLSchema#\"}},\"@graph\":[{{\"@id\":\"{}\",\"@type\":\"prov:Entity\",\"prov:atLocation\":{{\"@id\":\"{}\"}},\"prov:value\":\"{}\"}},{{\"@id\":\"{}\",\"@type\":\"prov:Activity\",\"prov:startedAtTime\":{{\"@value\":\"{}\",\"@type\":\"xsd:dateTime\"}},\"prov:used\":{{\"@id\":\"{}\"}},\"prov:wasAssociatedWith\":{{\"@id\":\"{}\"}}}},{{\"@id\":\"{}\",\"@type\":\"prov:SoftwareAgent\"}},{{\"@id\":\"{}\",\"@type\":\"prov:Entity\",\"prov:value\":\"{}\",\"{WARC_RECORD_DIGEST_IRI}\":\"{}\",{},\"prov:wasDerivedFrom\":{{\"@id\":\"{}\"}},\"prov:wasGeneratedBy\":{{\"@id\":\"{}\"}}}}]}}",
            self.source_entity_id,
            self.source_url,
            self.source_hash,
            self.capture_activity_id,
            self.warc_date,
            self.source_entity_id,
            self.software_agent_id,
            self.software_agent_id,
            self.record_entity_id,
            self.block_digest,
            self.warc_record_digest,
            completeness_attributes,
            self.source_entity_id,
            self.capture_activity_id,
        )
    }
}

fn warc_payload_completeness_attributes(completeness: WarcPayloadCompleteness) -> String {
    match completeness {
        WarcPayloadCompleteness::Complete => {
            format!("\"{WARC_PAYLOAD_COMPLETENESS_IRI}\":\"complete\"")
        }
        WarcPayloadCompleteness::Truncated(reason) => format!(
            "\"{WARC_PAYLOAD_COMPLETENESS_IRI}\":\"truncated\",\"{WARC_TRUNCATION_REASON_IRI}\":\"{}\"",
            warc_truncation_reason_token(reason)
        ),
    }
}

const fn warc_truncation_reason_token(reason: WarcTruncationReason) -> &'static str {
    match reason {
        WarcTruncationReason::Length => "length",
        WarcTruncationReason::Time => "time",
        WarcTruncationReason::Disconnect => "disconnect",
        WarcTruncationReason::Unspecified => "unspecified",
    }
}

fn sha256_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::from("sha256:");
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn valid_software_commit_sha(software_commit_sha: &str) -> bool {
    software_commit_sha.len() == MAX_PROV_SOFTWARE_COMMIT_SHA_BYTES
        && software_commit_sha
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
