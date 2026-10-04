#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use originweave_core::Origin;
use originweave_evidence::{
    EvidenceError, EvidenceSourceKind, HttpMethod, MAX_HEADER_COUNT, MAX_METADATA_NAME_BYTES,
    MAX_METADATA_VALUE_BYTES, MAX_PATH_BYTES, MAX_PROVENANCE_TEXT_BYTES, MAX_QUERY_FIELD_COUNT,
    NetworkEvidence, ProvenanceRecord, VerificationResult,
};

const VALID_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn origin() -> Origin {
    Origin::parse("https://example.com").expect("origin")
}

#[test]
fn adversarial_header_and_query_variants_are_redacted_by_default() {
    let headers = BTreeMap::from([
        ("X-Auth-Token".to_owned(), "secret".to_owned()),
        ("X-Amz-Security-Token".to_owned(), "secret".to_owned()),
        ("X-Amz-Signature".to_owned(), "secret".to_owned()),
        (
            "Location".to_owned(),
            "https://example.com/private".to_owned(),
        ),
        (
            "Referer".to_owned(),
            "https://example.com/private".to_owned(),
        ),
        ("ETag".to_owned(), "Bearer secret".to_owned()),
        ("Content-Type".to_owned(), "client_secret=secret".to_owned()),
    ]);
    let query = BTreeMap::from([
        ("code".to_owned(), "oauth-code".to_owned()),
        ("client_secret".to_owned(), "secret".to_owned()),
        ("X-Amz-Credential".to_owned(), "credential".to_owned()),
        ("X-Amz-Signature".to_owned(), "signature".to_owned()),
    ]);
    let evidence =
        NetworkEvidence::capture(HttpMethod::Get, origin(), "/", headers, query).expect("evidence");

    assert!(
        evidence
            .headers()
            .values()
            .all(|value| value == "[REDACTED]")
    );
    assert!(evidence.query().values().all(|value| value == "[REDACTED]"));
}

