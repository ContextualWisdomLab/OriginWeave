#![allow(clippy::expect_used)]

use originweave_evidence::{
    CaptureManifest, CaptureManifestError, CaptureManifestValueBinding, EvidenceSourceKind,
    ExtractionCardinality, ExtractionField, ExtractionSchema, ExtractionSourceChannel,
    ExtractionValueType, ProvenanceRecord, VerificationResult, WarcPayloadCompleteness,
    WarcProvBundle, WarcResourceRecord, WarcTruncationReason,
};
use sha2::{Digest, Sha256};

const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const DATE: &str = "2026-08-29T00:00:00Z";
const SOFTWARE_COMMIT_SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const VALUE_HASH: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn structured_value_rejects_truncated_warc_source() {
    let schema = ExtractionSchema::new(
        "catalog-v3",
        vec![
            ExtractionField::new(
                "title",
                ExtractionValueType::Text,
                ExtractionCardinality::One,
                true,
                &[ExtractionSourceChannel::NetworkResponse],
            )
            .expect("title field"),
        ],
    )
    .expect("schema");

    let payload = b"partial-response";
    let source_hash = format!("sha256:{:x}", Sha256::digest(payload));
    let provenance = ProvenanceRecord::new(
        "https://example.com/item",
        "body",
        &source_hash,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("verified provenance");
    let record = WarcResourceRecord::new_with_completeness(
        RECORD_ID,
        DATE,
        "https://example.com/item",
        "text/plain",
        payload.to_vec(),
        provenance,
        WarcPayloadCompleteness::Truncated(WarcTruncationReason::Disconnect),
    )
    .expect("truncated WARC evidence remains representable");
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let value = CaptureManifestValueBinding::new("title", VALUE_HASH, RECORD_ID)
        .expect("structured value binding");

    assert_eq!(
        CaptureManifest::new_with_warc_values(
            &schema,
            &[(&record, &bundle)],
            std::slice::from_ref(&value),
        ),
        Err(CaptureManifestError::ValueSourceRecordTruncated)
    );
    assert_eq!(
        CaptureManifestError::ValueSourceRecordTruncated.to_string(),
        "capture manifest structured value references a truncated WARC record"
    );
}
