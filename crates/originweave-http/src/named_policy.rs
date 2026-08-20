use std::time::Duration;

use crate::policy::{
    AlpnHttp11Policy, DEFAULT_MAX_CHUNK_COUNT, DEFAULT_MAX_CONTENT_EXPANSION_RATIO,
    DEFAULT_MAX_DECODED_CONTENT_BYTES, DEFAULT_MAX_ENCODED_CONTENT_BYTES,
    DEFAULT_MAX_HEADER_FIELD_COUNT, DEFAULT_MAX_HEADER_NAME_BYTES,
    DEFAULT_MAX_HEADER_SECTION_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES,
    DEFAULT_MAX_INTERIM_RESPONSE_COUNT, DEFAULT_MAX_REQUEST_BYTES, DEFAULT_MAX_STATUS_LINE_BYTES,
    DEFAULT_MAX_TRAILER_FIELD_COUNT, DEFAULT_MAX_TRAILER_SECTION_BYTES, HttpClientPolicy,
    IntegrityRequirement,
};
use crate::HttpError;

/// Named resource and decoding budgets for one bounded HTTP/1.1 exchange.
///
/// Prefer this type with [`HttpClientPolicy::from_limits`] when selecting non-default limits so
/// same-typed byte and count budgets cannot be transposed accidentally at a call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpPolicyLimits {
    /// Maximum serialized request bytes.
    pub max_request_bytes: usize,
    /// Maximum response status-line bytes.
    pub max_status_line_bytes: usize,
    /// Maximum caller or response field count.
    pub max_header_field_count: usize,
    /// Maximum field-name bytes.
    pub max_header_name_bytes: usize,
    /// Maximum field-value bytes.
    pub max_header_value_bytes: usize,
    /// Maximum complete response header-section bytes.
    pub max_header_section_bytes: usize,
    /// Maximum informational-response count before the final response.
    pub max_interim_response_count: usize,
    /// Maximum chunk count, including the terminating zero chunk.
    pub max_chunk_count: usize,
    /// Maximum trailer field count.
    pub max_trailer_field_count: usize,
    /// Maximum complete trailer-section bytes.
    pub max_trailer_section_bytes: usize,
    /// Maximum encoded response-content bytes.
    pub max_encoded_content_bytes: usize,
    /// Maximum decoded response-content bytes.
    pub max_decoded_content_bytes: usize,
    /// Maximum decoded-to-encoded content expansion ratio.
    pub max_content_expansion_ratio: usize,
}

impl HttpPolicyLimits {
    /// Return the reviewed default HTTP resource limits.
    #[must_use]
    pub const fn strict_defaults() -> Self {
        Self {
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
}

impl HttpClientPolicy {
    /// Validate a timeout, named resource limits, ALPN policy, and integrity requirement.
    ///
    /// This is the preferred constructor for callers that customize resource budgets. The
    /// positional [`HttpClientPolicy::new`] constructor remains available for compatibility.
    pub fn from_limits(
        exchange_timeout: Duration,
        limits: HttpPolicyLimits,
        alpn_policy: AlpnHttp11Policy,
        integrity_requirement: IntegrityRequirement,
    ) -> Result<Self, HttpError> {
        Self::new(
            exchange_timeout,
            limits.max_request_bytes,
            limits.max_status_line_bytes,
            limits.max_header_field_count,
            limits.max_header_name_bytes,
            limits.max_header_value_bytes,
            limits.max_header_section_bytes,
            limits.max_interim_response_count,
            limits.max_chunk_count,
            limits.max_trailer_field_count,
            limits.max_trailer_section_bytes,
            limits.max_encoded_content_bytes,
            limits.max_decoded_content_bytes,
            limits.max_content_expansion_ratio,
            alpn_policy,
            integrity_requirement,
        )
    }
}
