#![allow(clippy::expect_used)]

use originweave_evidence::{
    CAPTURE_MANIFEST_VERSION, CaptureManifest, CaptureManifestError,
    CaptureManifestVerificationError, EvidenceSourceKind, ExtractionCardinality, ExtractionField,
    ExtractionNormalizationRule, ExtractionSchema, ExtractionSourceChannel, ExtractionValueType,
    MAX_CAPTURE_MANIFEST_RECORDS, ProvenanceRecord, VerificationResult, WarcProvBundle,
    WarcProvBundleVerificationError, WarcResourceRecord,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const RECORD_ID_A: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const RECORD_ID_B: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174001";
const DATE: &str = "2026-08-24T00:00:00Z";
const SOFTWARE_COMMIT_SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER_SOFTWARE_COMMIT_SHA: &str = "1123456789abcdef0123456789abcdef01234567";

fn schema(version: &str) -> ExtractionSchema {
    let field = ExtractionField::new(
        "title",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &[ExtractionSourceChannel::NetworkResponse],
    )
    .expect("field contract");
    ExtractionSchema::new(version, vec![field]).expect("schema contract")
}

fn schema_covering_all_semantics(version: &str) -> ExtractionSchema {
    let text = ExtractionField::new_with_normalization(
        "title",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        ExtractionNormalizationRule::TrimTextWhitespace,
        &[
            ExtractionSourceChannel::SemanticNode,
            ExtractionSourceChannel::StructuredData,
            ExtractionSourceChannel::TableCell,
            ExtractionSourceChannel::NetworkResponse,
            ExtractionSourceChannel::ModelInterpretation,
        ],
    )
    .expect("text field");
    let integer = ExtractionField::new(
        "quantity",
        ExtractionValueType::Integer,
        ExtractionCardinality::ZeroOrOne,
        false,
        &[ExtractionSourceChannel::NetworkResponse],
    )
    .expect("integer field");
    let decimal = ExtractionField::new(
        "price",
        ExtractionValueType::Decimal,
        ExtractionCardinality::Many,
        true,
        &[ExtractionSourceChannel::StructuredData],
    )
    .expect("decimal field");
    let boolean = ExtractionField::new(
        "available",
        ExtractionValueType::Boolean,
        ExtractionCardinality::Many,
        false,
        &[ExtractionSourceChannel::TableCell],
    )
    .expect("boolean field");
    let timestamp = ExtractionField::new_with_normalization(
        "captured_at",
        ExtractionValueType::Timestamp,
        ExtractionCardinality::One,
        true,
        ExtractionNormalizationRule::Rfc3339Utc,
        &[ExtractionSourceChannel::SemanticNode],
    )
    .expect("timestamp field");
    ExtractionSchema::new(version, vec![text, integer, decimal, boolean, timestamp])
        .expect("complete semantic schema")
}

fn resource_record(record_id: &str, payload: &[u8]) -> WarcResourceRecord {
    let provenance = ProvenanceRecord::new(
        "https://example.com/item",
        "body",
        SOURCE_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("verified provenance");
    WarcResourceRecord::new(
        record_id,
        DATE,
        "https://example.com/item",
        "text/plain",
        payload.to_vec(),
        provenance,
    )
    .expect("WARC resource record")
}

fn assert_standard_error_contract<E: std::error::Error + Send + Sync + 'static>() {}

#[test]
fn capture_manifest_binds_schema_warc_prov_and_software_identity_without_payload() {
    assert_standard_error_contract::<CaptureManifestError>();
    assert_standard_error_contract::<CaptureManifestVerificationError>();

    let schema = schema("catalog-v1");
    let record = resource_record(RECORD_ID_A, b"secret-like-captured-payload");
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let manifest = CaptureManifest::new(&schema, &[(&record, &bundle)]).expect("capture manifest");

    assert_eq!(manifest.version(), CAPTURE_MANIFEST_VERSION);
    assert_eq!(manifest.schema_version(), "catalog-v1");
    assert!(manifest.schema_digest().starts_with("sha256:"));
    assert_eq!(manifest.software_commit_sha(), SOFTWARE_COMMIT_SHA);
    assert_eq!(manifest.records().len(), 1);
    assert_eq!(manifest.records()[0].warc_record_id(), RECORD_ID_A);
    assert!(
        manifest.records()[0]
            .warc_record_digest()
            .starts_with("sha256:")
    );
    assert!(
        manifest.records()[0]
            .prov_json_ld_digest()
            .starts_with("sha256:")
    );
    assert!(manifest.manifest_digest().starts_with("sha256:"));
    assert_eq!(manifest.verify(&schema, &[(&record, &bundle)]), Ok(()));

    let json = manifest.to_json();
    assert!(json.contains("\"manifestVersion\":1"));
    assert!(json.contains("\"schemaVersion\":\"catalog-v1\""));
    assert!(json.contains(RECORD_ID_A));
    assert!(!json.contains("secret-like-captured-payload"));
    assert!(!json.contains("https://example.com/item"));
    assert!(!json.contains(SOURCE_HASH));
}

#[test]
fn capture_manifest_schema_digest_binds_every_typed_schema_dimension() {
    let record = resource_record(RECORD_ID_A, b"schema-sensitive");
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let complete_schema = schema_covering_all_semantics("catalog-semantic-v1");
    let minimal_schema = schema("catalog-semantic-v1");

    let complete = CaptureManifest::new(&complete_schema, &[(&record, &bundle)])
        .expect("complete manifest");
    let minimal =
        CaptureManifest::new(&minimal_schema, &[(&record, &bundle)]).expect("minimal manifest");

    assert_ne!(complete.schema_digest(), minimal.schema_digest());
    assert_ne!(complete, minimal);
}

#[test]
fn capture_manifest_error_contracts_preserve_typed_causes() {
    let missing = CaptureManifestError::MissingRecord;
    assert_eq!(
        missing.to_string(),
        "capture manifest requires at least one record"
    );
    assert!(std::error::Error::source(&missing).is_none());

    let limit = CaptureManifestError::LimitExceeded;
    assert_eq!(limit.to_string(), "capture manifest record limit exceeded");
    assert!(std::error::Error::source(&limit).is_none());

    let duplicate = CaptureManifestError::DuplicateRecord;
    assert_eq!(
        duplicate.to_string(),
        "capture manifest contains a duplicate WARC record"
    );
    assert!(std::error::Error::source(&duplicate).is_none());

    let software = CaptureManifestError::SoftwareRevisionMismatch;
    assert_eq!(
        software.to_string(),
        "capture manifest records do not share one OriginWeave software revision"
    );
    assert!(std::error::Error::source(&software).is_none());

    let mismatch = CaptureManifestError::BundleMismatch(
        WarcProvBundleVerificationError::RecordIdentityMismatch,
    );
    assert!(mismatch.to_string().contains("WARC/PROV mismatch"));
    assert!(std::error::Error::source(&mismatch).is_some());

    let identity = CaptureManifestVerificationError::IdentityMismatch;
    assert_eq!(identity.to_string(), "capture manifest identity does not match");
    assert!(std::error::Error::source(&identity).is_none());

    let invalid = CaptureManifestVerificationError::InvalidCandidate(mismatch);
    assert!(
        invalid
            .to_string()
            .starts_with("invalid capture manifest verification candidate:")
    );
    assert!(std::error::Error::source(&invalid).is_some());
}

#[test]
fn capture_manifest_is_order_independent_and_rejects_empty_duplicate_or_oversized_sets() {
    let schema = schema("catalog-v1");
    let record_a = resource_record(RECORD_ID_A, b"a");
    let record_b = resource_record(RECORD_ID_B, b"b");
    let bundle_a = WarcProvBundle::new(&record_a, SOFTWARE_COMMIT_SHA).expect("PROV A");
    let bundle_b = WarcProvBundle::new(&record_b, SOFTWARE_COMMIT_SHA).expect("PROV B");

    let first = CaptureManifest::new(&schema, &[(&record_b, &bundle_b), (&record_a, &bundle_a)])
        .expect("first manifest");
    let second = CaptureManifest::new(&schema, &[(&record_a, &bundle_a), (&record_b, &bundle_b)])
        .expect("second manifest");
    assert_eq!(first, second);
    assert_eq!(first.records()[0].warc_record_id(), RECORD_ID_A);
    assert_eq!(first.records()[1].warc_record_id(), RECORD_ID_B);

    assert_eq!(
        CaptureManifest::new(&schema, &[]),
        Err(CaptureManifestError::MissingRecord)
    );
    assert_eq!(
        CaptureManifest::new(&schema, &[(&record_a, &bundle_a), (&record_a, &bundle_a)],),
        Err(CaptureManifestError::DuplicateRecord)
    );

    let oversized = vec![(&record_a, &bundle_a); MAX_CAPTURE_MANIFEST_RECORDS + 1];
    assert_eq!(
        CaptureManifest::new(&schema, &oversized),
        Err(CaptureManifestError::LimitExceeded)
    );
}

#[test]
fn capture_manifest_rejects_mismatched_bundle_or_mixed_software_revision() {
    let schema = schema("catalog-v1");
    let record_a = resource_record(RECORD_ID_A, b"a");
    let record_b = resource_record(RECORD_ID_B, b"b");
    let bundle_a = WarcProvBundle::new(&record_a, SOFTWARE_COMMIT_SHA).expect("PROV A");
    let bundle_b_other = WarcProvBundle::new(&record_b, OTHER_SOFTWARE_COMMIT_SHA)
        .expect("PROV B with other software");

    assert_eq!(
        CaptureManifest::new(&schema, &[(&record_b, &bundle_a)]),
        Err(CaptureManifestError::BundleMismatch(
            WarcProvBundleVerificationError::RecordIdentityMismatch,
        ))
    );
    assert_eq!(
        CaptureManifest::new(
            &schema,
            &[(&record_a, &bundle_a), (&record_b, &bundle_b_other)],
        ),
        Err(CaptureManifestError::SoftwareRevisionMismatch)
    );
}

#[test]
fn capture_manifest_verification_fails_closed_on_schema_or_record_drift() {
    let original_schema = schema("catalog-v1");
    let record = resource_record(RECORD_ID_A, b"original");
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let manifest =
        CaptureManifest::new(&original_schema, &[(&record, &bundle)]).expect("capture manifest");

    assert_eq!(
        manifest.verify(&schema("catalog-v2"), &[(&record, &bundle)]),
        Err(CaptureManifestVerificationError::IdentityMismatch)
    );

    let changed_record = resource_record(RECORD_ID_A, b"changed");
    let changed_bundle =
        WarcProvBundle::new(&changed_record, SOFTWARE_COMMIT_SHA).expect("changed PROV bundle");
    assert_eq!(
        manifest.verify(&original_schema, &[(&changed_record, &changed_bundle)]),
        Err(CaptureManifestVerificationError::IdentityMismatch)
    );

    let other_record = resource_record(RECORD_ID_B, b"other");
    assert_eq!(
        manifest.verify(&original_schema, &[(&other_record, &bundle)]),
        Err(CaptureManifestVerificationError::InvalidCandidate(
            CaptureManifestError::BundleMismatch(
                WarcProvBundleVerificationError::RecordIdentityMismatch,
            ),
        ))
    );
}
