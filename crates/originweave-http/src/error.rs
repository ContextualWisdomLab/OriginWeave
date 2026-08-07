use std::fmt;
use std::io;
use std::time::Duration;

/// A deterministic failure from the bounded HTTP/1.1 authority.
#[derive(Debug)]
pub enum HttpError {
    /// A configured timeout was zero or exceeded the reviewed maximum.
    InvalidExchangeTimeout {
        /// Supplied timeout.
        timeout: Duration,
        /// Maximum permitted timeout.
        maximum_timeout: Duration,
    },
    /// A byte or count budget was zero or exceeded its reviewed maximum.
    InvalidPolicyLimit {
        /// Stable policy-field name.
        limit_name: &'static str,
        /// Supplied limit.
        value: usize,
        /// Maximum permitted limit.
        maximum: usize,
    },
    /// The request target was not a strict origin-form target.
    InvalidRequestTarget,
    /// A caller request field had invalid name or value syntax.
    InvalidRequestField,
    /// A caller attempted to supply authority, credential, or framing metadata.
    ForbiddenRequestField,
    /// The same caller request field name appeared more than once.
    DuplicateRequestField,
    /// The deterministic serialized request exceeded policy.
    RequestTooLarge {
        /// Serialized request bytes.
        byte_count: usize,
        /// Maximum permitted bytes.
        maximum_bytes: usize,
    },
    /// The request target origin differed from the authenticated TLS origin.
    OriginAuthorityMismatch,
    /// The authenticated TLS peer evidence was internally inconsistent.
    TlsPeerEvidenceMismatch,
    /// ALPN did not authorize the fixed HTTP/1.1 adapter.
    UnexpectedAlpn,
    /// The HTTP exchange exceeded its total monotonic deadline.
    ExchangeTimedOut {
        /// Configured total deadline.
        timeout: Duration,
    },
    /// A bounded stream operation failed.
    Io {
        /// Stable operation name without content or destination data.
        operation: &'static str,
        /// Underlying I/O failure.
        source: io::Error,
    },
    /// The peer emitted a malformed or unsupported HTTP status line.
    InvalidStatusLine,
    /// The status line exceeded the configured maximum.
    StatusLineTooLarge {
        /// Observed status-line bytes.
        byte_count: usize,
        /// Maximum permitted bytes.
        maximum_bytes: usize,
    },
    /// A response field line was malformed.
    InvalidResponseField,
    /// The response contained too many fields.
    TooManyResponseFields {
        /// Observed response field count.
        field_count: usize,
        /// Maximum permitted field count.
        maximum_count: usize,
    },
    /// A response field name exceeded policy.
    ResponseFieldNameTooLarge {
        /// Observed field-name bytes.
        byte_count: usize,
        /// Maximum permitted bytes.
        maximum_bytes: usize,
    },
    /// A response field value exceeded policy.
    ResponseFieldValueTooLarge {
        /// Observed field-value bytes.
        byte_count: usize,
        /// Maximum permitted bytes.
        maximum_bytes: usize,
    },
    /// The response head exceeded the configured section budget.
    HeaderSectionTooLarge {
        /// Observed response-head bytes.
        byte_count: usize,
        /// Maximum permitted bytes.
        maximum_bytes: usize,
    },
    /// `Transfer-Encoding` is outside the fixed-length vertical slice.
    UnsupportedTransferEncoding,
    /// Multiple content lengths disagreed or contained invalid decimal syntax.
    InvalidContentLength,
    /// The declared content length exceeded policy.
    ContentTooLarge {
        /// Declared or observed content bytes.
        byte_count: usize,
        /// Maximum permitted bytes.
        maximum_bytes: usize,
    },
    /// A body-bearing response omitted the fixed-length framing required by this slice.
    UnsupportedBodyFraming,
    /// The TLS stream ended before the declared response content was complete.
    IncompleteResponse,
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExchangeTimeout {
                timeout,
                maximum_timeout,
            } => write!(
                formatter,
                "HTTP exchange timeout {timeout:?} is outside 1ns..={maximum_timeout:?}"
            ),
            Self::InvalidPolicyLimit {
                limit_name,
                value,
                maximum,
            } => write!(
                formatter,
                "HTTP policy limit {limit_name}={value} is outside 1..={maximum}"
            ),
            Self::InvalidRequestTarget => formatter.write_str("invalid HTTP request target"),
            Self::InvalidRequestField => formatter.write_str("invalid HTTP request field"),
            Self::ForbiddenRequestField => formatter.write_str("forbidden HTTP request field"),
            Self::DuplicateRequestField => formatter.write_str("duplicate HTTP request field"),
            Self::RequestTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP request uses {byte_count} bytes, above maximum {maximum_bytes}"
            ),
            Self::OriginAuthorityMismatch => {
                formatter.write_str("HTTP target origin does not match authenticated TLS origin")
            }
            Self::TlsPeerEvidenceMismatch => {
                formatter.write_str("authenticated TLS peer evidence is inconsistent")
            }
            Self::UnexpectedAlpn => formatter.write_str("TLS ALPN does not authorize HTTP/1.1"),
            Self::ExchangeTimedOut { timeout } => {
                write!(formatter, "HTTP exchange exceeded deadline {timeout:?}")
            }
            Self::Io { operation, .. } => write!(formatter, "HTTP {operation} I/O failed"),
            Self::InvalidStatusLine => formatter.write_str("invalid HTTP/1.1 status line"),
            Self::StatusLineTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP status line uses {byte_count} bytes, above maximum {maximum_bytes}"
            ),
            Self::InvalidResponseField => formatter.write_str("invalid HTTP response field"),
            Self::TooManyResponseFields {
                field_count,
                maximum_count,
            } => write!(
                formatter,
                "HTTP response has {field_count} fields, above maximum {maximum_count}"
            ),
            Self::ResponseFieldNameTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP response field name uses {byte_count} bytes, above maximum {maximum_bytes}"
            ),
            Self::ResponseFieldValueTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP response field value uses {byte_count} bytes, above maximum {maximum_bytes}"
            ),
            Self::HeaderSectionTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP response head uses {byte_count} bytes, above maximum {maximum_bytes}"
            ),
            Self::UnsupportedTransferEncoding => {
                formatter.write_str("HTTP transfer coding is not supported by this slice")
            }
            Self::InvalidContentLength => formatter.write_str("invalid HTTP Content-Length"),
            Self::ContentTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP content uses {byte_count} bytes, above maximum {maximum_bytes}"
            ),
            Self::UnsupportedBodyFraming => formatter.write_str(
                "HTTP response requires framing outside the fixed-length vertical slice",
            ),
            Self::IncompleteResponse => formatter.write_str("HTTP response ended before completion"),
        }
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
