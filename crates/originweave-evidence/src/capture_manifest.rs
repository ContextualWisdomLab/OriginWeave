//! Deterministic identity manifests for schema-bound WARC/PROV capture evidence.
//!
//! A capture manifest binds one reviewed extraction-schema contract to exact WARC and PROV
//! serialization identities plus one immutable OriginWeave software revision. It intentionally
//! contains no captured payload, source URL, source locator, credential, browser authority,
//! persistence authority, retention decision, signature, or release authority.

use std::{collections::BTreeMap, fmt};

use sha2::{Digest, Sha256};

use crate::{
    ExtractionCardinality, ExtractionNormalizationRule, ExtractionSchema, ExtractionSourceChannel,
    ExtractionValueType, WarcProvBundle, WarcProvBundleVerificationError, WarcResourceRecord,
};

/// Version of the deterministic OriginWeave capture-manifest serialization contract.
pub const CAPTURE_MANIFEST_VERSION: u16 = 1;
/// Maximum number of WARC/PROV pairs admitted by one capture manifest.
pub const MAX_CAPTURE_MANIFEST_RECORDS: usize = 256;

/// A validation failure while constructing one deterministic capture manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureManifestError {
    /// A capture manifest must bind at least one WARC/PROV pair.
    MissingRecord,
    /// The number of supplied records exceeded the bounded manifest limit.
    LimitExceeded,
    /// More than one supplied pair used the same WARC record identifier.
    DuplicateRecord,
    /// A supplied PROV bundle did not exactly verify its paired WARC record.
    BundleMismatch(WarcProvBundleVerificationError),
    /// Supplied PROV bundles referred to different OriginWeave software revisions.
    SoftwareRevisionMismatch,
}

impl fmt::Display for CaptureManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRecord => {
                formatter.write_str("capture manifest requires at least one record")
            }
            Self::LimitExceeded => formatter.write_str("capture manifest record limit exceeded"),
            Self::DuplicateRecord => {
                formatter.write_str("capture manifest contains a duplicate WARC record")
            }
            Self::BundleMismatch(error) => {
                write!(formatter, "capture manifest WARC/PROV mismatch: {error}")
            }
            Self::SoftwareRevisionMismatch => formatter.write_str(
                "capture manifest records do not share one OriginWeave software revision",
            ),
        }
    }
}

impl std::error::Error for CaptureManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BundleMismatch(error) => Some(error),
            Self::MissingRecord
            | Self::LimitExceeded
            | Self::DuplicateRecord
            | Self::SoftwareRevisionMismatch => None,
        }
    }
}

/// A deterministic offline verification failure for a previously constructed capture manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureManifestVerificationError {
    /// A valid candidate manifest did not have the same immutable identity as the expected manifest.
    IdentityMismatch,
    /// The candidate inputs did not form a valid manifest at all.
    InvalidCandidate(CaptureManifestError),
}

impl fmt::Display for CaptureManifestVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentityMismatch => {
                formatter.write_str("capture manifest identity does not match")
            }
            Self::InvalidCandidate(error) => {
                write!(
                    formatter,
                    "invalid capture manifest verification candidate: {error}"
                )
            }
        }
    }
}

impl std::error::Error for CaptureManifestVerificationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IdentityMismatch => None,
            Self::InvalidCandidate(error) => Some(error),
        }
    }
}

/// Payload-free immutable identity for one WARC/PROV pair in a [`CaptureManifest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureManifestRecord {
    warc_record_id: String,
    warc_record_digest: String,
    prov_json_ld_digest: String,
}

impl CaptureManifestRecord {
    /// Return the UUID URN of the WARC record bound by this entry.
    #[must_use]
    pub fn warc_record_id(&self) -> &str {
        &self.warc_record_id
    }

    /// Return the SHA-256 digest of the complete deterministic WARC serialization.
    #[must_use]
    pub fn warc_record_digest(&self) -> &str {
        &self.warc_record_digest
    }

    /// Return the SHA-256 digest of the deterministic PROV JSON-LD serialization.
    #[must_use]
    pub fn prov_json_ld_digest(&self) -> &str {
        &self.prov_json_ld_digest
    }
}

