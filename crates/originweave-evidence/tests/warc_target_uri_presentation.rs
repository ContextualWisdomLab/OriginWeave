#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceSourceKind, ProvenanceRecord, VerificationResult, WarcResourceRecord,
    WarcResourceRecordError,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const DATE: &str = "2026-08-21T00:00:00Z";

fn provenance(source_url: &str) -> ProvenanceRecord {
    ProvenanceRecord::new(
        source_url,
        "body",
        SOURCE_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("provenance")
}

#[test]
fn warc_target_uri_rejects_invisible_formatting_characters_before_serialization() {
    for formatting_character in [
        '\u{00ad}', '\u{061c}', '\u{200b}', '\u{200e}', '\u{202e}', '\u{2066}', '\u{2060}',
        '\u{feff}',
    ] {
        let target_uri = format!("https://example.com/item{formatting_character}shadow");
        let source_provenance = provenance(&target_uri);

        assert_eq!(
            WarcResourceRecord::new(
                RECORD_ID,
                DATE,
                &target_uri,
                "text/plain",
                Vec::new(),
                source_provenance,
            ),
            Err(WarcResourceRecordError::InvalidTargetUri),
            "target_uri={target_uri:?}"
        );
    }
}

#[test]
fn warc_target_uri_rejects_control_and_whitespace_before_provenance_comparison() {
    let source_provenance = provenance("https://example.com/item");
    for target_uri in [
        "https://example.com/item\rshadow",
        "https://example.com/item shadow",
    ] {
        assert_eq!(
            WarcResourceRecord::new(
                RECORD_ID,
                DATE,
                target_uri,
                "text/plain",
                Vec::new(),
                source_provenance.clone(),
            ),
            Err(WarcResourceRecordError::InvalidTargetUri),
            "target_uri={target_uri:?}"
        );
    }
}

#[test]
fn warc_target_uri_rejects_raw_unicode_because_warc_uses_rfc3986_uri_syntax() {
    let target_uri = "https://example.com/상품/상세";

    assert_eq!(
        WarcResourceRecord::new(
            RECORD_ID,
            DATE,
            target_uri,
            "text/plain",
            Vec::new(),
            provenance(target_uri),
        ),
        Err(WarcResourceRecordError::InvalidTargetUri),
    );
}

#[test]
fn warc_target_uri_rejects_ascii_characters_outside_rfc3986_uri_syntax() {
    for invalid_character in ['<', '>', '"', '{', '}', '|', '^', '`'] {
        let target_uri = format!("https://example.com/item{invalid_character}shadow");

        assert_eq!(
            WarcResourceRecord::new(
                RECORD_ID,
                DATE,
                &target_uri,
                "text/plain",
                Vec::new(),
                provenance(&target_uri),
            ),
            Err(WarcResourceRecordError::InvalidTargetUri),
            "target_uri={target_uri:?}"
        );
    }
}

#[test]
fn warc_target_uri_accepts_percent_encoded_utf8_path_octets() {
    let target_uri = "https://example.com/%EC%83%81%ED%92%88/%EC%83%81%EC%84%B8";
    let record = WarcResourceRecord::new(
        RECORD_ID,
        DATE,
        target_uri,
        "text/plain",
        Vec::new(),
        provenance(target_uri),
    )
    .expect("RFC 3986 percent-encoded target URI");

    assert_eq!(record.target_uri(), target_uri);
}
