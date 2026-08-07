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

    fn assign(self, input: &mut PolicyInput, value: usize) {
        match self {
            Self::RequestBytes => input.max_request_bytes = value,
            Self::StatusLineBytes => input.max_status_line_bytes = value,
            Self::HeaderFieldCount => input.max_header_field_count = value,
            Self::HeaderNameBytes => input.max_header_name_bytes = value,
            Self::HeaderValueBytes => input.max_header_value_bytes = value,
            Self::HeaderSectionBytes => input.max_header_section_bytes = value,
            Self::InterimResponseCount => input.max_interim_response_count = value,
            Self::ChunkCount => input.max_chunk_count = value,
            Self::TrailerFieldCount => input.max_trailer_field_count = value,
            Self::TrailerSectionBytes => input.max_trailer_section_bytes = value,
            Self::EncodedContentBytes => input.max_encoded_content_bytes = value,
            Self::DecodedContentBytes => input.max_decoded_content_bytes = value,
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
    let limits = [
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

    for limit in limits {
        let maximum = limit.maximum();
        for value in [0, maximum + 1] {
            let mut input = PolicyInput::defaults();
            limit.assign(&mut input, value);
            assert!(matches!(
                input.build(),
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
