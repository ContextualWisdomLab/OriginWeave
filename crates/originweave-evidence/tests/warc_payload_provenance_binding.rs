#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceSourceKind, ProvenanceRecord, VerificationResult, WarcPayloadCompleteness,
    WarcResourceRecord, WarcResourceRecordError, WarcTruncationReason,
};

const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const DATE: &str = "2026-08-28T00:00:00Z";
const SOURCE_URL: &str = "https://example.com/item";
const HELLO_HASH: &str = "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
const OTHER_HASH: &str = "sha256:486ea46224d1bb4fb680f34f7c9ad96a8f24ec88be73ea8e5a6c65260e9cb8a7";

fn provenance(source_hash: &str) -> ProvenanceRecord {
    ProvenanceRecord::new(
        SOURCE_URL,
        "body",
        source_hash,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("valid provenance")
}

#[test]
fn complete_payload_must_match_verified_source_hash() {
    let record = WarcResourceRecord::new(
        RECORD_ID,
        DATE,
        SOURCE_URL,
        "text/plain",
        b"hello".to_vec(),
        provenance(HELLO_HASH),
    )
    .expect("matching payload provenance");
    assert_eq!(record.block_digest(), HELLO_HASH);

    assert_eq!(
        WarcResourceRecord::new(
            RECORD_ID,
            DATE,
            SOURCE_URL,
            "text/plain",
            b"hello".to_vec(),
            provenance(OTHER_HASH),
        ),
        Err(WarcResourceRecordError::PayloadProvenanceMismatch)
    );
}

#[test]
fn truncated_payload_still_binds_the_exact_retained_bytes() {
    assert_eq!(
        WarcResourceRecord::new_with_completeness(
            RECORD_ID,
            DATE,
            SOURCE_URL,
            "text/plain",
            b"hello".to_vec(),
            provenance(OTHER_HASH),
            WarcPayloadCompleteness::Truncated(WarcTruncationReason::Length),
        ),
        Err(WarcResourceRecordError::PayloadProvenanceMismatch)
    );
}
