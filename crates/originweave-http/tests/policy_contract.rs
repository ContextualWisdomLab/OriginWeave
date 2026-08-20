#![allow(clippy::expect_used)]

use std::time::Duration;

use originweave_http::{
    AlpnHttp11Policy, DEFAULT_MAX_CHUNK_COUNT, DEFAULT_MAX_CONTENT_EXPANSION_RATIO,
    DEFAULT_MAX_DECODED_CONTENT_BYTES, DEFAULT_MAX_ENCODED_CONTENT_BYTES,
    DEFAULT_MAX_HEADER_FIELD_COUNT, DEFAULT_MAX_HEADER_NAME_BYTES,
    DEFAULT_MAX_HEADER_SECTION_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES,
    DEFAULT_MAX_INTERIM_RESPONSE_COUNT, DEFAULT_MAX_REQUEST_BYTES, DEFAULT_MAX_STATUS_LINE_BYTES,
    DEFAULT_MAX_TRAILER_FIELD_COUNT, DEFAULT_MAX_TRAILER_SECTION_BYTES, HttpClientPolicy,
    HttpError, HttpPolicyLimits, IntegrityRequirement, MAX_HTTP_EXCHANGE_TIMEOUT,
};

fn build_named_policy(
    exchange_timeout: Duration,
    limits: HttpPolicyLimits,
) -> Result<HttpClientPolicy, HttpError> {
    HttpClientPolicy::from_limits(
        exchange_timeout,
        limits,
        AlpnHttp11Policy::RequireHttp11,
        IntegrityRequirement::Optional,
    )
}

#[derive(Clone, Copy)]
enum PolicyLimit {
    RequestBytes,
    StatusLineBytes,
    HeaderFieldCount,
    HeaderNameBytes,
    HeaderValueBytes,
    HeaderSectionBytes,
    InterimResponseCount,
    ChunkCount,
    TrailerFieldCount,
    TrailerSectionBytes,
    EncodedContentBytes,
    DecodedContentBytes,
}

