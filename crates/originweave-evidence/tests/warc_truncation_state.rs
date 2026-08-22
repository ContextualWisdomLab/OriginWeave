#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceSourceKind, ProvenanceRecord, VerificationResult, WarcPayloadCompleteness,
    WarcResourceRecord, WarcTruncationReason,
};

const RECORD_ID: &str = "urn:uuid:01234567-89ab-cdef-0123-456789abcdef";
const DATE: &str = "2026-08-22T00:00:00Z";
const SOURCE_URL: &str = "https://example.com/resource";
const SOURCE_HASH: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn verified_provenance() -> ProvenanceRecord {
    ProvenanceRecord::new(
        SOURCE_URL,
        "body",
        SOURCE_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("reviewed provenance fixture must be valid")
}

#[test]
fn warc_resource_records_preserve_explicit_completeness_and_truncation_reason() {
    let complete = WarcResourceRecord::new_with_completeness(
        RECORD_ID,
        DATE,
        SOURCE_URL,
        "text/plain",
        b"hello".to_vec(),
        verified_provenance(),
        WarcPayloadCompleteness::Complete,
    )
    .expect("complete capture must be admitted");
    assert_eq!(complete.completeness(), WarcPayloadCompleteness::Complete);
    let complete_bytes = String::from_utf8(complete.to_warc_bytes())
        .expect("text fixture must serialize as UTF-8 WARC bytes");
    assert!(!complete_bytes.contains("WARC-Truncated:"));

    for (reason, token) in [
        (WarcTruncationReason::Length, "length"),
        (WarcTruncationReason::Time, "time"),
        (WarcTruncationReason::Disconnect, "disconnect"),
        (WarcTruncationReason::Unspecified, "unspecified"),
    ] {
        let completeness = WarcPayloadCompleteness::Truncated(reason);
        let truncated = WarcResourceRecord::new_with_completeness(
            RECORD_ID,
            DATE,
            SOURCE_URL,
            "text/plain",
            b"hello".to_vec(),
            verified_provenance(),
            completeness,
        )
        .expect("typed truncated capture must be admitted");

        assert_eq!(truncated.completeness(), completeness);
        let truncated_bytes = String::from_utf8(truncated.to_warc_bytes())
            .expect("text fixture must serialize as UTF-8 WARC bytes");
        assert!(truncated_bytes.contains(&format!("WARC-Truncated: {token}\r\n")));
        assert!(truncated_bytes.contains("Content-Length: 5\r\n"));
    }
}
