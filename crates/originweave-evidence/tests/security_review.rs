#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use originweave_core::Origin;
use originweave_evidence::{
    EvidenceError, HttpMethod, MAX_HEADER_COUNT, MAX_METADATA_VALUE_BYTES, MAX_PATH_BYTES,
    MAX_QUERY_FIELD_COUNT, NetworkEvidence,
};

fn origin() -> Origin {
    Origin::parse("https://example.com").expect("origin")
}

#[test]
fn adversarial_header_and_query_variants_are_redacted_by_default() {
    let headers = BTreeMap::from([
        ("X-Auth-Token".to_owned(), "secret".to_owned()),
        ("X-Amz-Security-Token".to_owned(), "secret".to_owned()),
        ("X-Amz-Signature".to_owned(), "secret".to_owned()),
        ("Location".to_owned(), "https://example.com/private".to_owned()),
        ("Referer".to_owned(), "https://example.com/private".to_owned()),
    ]);
    let query = BTreeMap::from([
        ("code".to_owned(), "oauth-code".to_owned()),
        ("client_secret".to_owned(), "secret".to_owned()),
        ("X-Amz-Credential".to_owned(), "credential".to_owned()),
        ("X-Amz-Signature".to_owned(), "signature".to_owned()),
    ]);
    let evidence = NetworkEvidence::capture(HttpMethod::Get, origin(), "/", headers, query)
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
fn capture_enforces_path_count_and_value_boundaries() {
    let maximum_path = format!("/{}", "a".repeat(MAX_PATH_BYTES - 1));
    NetworkEvidence::capture(
        HttpMethod::Get,
        origin(),
        &maximum_path,
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .expect("path at maximum length");
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
fn capture_rejects_malformed_percent_escapes_and_ambiguous_segments() {
    for path in [
        "/bad%",
        "/bad%2",
        "/bad%zz",
        "/a/./b",
        "/a/../b",
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
}