impl PolicyLimit {
    const fn name(self) -> &'static str {
        match self {
            Self::RequestBytes => "max_request_bytes",
            Self::StatusLineBytes => "max_status_line_bytes",
            Self::HeaderFieldCount => "max_header_field_count",
            Self::HeaderNameBytes => "max_header_name_bytes",
            Self::HeaderValueBytes => "max_header_value_bytes",
            Self::HeaderSectionBytes => "max_header_section_bytes",
            Self::InterimResponseCount => "max_interim_response_count",
            Self::ChunkCount => "max_chunk_count",
            Self::TrailerFieldCount => "max_trailer_field_count",
            Self::TrailerSectionBytes => "max_trailer_section_bytes",
            Self::EncodedContentBytes => "max_encoded_content_bytes",
            Self::DecodedContentBytes => "max_decoded_content_bytes",
        }
    }

    const fn maximum(self) -> usize {
        match self {
            Self::RequestBytes => DEFAULT_MAX_REQUEST_BYTES,
            Self::StatusLineBytes => DEFAULT_MAX_STATUS_LINE_BYTES,
            Self::HeaderFieldCount => DEFAULT_MAX_HEADER_FIELD_COUNT,
            Self::HeaderNameBytes => DEFAULT_MAX_HEADER_NAME_BYTES,
            Self::HeaderValueBytes => DEFAULT_MAX_HEADER_VALUE_BYTES,
            Self::HeaderSectionBytes => DEFAULT_MAX_HEADER_SECTION_BYTES,
            Self::InterimResponseCount => DEFAULT_MAX_INTERIM_RESPONSE_COUNT,
            Self::ChunkCount => DEFAULT_MAX_CHUNK_COUNT,
            Self::TrailerFieldCount => DEFAULT_MAX_TRAILER_FIELD_COUNT,
            Self::TrailerSectionBytes => DEFAULT_MAX_TRAILER_SECTION_BYTES,
            Self::EncodedContentBytes => DEFAULT_MAX_ENCODED_CONTENT_BYTES,
            Self::DecodedContentBytes => DEFAULT_MAX_DECODED_CONTENT_BYTES,
        }
    }

    fn assign(self, limits: &mut HttpPolicyLimits, value: usize) {
        match self {
            Self::RequestBytes => limits.max_request_bytes = value,
            Self::StatusLineBytes => limits.max_status_line_bytes = value,
            Self::HeaderFieldCount => limits.max_header_field_count = value,
            Self::HeaderNameBytes => limits.max_header_name_bytes = value,
            Self::HeaderValueBytes => limits.max_header_value_bytes = value,
            Self::HeaderSectionBytes => limits.max_header_section_bytes = value,
            Self::InterimResponseCount => limits.max_interim_response_count = value,
            Self::ChunkCount => limits.max_chunk_count = value,
            Self::TrailerFieldCount => limits.max_trailer_field_count = value,
            Self::TrailerSectionBytes => limits.max_trailer_section_bytes = value,
            Self::EncodedContentBytes => limits.max_encoded_content_bytes = value,
            Self::DecodedContentBytes => limits.max_decoded_content_bytes = value,
        }
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
fn named_defaults_match_the_complete_reviewed_resource_policy() {
    let limits = HttpPolicyLimits::strict_defaults();
    assert_eq!(limits.max_request_bytes, DEFAULT_MAX_REQUEST_BYTES);
    assert_eq!(limits.max_status_line_bytes, DEFAULT_MAX_STATUS_LINE_BYTES);
    assert_eq!(limits.max_header_field_count, DEFAULT_MAX_HEADER_FIELD_COUNT);
    assert_eq!(limits.max_header_name_bytes, DEFAULT_MAX_HEADER_NAME_BYTES);
    assert_eq!(limits.max_header_value_bytes, DEFAULT_MAX_HEADER_VALUE_BYTES);
    assert_eq!(
        limits.max_header_section_bytes,
        DEFAULT_MAX_HEADER_SECTION_BYTES
    );
    assert_eq!(
        limits.max_interim_response_count,
        DEFAULT_MAX_INTERIM_RESPONSE_COUNT
    );
    assert_eq!(limits.max_chunk_count, DEFAULT_MAX_CHUNK_COUNT);
    assert_eq!(
        limits.max_trailer_field_count,
        DEFAULT_MAX_TRAILER_FIELD_COUNT
    );
    assert_eq!(
        limits.max_trailer_section_bytes,
        DEFAULT_MAX_TRAILER_SECTION_BYTES
    );
    assert_eq!(
        limits.max_encoded_content_bytes,
        DEFAULT_MAX_ENCODED_CONTENT_BYTES
    );
    assert_eq!(
        limits.max_decoded_content_bytes,
        DEFAULT_MAX_DECODED_CONTENT_BYTES
    );
    assert_eq!(
        limits.max_content_expansion_ratio,
        DEFAULT_MAX_CONTENT_EXPANSION_RATIO
    );
}

#[test]
fn callers_can_reduce_every_reviewed_limit_through_named_inputs() {
    let limits = HttpPolicyLimits {
        max_request_bytes: 1,
        max_status_line_bytes: 1,
        max_header_field_count: 1,
        max_header_name_bytes: 1,
        max_header_value_bytes: 1,
        max_header_section_bytes: 1,
        max_interim_response_count: 1,
        max_chunk_count: 1,
        max_trailer_field_count: 1,
        max_trailer_section_bytes: 1,
        max_encoded_content_bytes: 1,
        max_decoded_content_bytes: 1,
        max_content_expansion_ratio: 1,
    };
    let policy = HttpClientPolicy::from_limits(
        Duration::from_nanos(1),
        limits,
        AlpnHttp11Policy::PermitAbsentForManagedLoopback,
        IntegrityRequirement::RequireSupportedDigest,
    )
    .expect("minimum named policy");
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
fn new_preserves_every_budget_and_policy_in_order() {
    let policy = HttpClientPolicy::new(
        Duration::from_secs(7),
        11,
        12,
        13,
        14,
        15,
        16,
        3,
        17,
        4,
        18,
        19,
        20,
        5,
        AlpnHttp11Policy::PermitAbsentForManagedLoopback,
        IntegrityRequirement::RequireSupportedDigest,
    )
    .expect("distinct policy budgets");
    assert_eq!(policy.exchange_timeout(), Duration::from_secs(7));
    assert_eq!(policy.max_request_bytes(), 11);
    assert_eq!(policy.max_status_line_bytes(), 12);
    assert_eq!(policy.max_header_field_count(), 13);
    assert_eq!(policy.max_header_name_bytes(), 14);
    assert_eq!(policy.max_header_value_bytes(), 15);
    assert_eq!(policy.max_header_section_bytes(), 16);
    assert_eq!(policy.max_interim_response_count(), 3);
    assert_eq!(policy.max_chunk_count(), 17);
    assert_eq!(policy.max_trailer_field_count(), 4);
    assert_eq!(policy.max_trailer_section_bytes(), 18);
    assert_eq!(policy.max_encoded_content_bytes(), 19);
    assert_eq!(policy.max_decoded_content_bytes(), 20);
    assert_eq!(policy.max_content_expansion_ratio(), 5);
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
        assert!(matches!(
            build_named_policy(timeout, HttpPolicyLimits::strict_defaults()),
            Err(HttpError::InvalidExchangeTimeout { .. })
        ));
    }
    for ratio in [0, DEFAULT_MAX_CONTENT_EXPANSION_RATIO + 1] {
        let mut limits = HttpPolicyLimits::strict_defaults();
        limits.max_content_expansion_ratio = ratio;
        assert!(matches!(
            build_named_policy(MAX_HTTP_EXCHANGE_TIMEOUT, limits),
            Err(HttpError::InvalidExpansionRatio { .. })
        ));
    }
}

#[test]
fn each_count_and_byte_limit_rejects_zero_and_maximum_plus_one() {
    let policy_limits = [
        PolicyLimit::RequestBytes,
        PolicyLimit::StatusLineBytes,
        PolicyLimit::HeaderFieldCount,
        PolicyLimit::HeaderNameBytes,
        PolicyLimit::HeaderValueBytes,
        PolicyLimit::HeaderSectionBytes,
        PolicyLimit::InterimResponseCount,
        PolicyLimit::ChunkCount,
        PolicyLimit::TrailerFieldCount,
        PolicyLimit::TrailerSectionBytes,
        PolicyLimit::EncodedContentBytes,
        PolicyLimit::DecodedContentBytes,
    ];

    for limit in policy_limits {
        let maximum = limit.maximum();
        for value in [0, maximum + 1] {
            let mut limits = HttpPolicyLimits::strict_defaults();
            limit.assign(&mut limits, value);
            assert!(matches!(
                build_named_policy(MAX_HTTP_EXCHANGE_TIMEOUT, limits),
                Err(HttpError::InvalidPolicyLimit {
                    limit_name: observed,
                    value: observed_value,
                    maximum: observed_maximum,
                }) if observed == limit.name()
                    && observed_value == value
                    && observed_maximum == maximum
            ));
        }
    }
}
