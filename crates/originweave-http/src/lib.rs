//! Bounded HTTP/1.1 exchange authority for an already authenticated TLS stream.
//!
//! This crate is intentionally limited to one request/response exchange. It
//! does not resolve names, open sockets, select proxies, weaken TLS, follow
//! redirects, persist content, or grant browser authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod error;
mod exchange;
mod request;
mod response;

pub use error::{HttpError, HttpPolicyError, HttpRequestError};
pub use exchange::HttpExchange;
pub use request::{HttpMethod, HttpRequest, HttpRequestTarget};
pub use response::{
    HttpExchangePolicy, HttpHeader, HttpResponse, MAX_CHUNK_COUNT, MAX_HEADER_FIELDS,
    MAX_HEADER_NAME_BYTES, MAX_HEADER_SECTION_BYTES, MAX_HEADER_VALUE_BYTES, MAX_HTTP_BODY_BYTES,
    MAX_TRAILER_FIELDS, MAX_TRAILER_SECTION_BYTES,
};

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::expect_used)]
mod tests {
    use std::io::Cursor;
    use std::time::Duration;

    use super::*;
    use originweave_core::Origin;

    #[test]
    fn request_contract_rejects_cross_origin_and_emits_only_owned_fields() {
        let origin = Origin::parse("https://example.test").expect("valid origin");
        let target = HttpRequestTarget::parse("/agent?step=1").expect("valid target");
        let request = HttpRequest::new(HttpMethod::Get, origin.clone(), target.clone())
            .expect("valid request");

        assert_eq!(request.method(), HttpMethod::Get);
        assert_eq!(request.origin(), &origin);
        assert_eq!(request.target(), &target);
        assert_eq!(target.as_str(), "/agent?step=1");
        assert_eq!(
            request.serialize(),
            b"GET /agent?step=1 HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n"
        );
        let head = HttpRequest::new(
            HttpMethod::Head,
            origin.clone(),
            HttpRequestTarget::parse("/").expect("head target"),
        )
        .expect("head request");
        assert!(head.serialize().starts_with(b"HEAD / HTTP/1.1\r\n"));
        assert!(matches!(
            HttpRequest::new(
                HttpMethod::Get,
                Origin::parse("http://127.0.0.1").expect("parse loopback origin"),
                target,
            ),
            Err(HttpRequestError::InsecureOrigin)
        ));

        let policy = HttpExchangePolicy::new(Duration::from_secs(1), 32).expect("policy");
        assert_eq!(policy.exchange_timeout(), Duration::from_secs(1));
        assert_eq!(policy.max_body_bytes(), 32);
        assert!(policy.permit_absent_alpn().allow_absent_alpn());
        let response = HttpResponse::parse(
            &mut Cursor::new(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n"),
            HttpMethod::Get,
            &policy,
        )
        .expect("response parser");
        assert_eq!(response.headers()[0].value(), "0");

        assert!(
            !HttpRequestError::InvalidRequestTarget
                .to_string()
                .is_empty()
        );
        assert!(
            !HttpPolicyError::InvalidBodyLimit {
                limit: 0,
                maximum: 1
            }
            .to_string()
            .is_empty()
        );
    }
}
