#![allow(clippy::expect_used)]

use std::time::Duration;

use originweave_http::{
    AlpnHttp11Policy, DEFAULT_MAX_CHUNK_COUNT, DEFAULT_MAX_CONTENT_EXPANSION_RATIO,
    DEFAULT_MAX_DECODED_CONTENT_BYTES, DEFAULT_MAX_ENCODED_CONTENT_BYTES,
    DEFAULT_MAX_HEADER_FIELD_COUNT, DEFAULT_MAX_HEADER_NAME_BYTES,
    DEFAULT_MAX_HEADER_SECTION_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES,
    DEFAULT_MAX_INTERIM_RESPONSE_COUNT, DEFAULT_MAX_REQUEST_BYTES,
    DEFAULT_MAX_STATUS_LINE_BYTES, DEFAULT_MAX_TRAILER_FIELD_COUNT,
    DEFAULT_MAX_TRAILER_SECTION_BYTES, HttpClientPolicy, HttpError, IntegrityRequirement,
    MAX_HTTP_EXCHANGE_TIMEOUT,
};

#[derive(Clone, Copy)]
struct PolicyInput {
    exchange_timeout: Duration,
    max_request_bytes: usize,
    max_status_line_bytes: usize,
    max_header_field_count: usize,
    max_header_name_bytes: usize,
    max_header_value_bytes: usize,
    max_header_section_bytes: usize,
    max_interim_response_count: usize,
    max_chunk_count: usize,
    max_trailer_field_count: usize,
    max_trailer_section_bytes: usize,
    max_encoded_content_bytes: usize,
    max_decoded_content_bytes: usize,
    max_content_expansion_ratio: usize,
}

impl PolicyInput {
    const fn defaults() -> Self {
        Self {
            exchange_timeout: MAX_HTTP_EXCHANGE_TIMEOUT,
            max_request_bytes: DEFAULT_MAX_REQUEST_BYTES,
            max_status_line_bytes: DEFAULT_MAX_STATUS_LINE_BYTES,
            max_header_field_count: DEFAULT_MAX_HEADER_FIELD_COUNT,
            max_header_name_bytes: DEFAULT_MAX_HEADER_NAME_BYTES,
            max_header_value_bytes: DEFAULT_MAX_HEADER_VALUE_BYTES,
            max_header_section_bytes: DEFAULT_MAX_HEADER_SECTION_BYTES,
            max_interim_response_count: DEFAULT_MAX_INTERIM_RESPONSE_COUNT,
            max_chunk_count: DEFAULT_MAX_CHUNK_COUNT,
            max_trailer_field_count: DEFAULT_MAX_TRAILER_FIELD_COUNT,
            max_trailer_section_bytes: DEFAULT_MAX_TRAILER_SECTION_BYTES,
            max_encoded_content_bytes: DEFAULT_MAX_ENCODED_CONTENT_BYTES,
            max_decoded_content_bytes: DEFAULT_MAX_DECODED_CONTENT_BYTES,
            max_content_expansion_ratio: DEFAULT_MAX_CONTENT_EXPANSION_RATIO,
        }
    }

    fn build(self) -> Result<HttpClientPolicy, HttpError> {
        HttpClientPolicy::new(
            self.exchange_timeout,
            self.max_request_bytes,
            self.max_status_line_bytes,
            self.max_header_field_count,
            self.max_header_name_bytes,
            self.max_header_value_bytes,
            self.max_header_section_bytes,
            self.max_interim_response_count,
            self.max_chunk_count,
            self.max_trailer_field_count,
            self.max_trailer_section_bytes,
            self.max_encoded_content_bytes,
            self.max_decoded_content_bytes,
            self.max_content_expansion_ratio,
            AlpnHttp11Policy::RequireHttp11,
            IntegrityRequirement::Optional,
        )
    }
}

#[test]
fn strict_defaults_expose_the_complete_reviewed_policy() {
    let policy = HttpClientPolicy::strict_defaults();
    assert_eq!(policy.exchange_timeout(), MAX_HTTP_EXCHANGE_TIMEOUT);
    assert_eq!(policy.max_request_bytes(), DEFAULT_MAX_REQUEST_BYTES);
    assert_eq!(
        policy.max_status_line_bytes(),
        DEFAULT_MAX_STATUS_LINE_BYTES
    );
    assert_eq!(
        policy.max_header_field_count(),
        DEFAULT_MAX_HEADER_FIELD_COUNT
    );
    assert_eq!(
        policy.max_header_name_bytes(),
        DEFAULT_MAX_HEADER_NAME_BYTES
    );
    assert_eq!(
        policy.max_header_value_bytes(),
        DEFAULT_MAX_HEADER_VALUE_BYTES
    );
    assert_eq!(
        policy.max_header_section_bytes(),
        DEFAULT_MAX_HEADER_SECTION_BYTES
    );
    assert_eq!(
        policy.max_interim_response_count(),
        DEFAULT_MAX_INTERIM_RESPONSE_COUNT
    );
    assert_eq!(policy.max_chunk_count(), DEFAULT_MAX_CHUNK_COUNT);
    assert_eq!(
        policy.max_trailer_field_count(),
        DEFAULT_MAX_TRAILER_FIELD_COUNT
    );
    assert_eq!(
        policy.max_trailer_section_bytes(),
        DEFAULT_MAX_TRAILER_SECTION_BYTES
    );
    assert_eq!(
        policy.max_encoded_content_bytes(),
        DEFAULT_MAX_ENCODED_CONTENT_BYTES
    );
    assert_eq!(
        policy.max_decoded_content_bytes(),
        DEFAULT_MAX_DECODED_CONTENT_BYTES
    );
    assert_eq!(
        policy.max_content_expansion_ratio(),
        DEFAULT_MAX_CONTENT_EXPANSION_RATIO
    );
    assert_eq!(policy.alpn_policy(), AlpnHttp11Policy::RequireHttp11);
    assert_eq!(
        policy.integrity_requirement(),
        IntegrityRequirement::Optional
    );
}

