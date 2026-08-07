#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_http::{HttpMethod, HttpRequestTarget, RequestField};

#[test]
fn methods_expose_only_get_and_head_tokens() {
    assert_eq!(HttpMethod::Get.as_bytes(), b"GET");
    assert_eq!(HttpMethod::Head.as_bytes(), b"HEAD");
    assert!(!HttpMethod::Get.suppresses_response_content());
    assert!(HttpMethod::Head.suppresses_response_content());
}

#[test]
fn request_target_is_origin_bound_and_percent_encodes_unicode() {
    let origin = Origin::parse("https://example.com").expect("HTTPS origin");
    let target = HttpRequestTarget::parse(origin.clone(), "/reports/한글?q=1")
        .expect("origin-form target");
    assert_eq!(target.origin(), &origin);
    assert_eq!(target.path_and_query(), "/reports/%ED%95%9C%EA%B8%80?q=1");
    assert!(target.query_present());
    assert_eq!(target.path_prefix(), "/reports/%ED%95%9C%EA%B8%80");
    assert!(target.target_hash().starts_with("sha256:"));
    assert_eq!(target.target_hash().len(), 71);
}

#[test]
fn request_target_rejects_ambiguous_or_unsafe_forms() {
    let origin = Origin::parse("https://example.com").expect("HTTPS origin");
    for value in [
        "",
        "relative",
        "https://example.com/",
        "/fragment#secret",
        "/bad%",
        "/bad%2",
        "/bad%zz",
        "/space here",
        "/back\\slash",
        "/line\nfeed",
    ] {
        assert!(HttpRequestTarget::parse(origin.clone(), value).is_err(), "{value:?}");
    }
}

#[test]
fn request_fields_reject_authority_credentials_and_framing() {
    for blocked in [
        "host",
        "connection",
        "proxy-connection",
        "keep-alive",
        "transfer-encoding",
        "content-length",
        "trailer",
        "te",
        "upgrade",
        "authorization",
        "proxy-authorization",
        "cookie",
    ] {
        assert!(RequestField::new(blocked, b"x").is_err(), "{blocked}");
    }

    let field = RequestField::new("accept-language", b"ko-KR, en;q=0.8")
        .expect("safe request field");
    assert_eq!(field.name(), "accept-language");
    assert_eq!(field.value(), b"ko-KR, en;q=0.8");
    assert!(RequestField::new("Bad Name", b"x").is_err());
    assert!(RequestField::new("x-test", b"line\nfeed").is_err());
}
