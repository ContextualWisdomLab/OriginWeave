use originweave_evidence::{
    EvidenceSourceKind, ProvenanceRecord, VerificationResult, WarcResourceRecord,
};

#[test]
fn warc_record_debug_does_not_disclose_payload_or_provenance_locator() {
    let provenance = ProvenanceRecord::new(
        "https://example.com/resource",
        "private-selector-marker",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
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
