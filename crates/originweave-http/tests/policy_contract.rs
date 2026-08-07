#![allow(clippy::expect_used)]

use std::time::Duration;

use originweave_http::{
    AlpnHttp11Policy, HttpClientPolicy, DEFAULT_MAX_ENCODED_CONTENT_BYTES,
    DEFAULT_MAX_HEADER_FIELD_COUNT, DEFAULT_MAX_HEADER_NAME_BYTES,
    DEFAULT_MAX_HEADER_SECTION_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES, DEFAULT_MAX_REQUEST_BYTES,
    DEFAULT_MAX_STATUS_LINE_BYTES, MAX_HTTP_EXCHANGE_TIMEOUT,
};

#[test]
fn strict_defaults_use_the_reviewed_product_maxima() {
    let policy = HttpClientPolicy::strict_defaults();
    assert_eq!(policy.exchange_timeout(), MAX_HTTP_EXCHANGE_TIMEOUT);
    assert_eq!(policy.max_request_bytes(), DEFAULT_MAX_REQUEST_BYTES);
    assert_eq!(policy.max_status_line_bytes(), DEFAULT_MAX_STATUS_LINE_BYTES);
    assert_eq!(policy.max_header_field_count(), DEFAULT_MAX_HEADER_FIELD_COUNT);
    assert_eq!(policy.max_header_name_bytes(), DEFAULT_MAX_HEADER_NAME_BYTES);
    assert_eq!(policy.max_header_value_bytes(), DEFAULT_MAX_HEADER_VALUE_BYTES);
    assert_eq!(policy.max_header_section_bytes(), DEFAULT_MAX_HEADER_SECTION_BYTES);
    assert_eq!(
        policy.max_encoded_content_bytes(),
        DEFAULT_MAX_ENCODED_CONTENT_BYTES
    );
    assert_eq!(policy.alpn_policy(), AlpnHttp11Policy::RequireHttp11);
}

#[test]
fn every_configurable_budget_can_reduce_but_not_expand_the_reviewed_maximum() {
    let policy = HttpClientPolicy::new(
        Duration::from_nanos(1),
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        AlpnHttp11Policy::PermitAbsentForManagedLoopback,
    )
    .expect("minimum policy");
    assert_eq!(policy.exchange_timeout(), Duration::from_nanos(1));
    assert_eq!(
        policy.alpn_policy(),
        AlpnHttp11Policy::PermitAbsentForManagedLoopback
    );

    let invalid_cases = [
        HttpClientPolicy::new(
            Duration::ZERO,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            AlpnHttp11Policy::RequireHttp11,
        ),
        HttpClientPolicy::new(
            MAX_HTTP_EXCHANGE_TIMEOUT + Duration::from_nanos(1),
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            AlpnHttp11Policy::RequireHttp11,
        ),
        HttpClientPolicy::new(
            Duration::from_secs(1),
            DEFAULT_MAX_REQUEST_BYTES + 1,
            1,
            1,
            1,
            1,
            1,
            1,
            AlpnHttp11Policy::RequireHttp11,
        ),
        HttpClientPolicy::new(
            Duration::from_secs(1),
            1,
            DEFAULT_MAX_STATUS_LINE_BYTES + 1,
            1,
            1,
            1,
            1,
            1,
            AlpnHttp11Policy::RequireHttp11,
        ),
        HttpClientPolicy::new(
            Duration::from_secs(1),
            1,
            1,
            DEFAULT_MAX_HEADER_FIELD_COUNT + 1,
            1,
            1,
            1,
            1,
            AlpnHttp11Policy::RequireHttp11,
        ),
        HttpClientPolicy::new(
            Duration::from_secs(1),
            1,
            1,
            1,
            DEFAULT_MAX_HEADER_NAME_BYTES + 1,
            1,
            1,
            1,
            AlpnHttp11Policy::RequireHttp11,
        ),
        HttpClientPolicy::new(
            Duration::from_secs(1),
            1,
            1,
            1,
            1,
            DEFAULT_MAX_HEADER_VALUE_BYTES + 1,
            1,
            1,
            AlpnHttp11Policy::RequireHttp11,
        ),
        HttpClientPolicy::new(
            Duration::from_secs(1),
            1,
            1,
            1,
            1,
            1,
            DEFAULT_MAX_HEADER_SECTION_BYTES + 1,
            1,
            AlpnHttp11Policy::RequireHttp11,
        ),
        HttpClientPolicy::new(
            Duration::from_secs(1),
            1,
            1,
            1,
            1,
            1,
            1,
            DEFAULT_MAX_ENCODED_CONTENT_BYTES + 1,
            AlpnHttp11Policy::RequireHttp11,
        ),
    ];
    assert!(invalid_cases.into_iter().all(|result| result.is_err()));
}
