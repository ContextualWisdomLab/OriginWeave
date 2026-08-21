#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceSourceKind, MAX_WARC_PAYLOAD_BYTES, ProvenanceRecord, VerificationResult,
    WarcResourceRecord, WarcResourceRecordError,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const DATE: &str = "2026-08-21T00:00:00Z";

fn provenance(source_url: &str, verification: VerificationResult) -> ProvenanceRecord {
    ProvenanceRecord::new(
        source_url,
        "body",
        SOURCE_HASH,
        EvidenceSourceKind::NetworkResponse,
        verification,
    )
    .expect("provenance")
}

#[test]
fn resource_record_binds_verified_provenance_and_emits_deterministic_warc_bytes() {
    let record = WarcResourceRecord::new(
        RECORD_ID,
        DATE,
        "https://example.com/item",
        "text/plain",
        b"hello".to_vec(),
        provenance("https://example.com/item", VerificationResult::Verified),
    )
    .expect("resource record");

    assert_eq!(record.record_id(), RECORD_ID);
    assert_eq!(record.warc_date(), DATE);
    assert_eq!(record.target_uri(), "https://example.com/item");
    assert_eq!(record.content_type(), "text/plain");
    assert_eq!(record.payload(), b"hello");
    assert_eq!(
        record.block_digest(),
        "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    assert_eq!(record.provenance().source_url(), record.target_uri());
    assert!(record.provenance().verification_result() == VerificationResult::Verified);
    assert_eq!(
        record.to_warc_bytes(),
        b"WARC/1.1\r\nWARC-Type: resource\r\nWARC-Record-ID: <urn:uuid:123e4567-e89b-12d3-a456-426614174000>\r\nWARC-Date: 2026-08-21T00:00:00Z\r\nWARC-Target-URI: https://example.com/item\r\nContent-Type: text/plain\r\nWARC-Block-Digest: sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\r\nContent-Length: 5\r\n\r\nhello\r\n\r\n"
    );
    for date in [
        "2026-08-21T00:00:00.123Z",
        "2024-02-29T00:00:00Z",
        "2000-02-29T00:00:00Z",
    ] {
        WarcResourceRecord::new(
            RECORD_ID,
            date,
            "https://example.com/item",
            "text/plain",
            Vec::new(),
            provenance("https://example.com/item", VerificationResult::Verified),
        )
        .expect("valid UTC date");
    }
}

#[test]
fn resource_record_rejects_invalid_identifiers_dates_content_and_limits() {
    let valid = |record_id, date, content_type, payload| {
        WarcResourceRecord::new(
            record_id,
            date,
            "https://example.com/item",
            content_type,
            payload,
            provenance("https://example.com/item", VerificationResult::Verified),
        )
    };

    for record_id in [
        "",
        "http://example.com/record",
        "urn:uuid:123e4567-e89b-12d3-a456-42661417400",
        "urn:uuid:123e4567_e89b-12d3-a456-426614174000",
        "urn:uuid:123e4567-e89b-12d3-a456-42661417400z",
        "xrn:uuid:123e4567-e89b-12d3-a456-426614174000",
    ] {
        assert_eq!(
            valid(record_id, DATE, "text/plain", Vec::new()),
            Err(WarcResourceRecordError::InvalidRecordId),
            "record_id={record_id:?}"
        );
    }

    for date in [
        "",
        "2026-08-21 00:00:00Z",
        "2026-13-21T00:00:00Z",
        "2026-08-32T00:00:00Z",
        "2026-02-29T00:00:00Z",
        "2024-02-30T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "1900-02-29T00:00:00Z",
        "2026-08-21T24:00:00Z",
        "2026-0x-21T00:00:00Z",
        "2026-08-21T00:00:00+00:00",
        "2026-08-21T00:00:00.123",
        "2026-08-21T00:00:00.XZ",
        "2026-08-21T00:00:00.Z",
        "2026x08-21T00:00:00Z",
        "2026-08x21T00:00:00Z",
        "2026-08-21T00x00:00Z",
        "2026-08-21T00:00x00Z",
        "2026-08-21T00:00:00.12345678901234567890Z",
        "2026-08-21T00:00:00X",
        "2026-08-21T00:61:00Z",
        "2026-08-21T00:00:61Z",
    ] {
        assert_eq!(
            valid(RECORD_ID, date, "text/plain", Vec::new()),
            Err(WarcResourceRecordError::InvalidDate),
            "date={date:?}"
        );
    }

    for content_type in ["", "text plain", "text\nplain"] {
        assert_eq!(
            valid(RECORD_ID, DATE, content_type, Vec::new()),
            Err(WarcResourceRecordError::InvalidContentType),
            "content_type={content_type:?}"
        );
    }

    assert_eq!(
        valid(
            RECORD_ID,
            DATE,
            "text/plain",
            vec![b'x'; MAX_WARC_PAYLOAD_BYTES + 1],
        ),
        Err(WarcResourceRecordError::LimitExceeded)
    );
    assert_eq!(
        valid(
            RECORD_ID,
            DATE,
            &"x".repeat(originweave_evidence::MAX_WARC_CONTENT_TYPE_BYTES + 1),
            Vec::new(),
        ),
        Err(WarcResourceRecordError::LimitExceeded)
    );
}

#[test]
fn resource_record_rejects_provenance_drift_and_unverified_sources() {
    assert_eq!(
        WarcResourceRecord::new(
            RECORD_ID,
            DATE,
            "https://example.com/other",
            "text/plain",
            Vec::new(),
            provenance("https://example.com/item", VerificationResult::Verified),
        ),
        Err(WarcResourceRecordError::TargetUriMismatch)
    );
    for verification in [VerificationResult::Unverified, VerificationResult::Rejected] {
        assert_eq!(
            WarcResourceRecord::new(
                RECORD_ID,
                DATE,
                "https://example.com/item",
                "text/plain",
                Vec::new(),
                provenance("https://example.com/item", verification),
            ),
            Err(WarcResourceRecordError::UnverifiedProvenance)
        );
    }
}
