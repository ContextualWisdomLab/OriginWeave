#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceError, EvidenceSourceKind, ProvenanceRecord, VerificationResult, WarcResourceRecord,
};

const EMPTY_SHA256: &str =
    "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const DATE: &str = "2026-08-28T00:00:00Z";

#[test]
fn provenance_and_warc_preserve_query_bearing_resource_urls_without_debug_disclosure() {
    let source_url = "https://example.com/search?next=/products?category=widgets&q=public%20term";
    let provenance = ProvenanceRecord::new(
        source_url,
        "body",
        EMPTY_SHA256,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("query-bearing provenance URL should be accepted");

    assert_eq!(provenance.source_url(), source_url);
    let provenance_debug = format!("{provenance:?}");
    assert!(!provenance_debug.contains(source_url));
    assert!(!provenance_debug.contains("public%20term"));
    assert!(!provenance_debug.contains(EMPTY_SHA256));
    assert!(!provenance_debug.contains("body"));

    let record = WarcResourceRecord::new(
        RECORD_ID,
        DATE,
        source_url,
        "text/plain",
        Vec::new(),
        provenance,
    )
    .expect("query-bearing WARC target URI should be accepted");

    assert_eq!(record.target_uri(), source_url);
    let warc = String::from_utf8(record.to_warc_bytes()).expect("bounded WARC bytes are UTF-8");
    assert!(warc.contains(&format!("WARC-Target-URI: {source_url}\r\n")));
}

#[test]
fn provenance_query_support_rejects_credential_fields() {
    for source_url in [
        "https://example.com/callback?access_token=secret",
        "https://example.com/callback?ACCESS-TOKEN=secret",
        "https://example.com/callback?access%5Ftoken=secret",
        "https://example.com/download?api_key=secret",
        "https://example.com/download?client_secret=secret",
        "https://example.com/download?X-Amz-Credential=secret",
        "https://example.com/download?X-Amz-Signature=secret",
        "https://example.com/download?X%2Damz%2DSignature=secret",
        "https://example.com/download?x-goog-credential=secret",
        "https://example.com/login?password=secret",
        "https://example.com/login?auth=secret",
        "https://example.com/login?sig=secret",
        "https://example.com/callback?redirect=https://example.com/landing?token=secret",
        "https://example.com/callback?redirect=https%3A%2F%2Fexample.com%2Flanding%3Ftoken%3Dsecret",
    ] {
        assert_eq!(
            ProvenanceRecord::new(
                source_url,
                "body",
                EMPTY_SHA256,
                EvidenceSourceKind::NetworkResponse,
                VerificationResult::Verified,
            ),
            Err(EvidenceError::InvalidSourceUrl),
            "credential-bearing source_url={source_url:?}"
        );
    }
}

#[test]
fn provenance_query_support_does_not_admit_fragments_or_unsafe_uri_octets() {
    for source_url in [
        "https://example.com/search?q=value#fragment",
        "https://example.com/search#fragment",
        "https://example.com/search?q=bad value",
        "https://example.com/search?q=bad\\value",
        "https://example.com/search?q=raw|pipe",
        "https://example.com/search?q=raw-한글",
        "https://example.com/search?q=%",
        "https://example.com/search?q=%2",
        "https://example.com/search?q=%GG",
        "https://example.com/search?q=%2G",
    ] {
        assert_eq!(
            ProvenanceRecord::new(
                source_url,
                "body",
                EMPTY_SHA256,
                EvidenceSourceKind::NetworkResponse,
                VerificationResult::Verified,
            ),
            Err(EvidenceError::InvalidSourceUrl),
            "source_url={source_url:?}"
        );
    }
}
