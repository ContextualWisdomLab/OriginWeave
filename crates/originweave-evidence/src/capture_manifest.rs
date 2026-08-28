//! Deterministic identity manifests for schema-bound WARC/PROV capture evidence.
//!
//! A capture manifest binds one reviewed extraction-schema contract to exact WARC and PROV
//! serialization identities plus one immutable OriginWeave software revision. It intentionally
//! contains no captured payload, source URL, source locator, credential, browser authority,
//! persistence authority, retention decision, signature, or release authority.

use std::{collections::BTreeMap, collections::BTreeSet, fmt};

use sha2::{Digest, Sha256};

use crate::{
    ExtractionCardinality, ExtractionNormalizationRule, ExtractionSchema, ExtractionSourceChannel,
    ExtractionValueType, MAX_EXTRACTION_IDENTIFIER_BYTES, WarcProvBundle,
    WarcProvBundleVerificationError, WarcResourceRecord,
};

/// Version of the deterministic OriginWeave capture-manifest serialization contract.
pub const CAPTURE_MANIFEST_VERSION: u16 = 1;
/// Maximum number of WARC/PROV pairs admitted by one capture manifest.
pub const MAX_CAPTURE_MANIFEST_RECORDS: usize = 256;
/// Maximum number of structured-value digest bindings admitted by one capture manifest.
pub const MAX_CAPTURE_MANIFEST_VALUES: usize = 1_024;

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
    /// The number of structured values exceeded the bounded manifest limit.
    ValueLimitExceeded,
    /// A structured-value field identifier did not use the extraction-schema identifier grammar.
    InvalidValueField,
    /// A structured-value digest was not a canonical lowercase SHA-256 identifier.
    InvalidValueDigest,
    /// A structured-value field was not declared by the bound extraction schema.
    UnknownValueField,
    /// A structured value referenced a WARC record absent from the manifest.
    ValueSourceRecordMissing,
    /// A structured value used WARC evidence for a field that did not admit network-response evidence.
    ValueSourceChannelMismatch,
    /// The same field, value digest, and source WARC record were supplied more than once.
    DuplicateValue,
    /// A structured-value field exceeded its declared extraction cardinality.
    ValueCardinalityExceeded,
    /// A required extraction field had no structured-value binding.
    RequiredValueMissing,
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
            Self::ValueLimitExceeded => {
                formatter.write_str("capture manifest structured-value limit exceeded")
            }
            Self::InvalidValueField => {
                formatter.write_str("capture manifest structured-value field is invalid")
            }
            Self::InvalidValueDigest => formatter
                .write_str("capture manifest structured-value digest is not canonical SHA-256"),
            Self::UnknownValueField => formatter
                .write_str("capture manifest structured-value field is absent from the schema"),
            Self::ValueSourceRecordMissing => formatter
                .write_str("capture manifest structured value references an absent WARC record"),
            Self::ValueSourceChannelMismatch => formatter.write_str(
                "capture manifest structured value is not admitted by the field source channels",
            ),
            Self::DuplicateValue => {
                formatter.write_str("capture manifest contains a duplicate structured value")
            }
            Self::ValueCardinalityExceeded => {
                formatter.write_str("capture manifest structured value exceeds field cardinality")
            }
            Self::RequiredValueMissing => {
                formatter.write_str("capture manifest is missing a required structured value")
            }
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
            | Self::SoftwareRevisionMismatch
            | Self::ValueLimitExceeded
            | Self::InvalidValueField
            | Self::InvalidValueDigest
            | Self::UnknownValueField
            | Self::ValueSourceRecordMissing
            | Self::ValueSourceChannelMismatch
            | Self::DuplicateValue
            | Self::ValueCardinalityExceeded
            | Self::RequiredValueMissing => None,
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

/// Payload-free binding from one schema field digest to its exact source WARC record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CaptureManifestValueBinding {
    field_name: String,
    value_digest: String,
    source_warc_record_id: String,
}

impl CaptureManifestValueBinding {
    /// Validate one credential-safe structured-value identity binding.
    ///
    /// The binding carries only a schema field identifier, canonical value digest, and WARC record
    /// identifier. The referenced record and field authority are validated when the binding is
    /// admitted into a [`CaptureManifest`]. Raw extracted values are never stored here.
    pub fn new(
        field_name: &str,
        value_digest: &str,
        source_warc_record_id: &str,
    ) -> Result<Self, CaptureManifestError> {
        if !valid_value_field_name(field_name) {
            return Err(CaptureManifestError::InvalidValueField);
        }
        if !valid_sha256(value_digest) {
            return Err(CaptureManifestError::InvalidValueDigest);
        }
        Ok(Self {
            field_name: field_name.to_owned(),
            value_digest: value_digest.to_owned(),
            source_warc_record_id: source_warc_record_id.to_owned(),
        })
    }

    /// Return the extraction-schema field identifier.
    #[must_use]
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Return the canonical lowercase SHA-256 digest of the extracted value bytes.
    #[must_use]
    pub fn value_digest(&self) -> &str {
        &self.value_digest
    }

