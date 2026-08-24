#![allow(clippy::expect_used)]

use originweave_evidence::{
    CaptureManifest, CaptureManifestVerificationError, EvidenceSourceKind, ExtractionCardinality,
    ExtractionField, ExtractionSchema, ExtractionSourceChannel, ExtractionValueType,
    ProvenanceRecord, VerificationResult, WarcProvBundle, WarcResourceRecord,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const SOFTWARE_COMMIT_SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn manifest() -> CaptureManifest {
    let field = ExtractionField::new(
        "title",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &[ExtractionSourceChannel::NetworkResponse],
    )
    .expect("field contract");
    let schema = ExtractionSchema::new("catalog-v1", vec![field]).expect("schema contract");
    let provenance = ProvenanceRecord::new(
        "https://example.com/item",
        "body",
        SOURCE_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("verified provenance");
    let record = WarcResourceRecord::new(
        RECORD_ID,
        "2026-08-24T00:00:00Z",
        "https://example.com/item",
        "text/plain",
        b"captured-payload".to_vec(),
        provenance,
    )
    .expect("WARC resource record");
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    CaptureManifest::new(&schema, &[(&record, &bundle)]).expect("capture manifest")
}

#[test]
fn persisted_capture_manifest_requires_exact_deterministic_serialization() {
    let manifest = manifest();
    let exact = manifest.to_json().into_bytes();

    assert_eq!(manifest.verify_serialized_json(&exact), Ok(()));

    let mut drifted = exact.clone();
    drifted.push(b'\n');
    assert_eq!(
        manifest.verify_serialized_json(&drifted),
        Err(CaptureManifestVerificationError::IdentityMismatch)
    );
}
