use std::time::Duration;

use crate::HttpError;

/// Maximum total monotonic duration for one HTTP exchange.
pub const MAX_HTTP_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(120);
/// Maximum serialized request bytes supported by the first HTTP slice.
pub const DEFAULT_MAX_REQUEST_BYTES: usize = 16_384;
/// Maximum HTTP/1.1 status-line bytes.
pub const DEFAULT_MAX_STATUS_LINE_BYTES: usize = 8_192;
/// Maximum response field count.
pub const DEFAULT_MAX_HEADER_FIELD_COUNT: usize = 128;
/// Maximum bytes in one response field name.
pub const DEFAULT_MAX_HEADER_NAME_BYTES: usize = 256;
/// Maximum bytes in one response field value.
pub const DEFAULT_MAX_HEADER_VALUE_BYTES: usize = 8_192;
/// Maximum response head bytes through the terminating empty line.
pub const DEFAULT_MAX_HEADER_SECTION_BYTES: usize = 65_536;
/// Maximum encoded response content bytes materialized by this slice.
pub const DEFAULT_MAX_ENCODED_CONTENT_BYTES: usize = 16 * 1024 * 1024;

/// Policy for authorizing HTTP/1.1 from TLS ALPN evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlpnHttp11Policy {
    /// Require the authenticated peer to negotiate exactly `http/1.1`.
    RequireHttp11,
    /// Permit explicit ALPN absence only when the authenticated TCP peer is loopback.
    PermitAbsentForManagedLoopback,
}

/// Validated byte, count, and deadline budgets for one HTTP exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientPolicy {
    exchange_timeout: Duration,
    max_request_bytes: usize,
    max_status_line_bytes: usize,
    max_header_field_count: usize,
    max_header_name_bytes: usize,
    max_header_value_bytes: usize,
    max_header_section_bytes: usize,
    max_encoded_content_bytes: usize,
    alpn_policy: AlpnHttp11Policy,
}

impl HttpClientPolicy {
    /// Validate a caller-reduced policy against the reviewed product maxima.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        exchange_timeout: Duration,
        max_request_bytes: usize,
        max_status_line_bytes: usize,
        max_header_field_count: usize,
        max_header_name_bytes: usize,
        max_header_value_bytes: usize,
        max_header_section_bytes: usize,
        max_encoded_content_bytes: usize,
        alpn_policy: AlpnHttp11Policy,
    ) -> Result<Self, HttpError> {
        if exchange_timeout.is_zero() || exchange_timeout > MAX_HTTP_EXCHANGE_TIMEOUT {
            return Err(HttpError::InvalidExchangeTimeout {
                timeout: exchange_timeout,
                maximum_timeout: MAX_HTTP_EXCHANGE_TIMEOUT,
            });
        }
        validate_limit(
            "max_request_bytes",
            max_request_bytes,
            DEFAULT_MAX_REQUEST_BYTES,
        )?;
        validate_limit(
            "max_status_line_bytes",
            max_status_line_bytes,
            DEFAULT_MAX_STATUS_LINE_BYTES,
        )?;
        validate_limit(
            "max_header_field_count",
            max_header_field_count,
            DEFAULT_MAX_HEADER_FIELD_COUNT,
        )?;
        validate_limit(
            "max_header_name_bytes",
            max_header_name_bytes,
            DEFAULT_MAX_HEADER_NAME_BYTES,
        )?;
        validate_limit(
            "max_header_value_bytes",
            max_header_value_bytes,
            DEFAULT_MAX_HEADER_VALUE_BYTES,
        )?;
        validate_limit(
            "max_header_section_bytes",
            max_header_section_bytes,
            DEFAULT_MAX_HEADER_SECTION_BYTES,
        )?;
        validate_limit(
            "max_encoded_content_bytes",
            max_encoded_content_bytes,
            DEFAULT_MAX_ENCODED_CONTENT_BYTES,
        )?;
        Ok(Self {
            exchange_timeout,
            max_request_bytes,
            max_status_line_bytes,
            max_header_field_count,
            max_header_name_bytes,
            max_header_value_bytes,
            max_header_section_bytes,
            max_encoded_content_bytes,
            alpn_policy,
        })
    }

    /// Return the conservative reviewed product maxima.
    #[must_use]
    pub fn strict_defaults() -> Self {
        Self {
            exchange_timeout: MAX_HTTP_EXCHANGE_TIMEOUT,
            max_request_bytes: DEFAULT_MAX_REQUEST_BYTES,
            max_status_line_bytes: DEFAULT_MAX_STATUS_LINE_BYTES,
            max_header_field_count: DEFAULT_MAX_HEADER_FIELD_COUNT,
            max_header_name_bytes: DEFAULT_MAX_HEADER_NAME_BYTES,
            max_header_value_bytes: DEFAULT_MAX_HEADER_VALUE_BYTES,
            max_header_section_bytes: DEFAULT_MAX_HEADER_SECTION_BYTES,
            max_encoded_content_bytes: DEFAULT_MAX_ENCODED_CONTENT_BYTES,
            alpn_policy: AlpnHttp11Policy::RequireHttp11,
        }
    }

    /// Return the total monotonic exchange timeout.
    #[must_use]
    pub const fn exchange_timeout(&self) -> Duration {
        self.exchange_timeout
    }

    /// Return the request byte budget.
    #[must_use]
    pub const fn max_request_bytes(&self) -> usize {
        self.max_request_bytes
    }

    /// Return the status-line byte budget.
    #[must_use]
    pub const fn max_status_line_bytes(&self) -> usize {
        self.max_status_line_bytes
    }

    /// Return the response field count budget.
    #[must_use]
    pub const fn max_header_field_count(&self) -> usize {
        self.max_header_field_count
    }

    /// Return the response field-name byte budget.
    #[must_use]
    pub const fn max_header_name_bytes(&self) -> usize {
        self.max_header_name_bytes
    }

    /// Return the response field-value byte budget.
    #[must_use]
    pub const fn max_header_value_bytes(&self) -> usize {
        self.max_header_value_bytes
    }

    /// Return the response head byte budget.
    #[must_use]
    pub const fn max_header_section_bytes(&self) -> usize {
        self.max_header_section_bytes
    }

    /// Return the encoded response content byte budget.
    #[must_use]
    pub const fn max_encoded_content_bytes(&self) -> usize {
        self.max_encoded_content_bytes
    }

    /// Return the TLS ALPN admission policy.
    #[must_use]
    pub const fn alpn_policy(&self) -> AlpnHttp11Policy {
        self.alpn_policy
    }
}

fn validate_limit(limit_name: &'static str, value: usize, maximum: usize) -> Result<(), HttpError> {
    if value == 0 || value > maximum {
        Err(HttpError::InvalidPolicyLimit {
            limit_name,
            value,
            maximum,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn each_limit_rejects_zero_and_preserves_its_name() {
        let error = validate_limit("sample_limit", 0, 3).expect_err("zero limit");
        assert!(matches!(
            error,
            HttpError::InvalidPolicyLimit {
                limit_name: "sample_limit",
                value: 0,
                maximum: 3
            }
        ));
        validate_limit("sample_limit", 3, 3).expect("exact maximum");
    }
}