    /// Return the exact WARC record identifier that supplied this value.
    #[must_use]
    pub fn source_warc_record_id(&self) -> &str {
        &self.source_warc_record_id
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
    values: Vec<CaptureManifestValueBinding>,
}

impl CaptureManifest {
    /// Construct one bounded deterministic evidence-only capture manifest.
    ///
    /// Every supplied PROV bundle must verify the paired WARC record exactly and all pairs must
    /// name the same canonical OriginWeave software revision. Caller order is not identity:
    /// records are canonicalized by WARC record identifier before serialization and comparison.
    /// This low-level constructor does not assert that required extraction fields have values;
    /// use [`Self::new_with_warc_values`] for a schema-conforming structured-result manifest.
    /// It performs no capture, network, browser, persistence, model, signing, or authorization
    /// operation.
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
            values: Vec::new(),
        })
    }

    /// Construct a schema-conforming manifest with WARC-backed structured-value identities.
    ///
    /// Every value must name a declared schema field that admits network-response evidence and an
    /// exact WARC record present in this manifest. Required fields must be present; `One` and
    /// `ZeroOrOne` fields admit at most one value. Duplicate bindings and over-limit collections
    /// fail closed. Values are canonicalized independently of caller order. No raw extracted value
    /// is retained and no browser, network, persistence, secret, model, or authorization operation
    /// is performed.
    pub fn new_with_warc_values(
        schema: &ExtractionSchema,
        records: &[(&WarcResourceRecord, &WarcProvBundle)],
        values: &[CaptureManifestValueBinding],
    ) -> Result<Self, CaptureManifestError> {
        let mut manifest = Self::new(schema, records)?;
        if values.len() > MAX_CAPTURE_MANIFEST_VALUES {
            return Err(CaptureManifestError::ValueLimitExceeded);
        }

        let mut seen_values = BTreeSet::new();
        let mut field_counts: BTreeMap<&str, usize> = BTreeMap::new();
        for value in values {
            let Some(field) = schema.field(value.field_name()) else {
                return Err(CaptureManifestError::UnknownValueField);
            };
            if !manifest
                .records
                .iter()
                .any(|record| record.warc_record_id() == value.source_warc_record_id())
            {
                return Err(CaptureManifestError::ValueSourceRecordMissing);
            }
            if !field
                .source_channels()
                .contains(&ExtractionSourceChannel::NetworkResponse)
            {
                return Err(CaptureManifestError::ValueSourceChannelMismatch);
            }
            if !seen_values.insert((
                value.field_name(),
                value.value_digest(),
                value.source_warc_record_id(),
            )) {
                return Err(CaptureManifestError::DuplicateValue);
            }

            let count = field_counts.entry(value.field_name()).or_default();
            *count += 1;
            if *count > 1
                && matches!(
                    field.cardinality(),
                    ExtractionCardinality::One | ExtractionCardinality::ZeroOrOne
                )
            {
                return Err(CaptureManifestError::ValueCardinalityExceeded);
            }
        }

        if schema.fields().iter().any(|field| {
            field.required()
                && field_counts
                    .get(field.identifier())
                    .copied()
                    .unwrap_or_default()
                    == 0
        }) {
            return Err(CaptureManifestError::RequiredValueMissing);
        }

        manifest.values = values.to_vec();
        manifest.values.sort();
        Ok(manifest)
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

    /// Return structured-value identities in canonical field/digest/source-record order.
    #[must_use]
    pub fn values(&self) -> &[CaptureManifestValueBinding] {
        &self.values
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
        let values = self
            .values
            .iter()
            .map(|value| {
                format!(
                    "{{\"fieldName\":\"{}\",\"valueDigest\":\"{}\",\"sourceWarcRecordId\":\"{}\"}}",
                    value.field_name, value.value_digest, value.source_warc_record_id
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"manifestVersion\":{},\"schemaVersion\":\"{}\",\"schemaDigest\":\"{}\",\"softwareCommitSha\":\"{}\",\"records\":[{}],\"values\":[{}]}}",
            self.version,
            self.schema_version,
            self.schema_digest,
            self.software_commit_sha,
            records,
            values
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

    /// Reconstruct a candidate evidence-only manifest and require exact immutable identity equality.
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

    /// Reconstruct a candidate WARC-backed structured-result manifest and require exact identity.
    ///
    /// Candidate schema, WARC/PROV evidence, field admission, cardinality, requiredness, source
    /// identity, and value digests are all revalidated before immutable manifest equality is tested.
    pub fn verify_with_warc_values(
        &self,
        schema: &ExtractionSchema,
        records: &[(&WarcResourceRecord, &WarcProvBundle)],
        values: &[CaptureManifestValueBinding],
    ) -> Result<(), CaptureManifestVerificationError> {
        let candidate = Self::new_with_warc_values(schema, records, values)
            .map_err(CaptureManifestVerificationError::InvalidCandidate)?;
        if candidate == *self {
            Ok(())
        } else {
            Err(CaptureManifestVerificationError::IdentityMismatch)
        }
    }
}

fn valid_value_field_name(field_name: &str) -> bool {
    if field_name.len() > MAX_EXTRACTION_IDENTIFIER_BYTES {
        return false;
    }
    let mut bytes = field_name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && bytes.all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'))
}

fn valid_sha256(value_digest: &str) -> bool {
    let Some(hex_digest) = value_digest.strip_prefix("sha256:") else {
        return false;
    };
    hex_digest.len() == 64
        && hex_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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
