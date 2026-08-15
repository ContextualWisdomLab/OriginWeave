#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use originweave_core::Origin;
use originweave_evidence::{EvidenceError, HttpMethod, NetworkEvidence};

fn origin() -> Origin {
    Origin::parse("https://example.com").expect("valid test origin")
}

#[test]
fn network_evidence_rejects_non_token_http_header_field_names() {
    for header_name in [
        "bad:name",
        "bad,name",
        "bad=name",
        "bad(name)",
        "bad[name]",
        "bad{name}",
        "bad/name",
        "bad?name",
        "bad@name",
        "bad\"name",
        "bad\\name",
    ] {
        let headers = BTreeMap::from([(header_name.to_owned(), "discarded".to_owned())]);
        let result =
            NetworkEvidence::capture(HttpMethod::Get, origin(), "/", headers, BTreeMap::new());

        assert_eq!(
            result,
            Err(EvidenceError::LimitExceeded),
            "header_name={header_name:?}"
        );
    }
}

#[test]
fn network_evidence_keeps_header_token_grammar_separate_from_query_names() {
    let header_name = "X-Trace!#$%&'*+-.^_`|~09Az";
    let query_name = "filter:status";
    let evidence = NetworkEvidence::capture(
        HttpMethod::Get,
        origin(),
        "/search",
        BTreeMap::from([(header_name.to_owned(), "discarded".to_owned())]),
        BTreeMap::from([(query_name.to_owned(), "discarded".to_owned())]),
    )
    .expect("valid HTTP token field-name and independent query name");

    assert_eq!(
        evidence.headers().get(header_name),
        Some(&"[REDACTED]".to_owned())
    );
    assert_eq!(
        evidence.query().get(query_name),
        Some(&"[REDACTED]".to_owned())
    );
}
