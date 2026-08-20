use std::time::Duration;

use crate::HttpError;

/// Largest accepted total HTTP exchange duration.
pub const MAX_HTTP_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(120);
/// Default largest serialized request size.
pub const DEFAULT_MAX_REQUEST_BYTES: usize = 16_384;
/// Default largest response status-line size.
pub const DEFAULT_MAX_STATUS_LINE_BYTES: usize = 8_192;
/// Default largest response or caller field count.
pub const DEFAULT_MAX_HEADER_FIELD_COUNT: usize = 128;
/// Default largest field-name size.
pub const DEFAULT_MAX_HEADER_NAME_BYTES: usize = 256;
/// Default largest field-value size.
pub const DEFAULT_MAX_HEADER_VALUE_BYTES: usize = 8_192;
/// Default largest complete response header section.
pub const DEFAULT_MAX_HEADER_SECTION_BYTES: usize = 65_536;
/// Default largest number of informational responses before the final response.
pub const DEFAULT_MAX_INTERIM_RESPONSE_COUNT: usize = 8;
/// Default largest number of chunks, including the terminating zero chunk.
pub const DEFAULT_MAX_CHUNK_COUNT: usize = 65_536;
/// Default largest trailer field count.
pub const DEFAULT_MAX_TRAILER_FIELD_COUNT: usize = 32;
/// Default largest complete trailer section.
pub const DEFAULT_MAX_TRAILER_SECTION_BYTES: usize = 16_384;
/// Default largest encoded response-content size.
pub const DEFAULT_MAX_ENCODED_CONTENT_BYTES: usize = 16 * 1024 * 1024;
/// Default largest decoded response-content size.
pub const DEFAULT_MAX_DECODED_CONTENT_BYTES: usize = 32 * 1024 * 1024;
/// Default largest decoded-to-encoded content expansion ratio.
pub const DEFAULT_MAX_CONTENT_EXPANSION_RATIO: usize = 32;
/// Largest decoded prefix inspected by the conservative MIME classifier.
pub const MAX_MIME_SNIFF_BYTES: usize = 1_445;
/// Largest accepted UTF-8 filename byte count.
pub const MAX_SAFE_FILENAME_BYTES: usize = 255;

/// Policy for binding HTTP/1.1 semantics to negotiated TLS ALPN evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlpnHttp11Policy {
    /// Require the peer to have selected the `http/1.1` ALPN identifier.
    RequireHttp11,
    /// Permit explicit ALPN absence only for a separately verified loopback peer.
    PermitAbsentForManagedLoopback,
}

/// Whether a successful response requires a supported RFC 9530 digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityRequirement {
    /// Validate supported digests when present and record explicit absence.
    Optional,
    /// Reject a response that has no supported digest value.
    RequireSupportedDigest,
}

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

/// Validated resource and protocol policy for one HTTP/1.1 exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientPolicy {
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
    alpn_policy: AlpnHttp11Policy,
    integrity_requirement: IntegrityRequirement,
}