/// Deterministic payload-free identity binding a schema to exact WARC/PROV capture evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureManifest {
    version: u16,
    schema_version: String,
    schema_digest: String,
    software_commit_sha: String,
    records: Vec<CaptureManifestRecord>,
}

impl CaptureManifest {
    /// Construct one bounded deterministic capture manifest.
    ///
    /// Every supplied PROV bundle must verify the paired WARC record exactly and all pairs must
    /// name the same canonical OriginWeave software revision. Caller order is not identity:
    /// records are canonicalized by WARC record identifier before serialization and comparison.
    /// This constructor performs no capture, network, browser, persistence, model, signing, or
    /// authorization operation.
    pub fn new(
        schema: &ExtractionSchema,
        records: &[(&WarcResourceRecord, &WarcProvBundle)],
    ) -> Result<Self, CaptureManifestError> {
        if records.is_empty() {
            return Err(CaptureManifestError::MissingRecord);
        }
        if records.len() > MAX_CAPTURE_MANIFEST_RECORDS {
            return Err(CaptureManifestError::LimitExceeded);
        }

        let software_commit_sha = records[0].1.software_commit_sha();
        let mut canonical_records = BTreeMap::new();
        for (record, bundle) in records {
            bundle
                .verify_record(record)
                .map_err(CaptureManifestError::BundleMismatch)?;
            if bundle.software_commit_sha() != software_commit_sha {
                return Err(CaptureManifestError::SoftwareRevisionMismatch);
            }

            let manifest_record = CaptureManifestRecord {
                warc_record_id: record.record_id().to_owned(),
                warc_record_digest: sha256_digest(&record.to_warc_bytes()),
                prov_json_ld_digest: sha256_digest(bundle.to_json_ld().as_bytes()),
            };
            if canonical_records
                .insert(record.record_id().to_owned(), manifest_record)
                .is_some()
            {
                return Err(CaptureManifestError::DuplicateRecord);
            }
        }

        Ok(Self {
            version: CAPTURE_MANIFEST_VERSION,
            schema_version: schema.version().to_owned(),
            schema_digest: extraction_schema_digest(schema),
            software_commit_sha: software_commit_sha.to_owned(),
            records: canonical_records.into_values().collect(),
        })
    }

    /// Return the capture-manifest serialization-contract version.
    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    /// Return the extraction-schema version identifier bound by this manifest.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Return a SHA-256 digest of the complete ordered extraction-schema contract.
    #[must_use]
    pub fn schema_digest(&self) -> &str {
        &self.schema_digest
    }

    /// Return the canonical lower-case OriginWeave Git SHA-1 shared by every PROV bundle.
    #[must_use]
    pub fn software_commit_sha(&self) -> &str {
        &self.software_commit_sha
    }

    /// Return manifest entries in canonical WARC-record-identifier order.
    #[must_use]
    pub fn records(&self) -> &[CaptureManifestRecord] {
        &self.records
    }