/// Retain the exact admitted maximum path while rejecting oversized capture inputs.
#[test]
fn capture_enforces_path_count_and_value_boundaries() {
    assert_eq!(MAX_PATH_BYTES, 4_096);
    let maximum_path = format!("/{}", "a".repeat(4_095));
    let maximum_evidence = NetworkEvidence::capture(
        HttpMethod::Get,
        origin(),
        &maximum_path,
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .expect("path at maximum length");
    assert_eq!(maximum_evidence.path(), maximum_path);
    assert_eq!(
        NetworkEvidence::capture(
            HttpMethod::Get,
            origin(),
            &format!("/{}", "a".repeat(MAX_PATH_BYTES)),
            BTreeMap::new(),
            BTreeMap::new(),
        ),
        Err(EvidenceError::LimitExceeded)
    );

    let headers_at_limit = (0..MAX_HEADER_COUNT)
        .map(|index| (format!("x-field-{index}"), "value".to_owned()))
        .collect();
    NetworkEvidence::capture(
        HttpMethod::Get,
        origin(),
        "/",
        headers_at_limit,
        BTreeMap::new(),
    )
    .expect("header count at limit");
    let headers_over_limit = (0..=MAX_HEADER_COUNT)
        .map(|index| (format!("x-field-{index}"), "value".to_owned()))
        .collect();
    assert_eq!(
        NetworkEvidence::capture(
            HttpMethod::Get,
            origin(),
            "/",
            headers_over_limit,
            BTreeMap::new(),
        ),
        Err(EvidenceError::LimitExceeded)
    );

    let query_over_limit = (0..=MAX_QUERY_FIELD_COUNT)
        .map(|index| (format!("field-{index}"), "value".to_owned()))
        .collect();
    assert_eq!(
        NetworkEvidence::capture(
            HttpMethod::Get,
            origin(),
            "/",
            BTreeMap::new(),
            query_over_limit,
        ),
        Err(EvidenceError::LimitExceeded)
    );

    let oversized_value = "x".repeat(MAX_METADATA_VALUE_BYTES + 1);
    assert_eq!(
        NetworkEvidence::capture(
            HttpMethod::Get,
            origin(),
            "/",
            BTreeMap::from([("content-type".to_owned(), oversized_value)]),
            BTreeMap::new(),
        ),
        Err(EvidenceError::LimitExceeded)
    );
}

#[test]
fn exact_query_field_limit_preserves_names_and_redacts_values() {
    assert_eq!(MAX_QUERY_FIELD_COUNT, 128);
    let query = (0..128)
        .map(|index| {
            (
                format!("field-{index}"),
                format!("synthetic-private-{index}"),
            )
        })
        .collect();
    let evidence = NetworkEvidence::capture(HttpMethod::Get, origin(), "/", BTreeMap::new(), query)
        .expect("exactly 128 query fields are accepted");
    assert_eq!(evidence.query().len(), 128);
    for index in 0..128 {
        assert_eq!(
            evidence.query().get(&format!("field-{index}")),
            Some(&"[REDACTED]".to_owned()),
        );
    }
    let oversized = (0..129)
        .map(|index| (format!("field-{index}"), "synthetic-private".to_owned()))
        .collect();
    assert_eq!(
        NetworkEvidence::capture(HttpMethod::Get, origin(), "/", BTreeMap::new(), oversized),
        Err(EvidenceError::LimitExceeded),
    );
}

/// Retain the maximum admitted header name with its maximum input value redacted.
#[test]
fn capture_rejects_invalid_or_oversized_metadata_names() {
    for invalid_name in [
        String::new(),
        "x".repeat(MAX_METADATA_NAME_BYTES + 1),
        "bad name".to_owned(),
        "bad\nname".to_owned(),
    ] {
        assert_eq!(
            NetworkEvidence::capture(
                HttpMethod::Get,
                origin(),
                "/",
                BTreeMap::from([(invalid_name, "value".to_owned())]),
                BTreeMap::new(),
            ),
            Err(EvidenceError::LimitExceeded)
        );
    }

    assert_eq!(MAX_METADATA_NAME_BYTES, 256);
    assert_eq!(MAX_METADATA_VALUE_BYTES, 8_192);
    let exact_name = "x".repeat(256);
    let exact_evidence = NetworkEvidence::capture(
        HttpMethod::Get,
        origin(),
        "/",
        BTreeMap::from([(exact_name.clone(), "x".repeat(8_192))]),
        BTreeMap::new(),
    )
    .expect("metadata exactly at limits");
    assert_eq!(
        exact_evidence.headers(),
        &BTreeMap::from([(exact_name, "[REDACTED]".to_owned())])
    );
}

#[test]
fn capture_rejects_malformed_percent_escapes_and_ambiguous_segments() {
    for path in [
        "/bad%",
        "/bad%2",
        "/bad%zz",
        "/bad%2z",
        "/a/./b",
        "/a/../b",
        "/a/.",
        "/a/..",
        "/a/%2e/b",
        "/a/%2E%2e/b",
        "/a/%2f/b",
        "/a/%5c/b",
        "/a/%3f/b",
        "/a/%23/b",
        "/a/%20/b",
        "/a/%00/b",
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
            "path={path}"
        );
    }

    NetworkEvidence::capture(
        HttpMethod::Get,
        origin(),
        "/a/%7e/%AF/b",
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .expect("unambiguous percent encoding");
}

/// Retain exact provenance URL and locator bytes at their independently bounded limits.
#[test]
fn provenance_text_fields_are_bounded_before_retention() {
    assert_eq!(MAX_PROVENANCE_TEXT_BYTES, 8_192);
    let exact_url = format!("https://example.com/{}", "a".repeat(4_095));
    let exact_locator = "x".repeat(8_192);
    let exact = ProvenanceRecord::new(
        &exact_url,
        &exact_locator,
        VALID_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("maximum path and locator are accepted without truncation");
    assert_eq!(exact.source_url(), exact_url);
    assert_eq!(exact.source_locator(), exact_locator);

    let oversized_url = format!(
        "https://example.com/{}",
        "a".repeat(MAX_PROVENANCE_TEXT_BYTES)
    );
    assert_eq!(
        ProvenanceRecord::new(
            &oversized_url,
            "body",
            VALID_HASH,
            EvidenceSourceKind::NetworkResponse,
            VerificationResult::Verified,
        ),
        Err(EvidenceError::LimitExceeded)
    );

    let oversized_locator = "x".repeat(MAX_PROVENANCE_TEXT_BYTES + 1);
    assert_eq!(
        ProvenanceRecord::new(
            "https://example.com",
            &oversized_locator,
            VALID_HASH,
            EvidenceSourceKind::DomTree,
            VerificationResult::Unverified,
        ),
        Err(EvidenceError::LimitExceeded)
    );
}