impl HttpClientPolicy {
    /// Validate every time, count, byte, and expansion budget supplied positionally.
    ///
    /// This compatibility constructor immediately converts its positional inputs to
    /// [`HttpPolicyLimits`] and delegates to [`Self::from_limits`]. New callers that customize
    /// any resource budget should use the named constructor directly.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
        alpn_policy: AlpnHttp11Policy,
        integrity_requirement: IntegrityRequirement,
    ) -> Result<Self, HttpError> {
        Self::from_limits(
            exchange_timeout,
            HttpPolicyLimits {
                max_request_bytes,
                max_status_line_bytes,
                max_header_field_count,
                max_header_name_bytes,
                max_header_value_bytes,
                max_header_section_bytes,
                max_interim_response_count,
                max_chunk_count,
                max_trailer_field_count,
                max_trailer_section_bytes,
                max_encoded_content_bytes,
                max_decoded_content_bytes,
                max_content_expansion_ratio,
            },
            alpn_policy,
            integrity_requirement,
        )
    }

    /// Validate a timeout, named resource limits, ALPN policy, and integrity requirement.
    ///
    /// This is the canonical constructor for callers that customize resource budgets. Each
    /// same-typed limit remains bound to its semantic field throughout validation and storage.
    pub fn from_limits(
        exchange_timeout: Duration,
        limits: HttpPolicyLimits,
        alpn_policy: AlpnHttp11Policy,
        integrity_requirement: IntegrityRequirement,
    ) -> Result<Self, HttpError> {
        if exchange_timeout.is_zero() || exchange_timeout > MAX_HTTP_EXCHANGE_TIMEOUT {
            return Err(HttpError::InvalidExchangeTimeout {
                timeout: exchange_timeout,
                maximum_timeout: MAX_HTTP_EXCHANGE_TIMEOUT,
            });
        }
        validate_limit(
            "max_request_bytes",
            limits.max_request_bytes,
            DEFAULT_MAX_REQUEST_BYTES,
        )?;
        validate_limit(
            "max_status_line_bytes",
            limits.max_status_line_bytes,
            DEFAULT_MAX_STATUS_LINE_BYTES,
        )?;
        validate_limit(
            "max_header_field_count",
            limits.max_header_field_count,
            DEFAULT_MAX_HEADER_FIELD_COUNT,
        )?;
        validate_limit(
            "max_header_name_bytes",
            limits.max_header_name_bytes,
            DEFAULT_MAX_HEADER_NAME_BYTES,
        )?;
        validate_limit(
            "max_header_value_bytes",
            limits.max_header_value_bytes,
            DEFAULT_MAX_HEADER_VALUE_BYTES,
        )?;
        validate_limit(
            "max_header_section_bytes",
            limits.max_header_section_bytes,
            DEFAULT_MAX_HEADER_SECTION_BYTES,
        )?;
        validate_limit(
            "max_interim_response_count",
            limits.max_interim_response_count,
            DEFAULT_MAX_INTERIM_RESPONSE_COUNT,
        )?;
        validate_limit(
            "max_chunk_count",
            limits.max_chunk_count,
            DEFAULT_MAX_CHUNK_COUNT,
        )?;
        validate_limit(
            "max_trailer_field_count",
            limits.max_trailer_field_count,
            DEFAULT_MAX_TRAILER_FIELD_COUNT,
        )?;
        validate_limit(
            "max_trailer_section_bytes",
            limits.max_trailer_section_bytes,
            DEFAULT_MAX_TRAILER_SECTION_BYTES,
        )?;
        validate_limit(
            "max_encoded_content_bytes",
            limits.max_encoded_content_bytes,
            DEFAULT_MAX_ENCODED_CONTENT_BYTES,
        )?;
        validate_limit(
            "max_decoded_content_bytes",
            limits.max_decoded_content_bytes,
            DEFAULT_MAX_DECODED_CONTENT_BYTES,
        )?;
        if limits.max_content_expansion_ratio == 0
            || limits.max_content_expansion_ratio > DEFAULT_MAX_CONTENT_EXPANSION_RATIO
        {
            return Err(HttpError::InvalidExpansionRatio {
                ratio: limits.max_content_expansion_ratio,
                maximum_ratio: DEFAULT_MAX_CONTENT_EXPANSION_RATIO,
            });
        }
        Ok(Self {
            exchange_timeout,
            max_request_bytes: limits.max_request_bytes,
            max_status_line_bytes: limits.max_status_line_bytes,
            max_header_field_count: limits.max_header_field_count,
            max_header_name_bytes: limits.max_header_name_bytes,
            max_header_value_bytes: limits.max_header_value_bytes,
            max_header_section_bytes: limits.max_header_section_bytes,
            max_interim_response_count: limits.max_interim_response_count,
            max_chunk_count: limits.max_chunk_count,
            max_trailer_field_count: limits.max_trailer_field_count,
            max_trailer_section_bytes: limits.max_trailer_section_bytes,
            max_encoded_content_bytes: limits.max_encoded_content_bytes,
            max_decoded_content_bytes: limits.max_decoded_content_bytes,
            max_content_expansion_ratio: limits.max_content_expansion_ratio,
            alpn_policy,
            integrity_requirement,
        })
    }

    /// Return the reviewed default HTTP policy.
    #[must_use]
    pub const fn strict_defaults() -> Self {
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
            alpn_policy: AlpnHttp11Policy::RequireHttp11,
            integrity_requirement: IntegrityRequirement::Optional,
        }
    }

    /// Return the total monotonic exchange timeout.
    #[must_use]
    pub const fn exchange_timeout(&self) -> Duration {
        self.exchange_timeout
    }

    /// Return the maximum serialized request bytes.
    #[must_use]
    pub const fn max_request_bytes(&self) -> usize {
        self.max_request_bytes
    }

    /// Return the maximum response status-line bytes.
    #[must_use]
    pub const fn max_status_line_bytes(&self) -> usize {
        self.max_status_line_bytes
    }

    /// Return the maximum caller or response field count.
    #[must_use]
    pub const fn max_header_field_count(&self) -> usize {
        self.max_header_field_count
    }

    /// Return the maximum field-name bytes.
    #[must_use]
    pub const fn max_header_name_bytes(&self) -> usize {
        self.max_header_name_bytes
    }

    /// Return the maximum field-value bytes.
    #[must_use]
    pub const fn max_header_value_bytes(&self) -> usize {
        self.max_header_value_bytes
    }

    /// Return the maximum response header-section bytes.
    #[must_use]
    pub const fn max_header_section_bytes(&self) -> usize {
        self.max_header_section_bytes
    }

    /// Return the maximum informational-response count.
    #[must_use]
    pub const fn max_interim_response_count(&self) -> usize {
        self.max_interim_response_count
    }

    /// Return the maximum chunk count.
    #[must_use]
    pub const fn max_chunk_count(&self) -> usize {
        self.max_chunk_count
    }

    /// Return the maximum trailer field count.
    #[must_use]
    pub const fn max_trailer_field_count(&self) -> usize {
        self.max_trailer_field_count
    }

    /// Return the maximum trailer-section bytes.
    #[must_use]
    pub const fn max_trailer_section_bytes(&self) -> usize {
        self.max_trailer_section_bytes
    }

    /// Return the maximum encoded content bytes.
    #[must_use]
    pub const fn max_encoded_content_bytes(&self) -> usize {
        self.max_encoded_content_bytes
    }

    /// Return the maximum decoded content bytes.
    #[must_use]
    pub const fn max_decoded_content_bytes(&self) -> usize {
        self.max_decoded_content_bytes
    }

    /// Return the maximum decoded-to-encoded expansion ratio.
    #[must_use]
    pub const fn max_content_expansion_ratio(&self) -> usize {
        self.max_content_expansion_ratio
    }

    /// Return the HTTP/1.1 ALPN policy.
    #[must_use]
    pub const fn alpn_policy(&self) -> AlpnHttp11Policy {
        self.alpn_policy
    }

    /// Return the response integrity requirement.
    #[must_use]
    pub const fn integrity_requirement(&self) -> IntegrityRequirement {
        self.integrity_requirement
    }
}

fn validate_limit(limit_name: &'static str, value: usize, maximum: usize) -> Result<(), HttpError> {
    if value == 0 || value > maximum {
        return Err(HttpError::InvalidPolicyLimit {
            limit_name,
            value,
            maximum,
        });
    }
    Ok(())
}