#[test]
fn callers_can_reduce_every_reviewed_limit() {
    let policy = HttpClientPolicy::new(
        Duration::from_nanos(1),
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        AlpnHttp11Policy::PermitAbsentForManagedLoopback,
        IntegrityRequirement::RequireSupportedDigest,
    )
    .expect("minimum policy");
    assert_eq!(policy.exchange_timeout(), Duration::from_nanos(1));
    assert_eq!(policy.max_request_bytes(), 1);
    assert_eq!(policy.max_decoded_content_bytes(), 1);
    assert_eq!(policy.max_content_expansion_ratio(), 1);
    assert_eq!(
        policy.alpn_policy(),
        AlpnHttp11Policy::PermitAbsentForManagedLoopback
    );
    assert_eq!(
        policy.integrity_requirement(),
        IntegrityRequirement::RequireSupportedDigest
    );
}

#[test]
fn timeout_and_expansion_ratio_fail_outside_the_reviewed_range() {
    for timeout in [
        Duration::ZERO,
        MAX_HTTP_EXCHANGE_TIMEOUT + Duration::from_nanos(1),
    ] {
        let mut input = PolicyInput::defaults();
        input.exchange_timeout = timeout;
        assert!(matches!(
            input.build(),
            Err(HttpError::InvalidExchangeTimeout { .. })
        ));
    }
    for ratio in [0, DEFAULT_MAX_CONTENT_EXPANSION_RATIO + 1] {
        let mut input = PolicyInput::defaults();
        input.max_content_expansion_ratio = ratio;
        assert!(matches!(
            input.build(),
            Err(HttpError::InvalidExpansionRatio { .. })
        ));
    }
}

#[test]
fn each_count_and_byte_limit_rejects_zero_and_maximum_plus_one() {
    let invalid_pairs = [
        ("max_request_bytes", DEFAULT_MAX_REQUEST_BYTES),
        ("max_status_line_bytes", DEFAULT_MAX_STATUS_LINE_BYTES),
        ("max_header_field_count", DEFAULT_MAX_HEADER_FIELD_COUNT),
        ("max_header_name_bytes", DEFAULT_MAX_HEADER_NAME_BYTES),
        ("max_header_value_bytes", DEFAULT_MAX_HEADER_VALUE_BYTES),
        ("max_header_section_bytes", DEFAULT_MAX_HEADER_SECTION_BYTES),
        (
            "max_interim_response_count",
            DEFAULT_MAX_INTERIM_RESPONSE_COUNT,
        ),
        ("max_chunk_count", DEFAULT_MAX_CHUNK_COUNT),
        (
            "max_trailer_field_count",
            DEFAULT_MAX_TRAILER_FIELD_COUNT,
        ),
        (
            "max_trailer_section_bytes",
            DEFAULT_MAX_TRAILER_SECTION_BYTES,
        ),
        (
            "max_encoded_content_bytes",
            DEFAULT_MAX_ENCODED_CONTENT_BYTES,
        ),
        (
            "max_decoded_content_bytes",
            DEFAULT_MAX_DECODED_CONTENT_BYTES,
        ),
    ];

    for (limit_name, maximum) in invalid_pairs {
        for value in [0, maximum + 1] {
            let mut input = PolicyInput::defaults();
            match limit_name {
                "max_request_bytes" => input.max_request_bytes = value,
                "max_status_line_bytes" => input.max_status_line_bytes = value,
                "max_header_field_count" => input.max_header_field_count = value,
                "max_header_name_bytes" => input.max_header_name_bytes = value,
                "max_header_value_bytes" => input.max_header_value_bytes = value,
                "max_header_section_bytes" => input.max_header_section_bytes = value,
                "max_interim_response_count" => input.max_interim_response_count = value,
                "max_chunk_count" => input.max_chunk_count = value,
                "max_trailer_field_count" => input.max_trailer_field_count = value,
                "max_trailer_section_bytes" => input.max_trailer_section_bytes = value,
                "max_encoded_content_bytes" => input.max_encoded_content_bytes = value,
                "max_decoded_content_bytes" => input.max_decoded_content_bytes = value,
                _ => unreachable!("test table contains only known policy fields"),
            }
            assert!(matches!(
                input.build(),
                Err(HttpError::InvalidPolicyLimit {
                    limit_name: observed,
                    value: observed_value,
                    maximum: observed_maximum,
                }) if observed == limit_name
                    && observed_value == value
                    && observed_maximum == maximum
            ));
        }
    }
}
