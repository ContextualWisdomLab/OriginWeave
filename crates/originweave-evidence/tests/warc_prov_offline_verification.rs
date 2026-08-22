#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceSourceKind, ProvenanceRecord, VerificationResult, WarcPayloadCompleteness,
    WarcProvBundle, WarcProvBundleVerificationError, WarcResourceRecord, WarcTruncationReason,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const OTHER_SOURCE_HASH: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const OTHER_RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174001";
const DATE: &str = "2026-08-22T12:00:00Z";
const OTHER_DATE: &str = "2026-08-22T12:00:01Z";
const SOURCE_URL: &str = "https://example.com/item";
const OTHER_SOURCE_URL: &str = "https://example.com/other";
const SOFTWARE_COMMIT_SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn record_with_provenance(
    record_id: &str,
    date: &str,
    source_url: &str,
    source_locator: &str,
    source_hash: &str,
    source_kind: EvidenceSourceKind,
    content_type: &str,
    payload: &[u8],
    completeness: WarcPayloadCompleteness,
) -> WarcResourceRecord {
    let provenance = ProvenanceRecord::new(
        source_url,
        source_locator,
        source_hash,
        source_kind,
        VerificationResult::Verified,
    )
    .expect("verified provenance");
    WarcResourceRecord::new_with_completeness(
        record_id,
        date,
        source_url,
        content_type,
        payload.to_vec(),
        provenance,
        completeness,
    )
    .expect("WARC resource record")
}

fn record(
    record_id: &str,
    date: &str,
    source_url: &str,
    source_hash: &str,
    content_type: &str,
    payload: &[u8],
    completeness: WarcPayloadCompleteness,
) -> WarcResourceRecord {
    record_with_provenance(
        record_id,
        date,
        source_url,
        "body",
        source_hash,
        EvidenceSourceKind::NetworkResponse,
        content_type,
        payload,
        completeness,
    )
}

fn baseline_record() -> WarcResourceRecord {
    record(
        RECORD_ID,
        DATE,
        SOURCE_URL,
        SOURCE_HASH,
        "text/plain",
        b"hello",
        WarcPayloadCompleteness::Complete,
    )
}

fn assert_standard_error_contract<E: std::error::Error + Send + Sync + 'static>() {}

#[test]
fn warc_prov_bundle_offline_verification_accepts_only_the_exact_bound_record() {
    assert_standard_error_contract::<WarcProvBundleVerificationError>();
    assert_eq!(
        WarcProvBundleVerificationError::RecordIdentityMismatch.to_string(),
        "WARC PROV record identity does not match"
    );
    assert_eq!(
        WarcProvBundleVerificationError::SourceEvidenceMismatch.to_string(),
        "WARC PROV source evidence does not match"
    );
    assert_eq!(
        WarcProvBundleVerificationError::CaptureTimeMismatch.to_string(),
        "WARC PROV capture time does not match"
    );
    assert_eq!(
        WarcProvBundleVerificationError::PayloadDigestMismatch.to_string(),
        "WARC PROV payload digest does not match"
    );
    assert_eq!(
        WarcProvBundleVerificationError::PayloadCompletenessMismatch.to_string(),
        "WARC PROV payload completeness does not match"
    );
    assert_eq!(
        WarcProvBundleVerificationError::WarcRecordDigestMismatch.to_string(),
        "WARC PROV serialized record digest does not match"
    );

    let exact = baseline_record();
    let bundle = WarcProvBundle::new(&exact, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    assert_eq!(bundle.verify_record(&exact), Ok(()));

    let mismatches = [
        (
            record(
                OTHER_RECORD_ID,
                DATE,
                SOURCE_URL,
                SOURCE_HASH,
                "text/plain",
                b"hello",
                WarcPayloadCompleteness::Complete,
            ),
            WarcProvBundleVerificationError::RecordIdentityMismatch,
        ),
        (
            record(
                RECORD_ID,
                DATE,
                OTHER_SOURCE_URL,
                SOURCE_HASH,
                "text/plain",
                b"hello",
                WarcPayloadCompleteness::Complete,
            ),
            WarcProvBundleVerificationError::SourceEvidenceMismatch,
        ),
        (
            record(
                RECORD_ID,
                DATE,
                SOURCE_URL,
                OTHER_SOURCE_HASH,
                "text/plain",
                b"hello",
                WarcPayloadCompleteness::Complete,
            ),
            WarcProvBundleVerificationError::SourceEvidenceMismatch,
        ),
        (
            record_with_provenance(
                RECORD_ID,
                DATE,
                SOURCE_URL,
                "different-body",
                SOURCE_HASH,
                EvidenceSourceKind::NetworkResponse,
                "text/plain",
                b"hello",
                WarcPayloadCompleteness::Complete,
            ),
            WarcProvBundleVerificationError::SourceEvidenceMismatch,
        ),
        (
            record_with_provenance(
                RECORD_ID,
                DATE,
                SOURCE_URL,
                "body",
                SOURCE_HASH,
                EvidenceSourceKind::StructuredData,
                "text/plain",
                b"hello",
                WarcPayloadCompleteness::Complete,
            ),
            WarcProvBundleVerificationError::SourceEvidenceMismatch,
        ),
        (
            record(
                RECORD_ID,
                OTHER_DATE,
                SOURCE_URL,
                SOURCE_HASH,
                "text/plain",
                b"hello",
                WarcPayloadCompleteness::Complete,
            ),
            WarcProvBundleVerificationError::CaptureTimeMismatch,
        ),
        (
            record(
                RECORD_ID,
                DATE,
                SOURCE_URL,
                SOURCE_HASH,
                "text/plain",
                b"world",
                WarcPayloadCompleteness::Complete,
            ),
            WarcProvBundleVerificationError::PayloadDigestMismatch,
        ),
        (
            record(
                RECORD_ID,
                DATE,
                SOURCE_URL,
                SOURCE_HASH,
                "text/plain",
                b"hello",
                WarcPayloadCompleteness::Truncated(WarcTruncationReason::Length),
            ),
            WarcProvBundleVerificationError::PayloadCompletenessMismatch,
        ),
        (
            record(
                RECORD_ID,
                DATE,
                SOURCE_URL,
                SOURCE_HASH,
                "application/octet-stream",
                b"hello",
                WarcPayloadCompleteness::Complete,
            ),
            WarcProvBundleVerificationError::WarcRecordDigestMismatch,
        ),
    ];

    for (candidate, expected) in mismatches {
        assert_eq!(bundle.verify_record(&candidate), Err(expected));
    }
}
