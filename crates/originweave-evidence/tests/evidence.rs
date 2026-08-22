#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use originweave_core::Origin;
use originweave_evidence::{
    EvidenceError, EvidenceSourceKind, HttpMethod, NetworkEvidence, ProvenanceRecord,
    VerificationResult,
};

const VALID_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn origin() -> Origin {
    Origin::parse("https://example.com").expect("origin")
}

#[test]
fn network_evidence_discards_every_metadata_value() {
    let headers = BTreeMap::from([
        ("Authorization".to_owned(), "Bearer secret".to_owned()),
        ("Cookie".to_owned(), "session=secret".to_owned()),
        ("Content-Type".to_owned(), "application/json".to_owned()),
        ("Accept".to_owned(), "application/json".to_owned()),
        ("X-Custom-Session".to_owned(), "secret".to_owned()),
    ]);
    let query = BTreeMap::from([
        ("access_token".to_owned(), "secret".to_owned()),
        ("q".to_owned(), "private search text".to_owned()),
    ]);
    let evidence = NetworkEvidence::capture(HttpMethod::Get, origin(), "/search", headers, query)
        .expect("evidence");

    assert_eq!(evidence.method(), HttpMethod::Get);
    assert_eq!(evidence.origin().as_str(), "https://example.com");
    assert_eq!(evidence.path(), "/search");
    assert!(
        evidence
            .headers()
            .values()
            .all(|value| value == "[REDACTED]")
    );
    assert!(evidence.query().values().all(|value| value == "[REDACTED]"));
}

#[test]
fn metadata_redaction_is_independent_of_field_name_case() {
    let headers = BTreeMap::from([
        ("PROXY-AUTHORIZATION".to_owned(), "x".to_owned()),
        ("Set-Cookie".to_owned(), "x".to_owned()),
        ("X-API-Key".to_owned(), "x".to_owned()),
        ("X-CSRF-Token".to_owned(), "x".to_owned()),
        ("ETAG".to_owned(), "safe-etag".to_owned()),
    ]);
    let query = BTreeMap::from([
        ("API_KEY".to_owned(), "x".to_owned()),
        ("key".to_owned(), "x".to_owned()),
        ("TOKEN".to_owned(), "x".to_owned()),
        ("secret".to_owned(), "x".to_owned()),
        ("Password".to_owned(), "x".to_owned()),
    ]);
    let evidence = NetworkEvidence::capture(HttpMethod::Post, origin(), "/submit", headers, query)
        .expect("evidence");

    assert!(
        evidence
            .headers()
            .values()
            .all(|value| value == "[REDACTED]")
    );
    assert!(evidence.query().values().all(|value| value == "[REDACTED]"));
}

#[test]
fn network_evidence_rejects_non_path_inputs() {
    for path in [
        "",
        "relative",
        "/path?secret=x",
        "/path#fragment",
        "/bad\npath",
        "/bad path",
        "/windows\\path",
        "/[segment]",
        "/raw|pipe",
        "/raw-한글",
    ] {
        assert_eq!(
            NetworkEvidence::capture(
                HttpMethod::Get,
                origin(),
                path,
                BTreeMap::new(),
                BTreeMap::new(),
            ),
            Err(EvidenceError::InvalidPath),
            "path={path:?}"
        );
    }
}

#[test]
fn provenance_accepts_safe_root_path_and_loopback_sources() {
    for source_url in [
        "https://example.com",
        "https://example.com/item/42",
        "http://localhost:9222/json/version",
        "http://[::1]:9222/json/version",
    ] {
        let record = ProvenanceRecord::new(
            source_url,
            "body",
            VALID_HASH,
            EvidenceSourceKind::StructuredData,
            VerificationResult::Verified,
        )
        .expect("safe provenance source");
        assert_eq!(record.source_url(), source_url);
    }
}

#[test]
fn provenance_rejects_credential_bearing_or_ambiguous_source_urls() {
    for source_url in [
        "",
        "example.com/path",
        "ftp://example.com/path",
        "http://example.com/path",
        "https://user:password@example.com/path",
        "https://example.com/path?access_token=secret",
        "https://example.com/path#fragment",
        "https://example.com/bad\\path",
        "https://example.com/\n",
        "https://example.com/a/%2f/b",
        "https://example.com/[segment]",
    ] {
        assert_eq!(
            ProvenanceRecord::new(
                source_url,
                "body",
                VALID_HASH,
                EvidenceSourceKind::VisualCapture,
                VerificationResult::Verified,
            ),
            Err(EvidenceError::InvalidSourceUrl),
            "source_url={source_url:?}"
        );
    }
}

#[test]
fn provenance_requires_locator_and_sha256_evidence() {
    let record = ProvenanceRecord::new(
        "https://example.com/item/42",
        "$.product.sale_price",
        VALID_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("provenance");
    assert_eq!(record.source_url(), "https://example.com/item/42");
    assert_eq!(record.source_locator(), "$.product.sale_price");
    assert_eq!(record.source_hash(), VALID_HASH);
    assert_eq!(record.source_kind(), EvidenceSourceKind::NetworkResponse);
    assert_eq!(record.verification_result(), VerificationResult::Verified);

    assert_eq!(
        ProvenanceRecord::new(
            "https://example.com",
            "",
            record.source_hash(),
            EvidenceSourceKind::DomTree,
            VerificationResult::Unverified,
        ),
        Err(EvidenceError::EmptyLocator)
    );

    for invalid_hash in [
        "not-prefixed",
        "sha256:not-a-hash",
        "sha256:0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF",
        "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
    ] {
        assert_eq!(
            ProvenanceRecord::new(
                "https://example.com",
                "body",
                invalid_hash,
                EvidenceSourceKind::AccessibilityTree,
                VerificationResult::Rejected,
            ),
            Err(EvidenceError::InvalidHash),
            "invalid_hash={invalid_hash}"
        );
    }
}

#[test]
fn all_http_methods_are_representable() {
    let methods = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Patch,
        HttpMethod::Delete,
        HttpMethod::Head,
        HttpMethod::Options,
    ];
    assert_eq!(methods.len(), 7);
}
