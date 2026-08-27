#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceSourceKind, ProvenanceRecord, VerificationResult, WarcResourceRecord,
    WarcResourceRecordError,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const DATE: &str = "2026-08-21T00:00:00Z";
const TARGET_URI: &str = "https://example.com/item";

fn provenance() -> ProvenanceRecord {
    ProvenanceRecord::new(
        TARGET_URI,
        "body",
        SOURCE_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("valid provenance")
}

#[test]
fn oversized_record_fields_report_the_bounded_limit_error() {
    let oversized_record_id = format!("{RECORD_ID}x");
    assert_eq!(
        WarcResourceRecord::new(
            &oversized_record_id,
            DATE,
            TARGET_URI,
            "text/plain",
            Vec::new(),
            provenance(),
        ),
        Err(WarcResourceRecordError::LimitExceeded),
    );

    let oversized_date = "2026-08-21T00:00:00.1234567890Z";
    assert_eq!(
        WarcResourceRecord::new(
            RECORD_ID,
            oversized_date,
            TARGET_URI,
            "text/plain",
            Vec::new(),
            provenance(),
        ),
        Err(WarcResourceRecordError::LimitExceeded),
    );
}