    /// Serialize the manifest deterministically without captured payloads or source locations.
    #[must_use]
    pub fn to_json(&self) -> String {
        let records = self
            .records
            .iter()
            .map(|record| {
                format!(
                    "{{\"warcRecordId\":\"{}\",\"warcRecordDigest\":\"{}\",\"provJsonLdDigest\":\"{}\"}}",
                    record.warc_record_id, record.warc_record_digest, record.prov_json_ld_digest
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"manifestVersion\":{},\"schemaVersion\":\"{}\",\"schemaDigest\":\"{}\",\"softwareCommitSha\":\"{}\",\"records\":[{}]}}",
            self.version,
            self.schema_version,
            self.schema_digest,
            self.software_commit_sha,
            records
        )
    }

    /// Return the SHA-256 identity of the deterministic manifest serialization.
    #[must_use]
    pub fn manifest_digest(&self) -> String {
        sha256_digest(self.to_json().as_bytes())
    }

    /// Require candidate bytes to be the exact deterministic serialization of this manifest.
    ///
    /// This verification deliberately does not parse or normalize JSON: whitespace, member-order,
    /// encoding, or other serialization drift is an identity mismatch even when a generic JSON
    /// parser could assign equivalent data semantics. It performs no network, browser, persistence,
    /// model, signing, or authority action and does not authenticate the producer of either value.
    pub fn verify_serialized_json(
        &self,
        candidate: &[u8],
    ) -> Result<(), CaptureManifestVerificationError> {
        let expected = self.to_json();
        if candidate == expected.as_bytes() {
            Ok(())
        } else {
            Err(CaptureManifestVerificationError::IdentityMismatch)
        }
    }

    /// Reconstruct a candidate manifest offline and require exact immutable identity equality.
    ///
    /// Malformed candidate inputs remain distinguishable from a valid-but-different manifest.
    /// Verification performs no network, browser, persistence, model, signing, or authority action.
    pub fn verify(
        &self,
        schema: &ExtractionSchema,
        records: &[(&WarcResourceRecord, &WarcProvBundle)],
    ) -> Result<(), CaptureManifestVerificationError> {
        let candidate = Self::new(schema, records)
            .map_err(CaptureManifestVerificationError::InvalidCandidate)?;
        if candidate == *self {
            Ok(())
        } else {
            Err(CaptureManifestVerificationError::IdentityMismatch)
        }
    }
}

fn extraction_schema_digest(schema: &ExtractionSchema) -> String {
    let mut hasher = Sha256::new();
    update_length_prefixed(&mut hasher, b"originweave:capture-schema:v1");
    update_length_prefixed(&mut hasher, schema.version().as_bytes());
    hasher.update((schema.fields().len() as u64).to_be_bytes());
    for field in schema.fields() {
        update_length_prefixed(&mut hasher, field.identifier().as_bytes());
        update_length_prefixed(&mut hasher, extraction_value_type_token(field.value_type()));
        update_length_prefixed(
            &mut hasher,
            extraction_cardinality_token(field.cardinality()),
        );
        hasher.update([u8::from(field.required())]);
        update_length_prefixed(
            &mut hasher,
            extraction_normalization_token(field.normalization_rule()),
        );
        hasher.update((field.source_channels().len() as u64).to_be_bytes());
        for channel in field.source_channels() {
            update_length_prefixed(&mut hasher, extraction_source_channel_token(*channel));
        }
    }
    encode_sha256(hasher.finalize())
}

fn update_length_prefixed(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

const fn extraction_value_type_token(value_type: ExtractionValueType) -> &'static [u8] {
    match value_type {
        ExtractionValueType::Text => b"text",
        ExtractionValueType::Integer => b"integer",
        ExtractionValueType::Decimal => b"decimal",
        ExtractionValueType::Boolean => b"boolean",
        ExtractionValueType::Timestamp => b"timestamp",
    }
}

const fn extraction_cardinality_token(cardinality: ExtractionCardinality) -> &'static [u8] {
    match cardinality {
        ExtractionCardinality::One => b"one",
        ExtractionCardinality::ZeroOrOne => b"zero_or_one",
        ExtractionCardinality::Many => b"many",
    }
}

const fn extraction_normalization_token(
    normalization_rule: ExtractionNormalizationRule,
) -> &'static [u8] {
    match normalization_rule {
        ExtractionNormalizationRule::Verbatim => b"verbatim",
        ExtractionNormalizationRule::TrimTextWhitespace => b"trim_text_whitespace",
        ExtractionNormalizationRule::Rfc3339Utc => b"rfc3339_utc",
    }
}

const fn extraction_source_channel_token(channel: ExtractionSourceChannel) -> &'static [u8] {
    match channel {
        ExtractionSourceChannel::SemanticNode => b"semantic_node",
        ExtractionSourceChannel::StructuredData => b"structured_data",
        ExtractionSourceChannel::TableCell => b"table_cell",
        ExtractionSourceChannel::NetworkResponse => b"network_response",
        ExtractionSourceChannel::ModelInterpretation => b"model_interpretation",
    }
}

fn sha256_digest(bytes: &[u8]) -> String {
    encode_sha256(Sha256::digest(bytes))
}

fn encode_sha256(digest: impl IntoIterator<Item = u8>) -> String {
    let mut encoded = String::from("sha256:");
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}
