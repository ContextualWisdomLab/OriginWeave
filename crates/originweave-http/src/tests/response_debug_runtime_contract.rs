#![allow(clippy::expect_used)]

use std::net::SocketAddr;
use std::time::Duration;

use originweave_core::Origin;
use originweave_tls::{NegotiatedAlpn, TlsProtocolVersion};

use crate::evidence::{
    AuthenticatedHttpResponse, EvidenceInput, HttpExchangeEvidence, HttpResourceBudgets,
};
use crate::mime::classify_observed_mime;
use crate::{
    BodyFraming, ContentCoding, HttpClientPolicy, HttpMethod, IntegrityStatus, MimeMismatch,
    NoSniffStatus,
};

#[test]
fn response_debug_executes_without_exposing_untrusted_bytes() {
    let content = b"secret-body".to_vec();
    let reason_phrase = b"TOP-SECRET-REASON".to_vec();
    let policy = HttpClientPolicy::strict_defaults();
    let socket_address = SocketAddr::from(([127, 0, 0, 1], 443));
    let evidence = HttpExchangeEvidence::from(EvidenceInput {
        origin: Origin::parse("https://example.com").expect("fixture origin"),
        requested_peer: socket_address,
        observed_peer: socket_address,
        tls_protocol_version: TlsProtocolVersion::Tls13,
        negotiated_alpn: NegotiatedAlpn::Protocol(b"http/1.1".to_vec()),
        method: HttpMethod::Get,
        target_hash: format!("sha256:{}", "0".repeat(64)),
        query_present: false,
        path_prefix: "/".to_owned(),
        status_code: 200,
        interim_response_count: 0,
        response_fields: Vec::new(),
        body_framing: BodyFraming::ContentLength(content.len()),
        encoded_content_bytes: content.len(),
        decoded_content_bytes: content.len(),
        content_coding: ContentCoding::Identity,
        chunk_count: 0,
        trailer_fields: Vec::new(),
        content_digest_status: IntegrityStatus::Absent,
        representation_digest_status: IntegrityStatus::Absent,
        supplied_mime: None,
        observed_mime: classify_observed_mime(&content, None),
        no_sniff_status: NoSniffStatus::Absent,
        mime_mismatch: MimeMismatch::ObservedOnly,
        content_disposition: None,
        redirect: None,
        exchange_duration: Duration::from_millis(1),
        resource_budgets: HttpResourceBudgets::from_policy(&policy),
    });
    let response = AuthenticatedHttpResponse {
        content,
        reason_phrase,
        evidence,
    };

    let debug = format!("{response:?}");

    assert!(debug.contains("AuthenticatedHttpResponse"));
    assert!(debug.contains("content_byte_count: 11"));
    assert!(debug.contains("reason_phrase_byte_count: 17"));
    assert!(debug.contains("status_code: 200"));
    assert!(!debug.contains("secret-body"));
    assert!(!debug.contains("TOP-SECRET-REASON"));
}
