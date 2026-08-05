#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use originweave_core::Origin;
use originweave_evidence::{
    EvidenceError, EvidenceSourceKind, HttpMethod, NetworkEvidence, ProvenanceRecord,
    VerificationResult,
};

fn origin() -> Origin {
    Origin::parse("https://example.com").expect("origin")
}

#[test]
fn network_evidence_redacts_credentials_but_preserves_safe_context() {
    let headers = BTreeMap::from([
        ("Authorization".to_owned(), "Bearer secret".to_owned()),
        ("Cookie".to_owned(), "session=secret".to_owned()),
        ("Content-Type".to_owned(), "application/json".to_owned()),
    ]);
    let query = BTreeMap::from([
        ("access_token".to_owned(), "secret".to_owned()),
        ("q".to_owned(), "browser".to_owned()),
    ]);
    let evidence = NetworkEvidence::capture(HttpMethod::Get, origin(), "/search", headers, query)
        .expect("evidence");

    assert_eq!(evidence.method(), HttpMethod::Get);
    assert_eq!(evidence.origin().as_str(), "https://example.com");
    assert_eq!(evidence.path(), "/search");
    assert_eq!(evidence.headers()["Authorization"], "[REDACTED]");
    assert_eq!(evidence.headers()["Cookie"], "[REDACTED]");
    assert_eq!(evidence.headers()["Content-Type"], "application/json");
    assert_eq!(evidence.query()["access_token"], "[REDACTED]");
    assert_eq!(evidence.query()["q"], "browser");
}

#[test]
fn every_sensitive_header_and_query_name_is_case_insensitively_redacted() {
    let headers = BTreeMap::from([
        ("PROXY-AUTHORIZATION".to_owned(), "x".to_owned()),
        ("Set-Cookie".to_owned(), "x".to_owned()),
        ("X-API-Key".to_owned(), "x".to_owned()),
        ("X-CSRF-Token".to_owned(), "x".to_owned()),
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
fn provenance_requires_locator_and_sha256_evidence() {
    let record = ProvenanceRecord::new(
        "https://example.com/item/42",
        "$.product.sale_price",
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("provenance");
    assert_eq!(record.source_url(), "https://example.com/item/42");
    assert_eq!(record.source_locator(), "$.product.sale_price");
    assert_eq!(
        record.source_hash(),
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    );
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
    assert_eq!(
        ProvenanceRecord::new(
            "https://example.com",
            "body",
            "sha256:not-a-hash",
            EvidenceSourceKind::AccessibilityTree,
            VerificationResult::Rejected,
        ),
        Err(EvidenceError::InvalidHash)
    );
    assert_eq!(
        ProvenanceRecord::new(
            "",
            "body",
            record.source_hash(),
            EvidenceSourceKind::VisualCapture,
            VerificationResult::Verified,
        ),
        Err(EvidenceError::InvalidSourceUrl)
    );
    assert_eq!(
        ProvenanceRecord::new(
            "https://example.com/\n",
            "body",
            record.source_hash(),
            EvidenceSourceKind::StructuredData,
            VerificationResult::Verified,
        ),
        Err(EvidenceError::InvalidSourceUrl)
    );
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
