#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_http::{HttpError, HttpRequestTarget, RequestField};

#[test]
fn request_targets_preserve_authority_and_encode_non_ascii_bytes() {
    let origin = Origin::parse("https://example.com:8443").expect("origin");
    let target = HttpRequestTarget::parse(origin.clone(), "/한글?q=값").expect("target");
    assert_eq!(target.origin(), &origin);
    assert_eq!(target.path_and_query(), "/%ED%95%9C%EA%B8%80?q=%EA%B0%92");
    assert!(target.query_present());
    assert_eq!(target.path_prefix(), "/%ED%95%9C%EA%B8%80");
    assert!(target.target_hash().starts_with("sha256:"));
    assert_eq!(target.target_hash().len(), 71);
}

#[test]
fn request_targets_preserve_valid_percent_escape_spelling() {
    let origin = Origin::parse("https://example.com").expect("origin");
    let lower = HttpRequestTarget::parse(origin.clone(), "/a%2fb").expect("lower escape");
    let upper = HttpRequestTarget::parse(origin, "/a%2Fb").expect("upper escape");
    assert_eq!(lower.path_and_query(), "/a%2fb");
    assert_eq!(upper.path_and_query(), "/a%2Fb");
    assert_ne!(lower.target_hash(), upper.target_hash());
}

#[test]
fn request_targets_accept_the_complete_origin_form_ascii_grammar() {
    let origin = Origin::parse("https://example.com").expect("origin");
    let input = "/AZaz09-._~!$&'()*+,;=:@/segment?x=/?:@!$&'()*+,;=-._~";
    let target = HttpRequestTarget::parse(origin, input).expect("valid origin-form target");
    assert_eq!(target.path_and_query(), input);
}

#[test]
fn request_target_hash_is_bound_to_the_origin() {
    let first = HttpRequestTarget::parse(
        Origin::parse("https://example.com").expect("first origin"),
        "/same",
    )
    .expect("first target");
    let second = HttpRequestTarget::parse(
        Origin::parse("https://example.net").expect("second origin"),
        "/same",
    )
    .expect("second target");
    assert_ne!(first.target_hash(), second.target_hash());
}

#[test]
fn invalid_origin_form_targets_fail_closed() {
    let origin = Origin::parse("https://example.com").expect("origin");
    for invalid in [
        "",
        "*",
        "https://example.com/path",
        "relative",
        "/fragment#value",
        "/back\\slash",
        "/white space",
        "/tab\tvalue",
        "/line\nvalue",
        "/carriage\rvalue",
        "/nul\0value",
        "/raw<angle",
        "/raw>angle",
        "/raw\"quote",
        "/raw{brace",
        "/raw}brace",
        "/raw|pipe",
        "/raw^caret",
        "/raw`tick",
        "/raw[bracket",
        "/raw]bracket",
        "/?query=<invalid>",
    ] {
        assert!(matches!(
            HttpRequestTarget::parse(origin.clone(), invalid),
            Err(HttpError::InvalidRequestTarget)
        ));
    }
}

#[test]
fn invalid_percent_escapes_report_the_exact_percent_offset() {
    let origin = Origin::parse("https://example.com").expect("origin");
    for (input, byte_index) in [("/%", 1), ("/%2", 1), ("/a%XZ", 2), ("/%0G", 1)] {
        assert!(matches!(
            HttpRequestTarget::parse(origin.clone(), input),
            Err(HttpError::InvalidPercentEncoding {
                byte_index: observed
            }) if observed == byte_index
        ));
    }
}

#[test]
fn request_target_size_is_bounded_after_encoding() {
    let origin = Origin::parse("https://example.com").expect("origin");
    let exact = format!("/{}", "a".repeat(8_191));
    assert!(HttpRequestTarget::parse(origin.clone(), &exact).is_ok());

    let excessive = format!("/{}", "a".repeat(8_192));
    assert!(matches!(
        HttpRequestTarget::parse(origin, &excessive),
        Err(HttpError::RequestTargetTooLarge {
            byte_count: 8_193,
            maximum_bytes: 8_192,
        })
    ));
}

#[test]
fn request_field_names_use_the_complete_token_alphabet() {
    let punctuation = "!#$%&'*+-.^_`|~";
    let field =
        RequestField::new(&format!("AZaz09{punctuation}"), b"value").expect("valid token field");
    assert_eq!(field.name(), "azaz09!#$%&'*+-.^_`|~");
    assert_eq!(field.value_byte_count(), 5);
}

#[test]
fn request_field_values_accept_internal_htab_visible_ascii_and_obs_text() {
    let value = [b'A', b'\t', b' ', b'~', 0x80, 0xff, b'Z'];
    let field = RequestField::new("X-Binary-Metadata", &value).expect("field");
    assert_eq!(field.name(), "x-binary-metadata");
    assert_eq!(field.value_byte_count(), value.len());
}

#[test]
fn request_field_values_reject_surrounding_optional_whitespace() {
    for invalid_value in [
        b" leading".as_slice(),
        b"\tleading",
        b"trailing ",
        b"trailing\t",
        b" \t surrounded \t",
    ] {
        assert!(matches!(
            RequestField::new("x-test", invalid_value),
            Err(HttpError::InvalidRequestFieldValue)
        ));
    }
}

#[test]
fn invalid_field_names_and_values_fail_closed() {
    for invalid_name in ["", "white space", "colon:name", "ümlaut", "line\nname"] {
        assert!(matches!(
            RequestField::new(invalid_name, b"value"),
            Err(HttpError::InvalidRequestFieldName)
        ));
    }
    for invalid_value in [vec![0_u8], vec![b'\n'], vec![b'\r'], vec![0x1f], vec![0x7f]] {
        assert!(matches!(
            RequestField::new("x-test", &invalid_value),
            Err(HttpError::InvalidRequestFieldValue)
        ));
    }
}

#[test]
fn field_name_and_value_sizes_are_bounded() {
    let exact_name = "a".repeat(256);
    assert!(RequestField::new(&exact_name, b"value").is_ok());

    let long_name = "a".repeat(257);
    assert!(matches!(
        RequestField::new(&long_name, b"value"),
        Err(HttpError::RequestFieldNameTooLarge {
            byte_count: 257,
            maximum_bytes: 256,
        })
    ));

    let exact_value = vec![b'a'; 8_192];
    assert!(RequestField::new("x-test", &exact_value).is_ok());

    let long_value = vec![b'a'; 8_193];
    assert!(matches!(
        RequestField::new("x-test", &long_value),
        Err(HttpError::RequestFieldValueTooLarge {
            byte_count: 8_193,
            maximum_bytes: 8_192,
        })
    ));
}

#[test]
fn authority_credentials_and_framing_are_caller_forbidden() {
    for forbidden in [
        "Host",
        "Connection",
        "Proxy-Connection",
        "Keep-Alive",
        "Transfer-Encoding",
        "Content-Length",
        "Trailer",
        "TE",
        "Upgrade",
        "Authorization",
        "Proxy-Authorization",
        "Cookie",
    ] {
        assert!(matches!(
            RequestField::new(forbidden, b"secret-value"),
            Err(HttpError::ForbiddenRequestField { field_name })
                if field_name == forbidden.to_ascii_lowercase()
        ));
    }
}

#[test]
fn request_field_debug_never_exposes_the_value() {
    let field = RequestField::new("x-api-label", b"do-not-log-this").expect("field");
    let debug = format!("{field:?}");
    assert!(debug.contains("x-api-label"));
    assert!(debug.contains("15"));
    assert!(!debug.contains("do-not-log-this"));
}
