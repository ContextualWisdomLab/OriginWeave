#![allow(clippy::expect_used)]

use originweave_evidence::{
    CaptureManifest, CaptureManifestError, CaptureManifestValueBinding,
    CaptureManifestVerificationError, EvidenceSourceKind, ExtractionCardinality, ExtractionField,
    ExtractionSchema, ExtractionSourceChannel, ExtractionValueType, MAX_CAPTURE_MANIFEST_VALUES,
    ProvenanceRecord, VerificationResult, WarcProvBundle, WarcResourceRecord,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VALUE_HASH_A: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const VALUE_HASH_B: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const RECORD_ID_A: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const RECORD_ID_B: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174001";
const DATE: &str = "2026-08-25T00:00:00Z";
const SOFTWARE_COMMIT_SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn schema() -> ExtractionSchema {
    let title = ExtractionField::new(
        "title",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &[ExtractionSourceChannel::NetworkResponse],
    )
    .expect("title field");
    let tags = ExtractionField::new(
        "tags",
        ExtractionValueType::Text,
        ExtractionCardinality::Many,
        false,
        &[ExtractionSourceChannel::NetworkResponse],
    )
    .expect("tags field");
    ExtractionSchema::new("catalog-v2", vec![title, tags]).expect("schema")
}

fn semantic_only_schema() -> ExtractionSchema {
    let title = ExtractionField::new(
        "title",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &[ExtractionSourceChannel::SemanticNode],
    )
    .expect("semantic-only field");
    ExtractionSchema::new("semantic-v1", vec![title]).expect("schema")
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
    .expect("WARC record")
}

#[test]
fn capture_manifest_binds_required_warc_backed_structured_values() {
    let schema = schema();
    let record = resource_record(RECORD_ID_A, b"captured-payload");
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let title = CaptureManifestValueBinding::new("title", VALUE_HASH_A, RECORD_ID_A)
        .expect("value binding");

    let manifest = CaptureManifest::new_with_warc_values(
        &schema,
        &[(&record, &bundle)],
        std::slice::from_ref(&title),
    )
    .expect("manifest with value");

    assert_eq!(manifest.values(), std::slice::from_ref(&title));
    assert_eq!(manifest.values()[0].field_name(), "title");
    assert_eq!(manifest.values()[0].value_digest(), VALUE_HASH_A);
    assert_eq!(manifest.values()[0].source_warc_record_id(), RECORD_ID_A);
    assert_eq!(
        manifest.verify_with_warc_values(
            &schema,
            &[(&record, &bundle)],
            std::slice::from_ref(&title),
        ),
        Ok(())
    );
    assert_eq!(
        manifest.verify_with_warc_values(&schema, &[], std::slice::from_ref(&title)),
        Err(CaptureManifestVerificationError::InvalidCandidate(
            CaptureManifestError::MissingRecord,
        ))
    );

    let json = manifest.to_json();
    assert!(json.contains("\"values\":["));
    assert!(json.contains("\"fieldName\":\"title\""));
    assert!(json.contains(VALUE_HASH_A));
    assert!(json.contains(RECORD_ID_A));
    assert!(!json.contains("captured-payload"));
}

#[test]
fn capture_manifest_value_admission_fails_closed() {
    assert_eq!(
        CaptureManifestValueBinding::new("Title", VALUE_HASH_A, RECORD_ID_A),
        Err(CaptureManifestError::InvalidValueField)
    );
    assert_eq!(
        CaptureManifestValueBinding::new("title", "sha256:ABC", RECORD_ID_A),
        Err(CaptureManifestError::InvalidValueDigest)
    );

    let schema = schema();
    let record_a = resource_record(RECORD_ID_A, b"a");
    let record_b = resource_record(RECORD_ID_B, b"b");
    let bundle_a = WarcProvBundle::new(&record_a, SOFTWARE_COMMIT_SHA).expect("PROV A");
    let bundle_b = WarcProvBundle::new(&record_b, SOFTWARE_COMMIT_SHA).expect("PROV B");
    let title_a =
        CaptureManifestValueBinding::new("title", VALUE_HASH_A, RECORD_ID_A).expect("title A");
    let title_b =
        CaptureManifestValueBinding::new("title", VALUE_HASH_B, RECORD_ID_B).expect("title B");
    let unknown = CaptureManifestValueBinding::new("missing", VALUE_HASH_A, RECORD_ID_A)
        .expect("syntactically valid unknown field");
    let missing_record = CaptureManifestValueBinding::new("title", VALUE_HASH_A, RECORD_ID_B)
        .expect("syntactically valid missing record");

    assert_eq!(
        CaptureManifest::new_with_warc_values(&schema, &[(&record_a, &bundle_a)], &[]),
        Err(CaptureManifestError::RequiredValueMissing)
    );
    assert_eq!(
        CaptureManifest::new_with_warc_values(
            &schema,
            &[(&record_a, &bundle_a)],
            std::slice::from_ref(&unknown),
        ),
        Err(CaptureManifestError::UnknownValueField)
    );
    assert_eq!(
        CaptureManifest::new_with_warc_values(
            &schema,
            &[(&record_a, &bundle_a)],
            std::slice::from_ref(&missing_record),
        ),
        Err(CaptureManifestError::ValueSourceRecordMissing)
    );
    assert_eq!(
        CaptureManifest::new_with_warc_values(
            &semantic_only_schema(),
            &[(&record_a, &bundle_a)],
            std::slice::from_ref(&title_a),
        ),
        Err(CaptureManifestError::ValueSourceChannelMismatch)
    );
    assert_eq!(
        CaptureManifest::new_with_warc_values(
            &schema,
            &[(&record_a, &bundle_a), (&record_b, &bundle_b)],
            &[title_a.clone(), title_b],
        ),
        Err(CaptureManifestError::ValueCardinalityExceeded)
    );
    assert_eq!(
        CaptureManifest::new_with_warc_values(
            &schema,
            &[(&record_a, &bundle_a)],
            &[title_a.clone(), title_a.clone()],
        ),
        Err(CaptureManifestError::DuplicateValue)
    );

    let oversized = vec![title_a; MAX_CAPTURE_MANIFEST_VALUES + 1];
    assert_eq!(
        CaptureManifest::new_with_warc_values(&schema, &[(&record_a, &bundle_a)], &oversized),
        Err(CaptureManifestError::ValueLimitExceeded)
    );
}

#[test]
fn capture_manifest_value_order_is_canonical_and_verification_detects_drift() {
    let schema = schema();
    let record_a = resource_record(RECORD_ID_A, b"a");
    let record_b = resource_record(RECORD_ID_B, b"b");
    let bundle_a = WarcProvBundle::new(&record_a, SOFTWARE_COMMIT_SHA).expect("PROV A");
    let bundle_b = WarcProvBundle::new(&record_b, SOFTWARE_COMMIT_SHA).expect("PROV B");
    let title =
        CaptureManifestValueBinding::new("title", VALUE_HASH_A, RECORD_ID_A).expect("title");
    let tag_a = CaptureManifestValueBinding::new("tags", VALUE_HASH_A, RECORD_ID_A).expect("tag A");
    let tag_b = CaptureManifestValueBinding::new("tags", VALUE_HASH_B, RECORD_ID_B).expect("tag B");

    let first = CaptureManifest::new_with_warc_values(
        &schema,
        &[(&record_b, &bundle_b), (&record_a, &bundle_a)],
        &[tag_b.clone(), title.clone(), tag_a.clone()],
    )
    .expect("first manifest");
    let second = CaptureManifest::new_with_warc_values(
        &schema,
        &[(&record_a, &bundle_a), (&record_b, &bundle_b)],
        &[tag_a, tag_b.clone(), title.clone()],
    )
    .expect("second manifest");

    assert_eq!(first, second);
    assert_eq!(first.values()[0].field_name(), "tags");
    assert_eq!(first.values()[1].field_name(), "tags");
    assert_eq!(first.values()[2].field_name(), "title");

    let drifted_tag =
        CaptureManifestValueBinding::new("tags", VALUE_HASH_A, RECORD_ID_B).expect("drifted tag");
    assert_eq!(
        first.verify_with_warc_values(
            &schema,
            &[(&record_a, &bundle_a), (&record_b, &bundle_b)],
            &[drifted_tag, tag_b, title],
        ),
        Err(CaptureManifestVerificationError::IdentityMismatch)
    );
}
