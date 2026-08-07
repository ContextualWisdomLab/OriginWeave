use std::fmt;
use std::time::Duration;

/// A deterministic failure produced by the bounded HTTP/1.1 authority.
#[derive(Debug)]
pub enum HttpError {
    /// The total exchange timeout is zero or exceeds the reviewed maximum.
    InvalidExchangeTimeout {
        /// The rejected timeout.
        timeout: Duration,
        /// The largest accepted timeout.
        maximum_timeout: Duration,
    },
    /// One configured count or byte limit is outside the reviewed range.
    InvalidPolicyLimit {
        /// Stable non-sensitive policy field name.
        limit_name: &'static str,
        /// The rejected value.
        value: usize,
        /// The largest accepted value.
        maximum: usize,
    },
    /// The configured decoded-to-encoded expansion ratio is invalid.
    InvalidExpansionRatio {
        /// The rejected ratio.
        ratio: usize,
        /// The largest accepted ratio.
        maximum_ratio: usize,
    },
    /// The request target is not a permitted origin-form target.
    InvalidRequestTarget,
    /// A percent escape in the request target is incomplete or non-hexadecimal.
    InvalidPercentEncoding {
        /// Byte offset of the rejected percent sign.
        byte_index: usize,
    },
    /// The encoded request target exceeds the reviewed maximum.
    RequestTargetTooLarge {
        /// Encoded target byte count.
        byte_count: usize,
        /// Largest accepted encoded target byte count.
        maximum_bytes: usize,
    },
    /// A request field name is empty or contains a non-token byte.
    InvalidRequestFieldName,
    /// A request field value contains a forbidden control byte.
    InvalidRequestFieldValue,
    /// A request field name exceeds the reviewed maximum.
    RequestFieldNameTooLarge {
        /// Field-name byte count.
        byte_count: usize,
        /// Largest accepted field-name byte count.
        maximum_bytes: usize,
    },
    /// A request field value exceeds the reviewed maximum.
    RequestFieldValueTooLarge {
        /// Field-value byte count.
        byte_count: usize,
        /// Largest accepted field-value byte count.
        maximum_bytes: usize,
    },
    /// The caller attempted to supply an authority, credential, or framing field.
    ForbiddenRequestField {
        /// Lowercase field name; field values are never retained.
        field_name: String,
    },
    /// The caller supplied the same field name more than once.
    DuplicateRequestField {
        /// Lowercase duplicate field name.
        field_name: String,
    },
    /// The number of caller fields exceeds the configured limit.
    ExcessiveRequestFieldCount {
        /// Submitted field count.
        field_count: usize,
        /// Largest accepted field count.
        maximum_count: usize,
    },
    /// The serialized request exceeds the configured byte limit.
    RequestTooLarge {
        /// Computed request byte count.
        byte_count: usize,
        /// Largest accepted request byte count.
        maximum_bytes: usize,
    },
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExchangeTimeout {
                timeout,
                maximum_timeout,
            } => write!(
                formatter,
                "HTTP exchange timeout {timeout:?} must be positive and no greater than {maximum_timeout:?}"
            ),
            Self::InvalidPolicyLimit {
                limit_name,
                value,
                maximum,
            } => write!(
                formatter,
                "HTTP policy limit {limit_name}={value} must be in 1..={maximum}"
            ),
            Self::InvalidExpansionRatio {
                ratio,
                maximum_ratio,
            } => write!(
                formatter,
                "HTTP content expansion ratio {ratio} must be in 1..={maximum_ratio}"
            ),
            Self::InvalidRequestTarget => {
                formatter.write_str("HTTP request target is not valid origin-form syntax")
            }
            Self::InvalidPercentEncoding { byte_index } => write!(
                formatter,
                "HTTP request target has an invalid percent escape at byte {byte_index}"
            ),
            Self::RequestTargetTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP request target has {byte_count} encoded bytes; maximum is {maximum_bytes}"
            ),
            Self::InvalidRequestFieldName => {
                formatter.write_str("HTTP request field name is invalid")
            }
            Self::InvalidRequestFieldValue => {
                formatter.write_str("HTTP request field value contains a forbidden control byte")
            }
            Self::RequestFieldNameTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP request field name has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::RequestFieldValueTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP request field value has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::ForbiddenRequestField { field_name } => write!(
                formatter,
                "HTTP request field {field_name} is controlled by a separate authority"
            ),
            Self::DuplicateRequestField { field_name } => {
                write!(formatter, "HTTP request field {field_name} is duplicated")
            }
            Self::ExcessiveRequestFieldCount {
                field_count,
                maximum_count,
            } => write!(
                formatter,
                "HTTP request has {field_count} caller fields; maximum is {maximum_count}"
            ),
            Self::RequestTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP request has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
        }
    }
}

impl std::error::Error for HttpError {}
