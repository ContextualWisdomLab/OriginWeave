#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceSourceKind, ProvenanceRecord, VerificationResult, WarcResourceRecord,
};

#[test]
fn warc_record_debug_does_not_disclose_payload_or_provenance_locator() {
    let provenance = ProvenanceRecord::new(
        "https://example.com/resource",
        "private-selector-marker",
        "sha256:3684d4581255ca55e94c2cb89affc7d0dc914b7462c1f496780d7f2214877709",
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("verified provenance");
    let record = WarcResourceRecord::new(
        "urn:uuid:123e4567-e89b-12d3-a456-426614174000",
        "2026-08-21T09:00:00Z",
        "https://example.com/resource",
        "application/octet-stream",
        vec![254, 237, 250, 206],
        provenance,
    )
    .expect("WARC resource record");

    let debug = format!("{record:?}");
    assert!(debug.contains("payload_byte_count"));
    assert!(!debug.contains("254, 237, 250, 206"));
    assert!(!debug.contains("private-selector-marker"));
}

#[test]
fn warc_record_debug_does_not_disclose_content_type_parameters() {
    let provenance = ProvenanceRecord::new(
        "https://example.com/resource",
        "body",
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("verified provenance");
    let record = WarcResourceRecord::new(
        "urn:uuid:123e4567-e89b-12d3-a456-426614174000",
        "2026-08-21T09:00:00Z",
        "https://example.com/resource",
        "text/plain; token=secret",
        Vec::new(),
        provenance,
    )
    .expect("WARC resource record");

    let debug = format!("{record:?}");
    assert!(debug.contains("content_type: \"text/plain\""));
    assert!(!debug.contains("token=secret"));
}
