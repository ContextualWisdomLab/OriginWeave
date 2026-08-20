use std::fmt::{Display, Formatter};
use std::io;
use std::time::Duration;

/// A failure while validating a bounded HTTP request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpRequestError {
    /// The origin was not an HTTPS origin.
    InsecureOrigin,
    /// The request target was not a bounded origin-form target.
    InvalidRequestTarget,
}

impl Display for HttpRequestError {
    #[inline(never)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InsecureOrigin => "HTTP request requires an HTTPS origin",
            Self::InvalidRequestTarget => "HTTP request target is invalid",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for HttpRequestError {}

/// A policy validation failure before network I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpPolicyError {
    /// The exchange deadline was zero or exceeded the hard ceiling.
    InvalidExchangeTimeout {
        /// The rejected deadline.
        timeout: Duration,
        /// The hard deadline ceiling.
        maximum: Duration,
    },
    /// The body ceiling was zero or exceeded the hard ceiling.
    InvalidBodyLimit {
        /// The rejected body ceiling.
        limit: usize,
        /// The hard body ceiling.
        maximum: usize,
    },
}

impl Display for HttpPolicyError {
    #[inline(never)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidExchangeTimeout { .. } => {
                formatter.write_str("invalid HTTP exchange timeout")
            }
            Self::InvalidBodyLimit { .. } => formatter.write_str("invalid HTTP body limit"),
        }
    }
}

impl std::error::Error for HttpPolicyError {}

/// A fail-closed HTTP request, framing, or exchange failure.
#[derive(Debug)]
pub enum HttpError {
    /// The request was not authorized by the authenticated TLS origin.
    RequestAuthorityMismatch,
    /// The TLS ALPN value was not `http/1.1`.
    UnexpectedAlpn,
    /// Absent ALPN was not permitted by the explicit policy.
    AbsentAlpnNotPermitted,
    /// The request or response status line exceeded its bound.
    HeaderLineLimitExceeded,
    /// The complete response header section exceeded its bound.
    HeaderSectionLimitExceeded,
    /// Too many response fields were received.
    HeaderFieldLimitExceeded,
    /// A status line was not strict HTTP/1.1 syntax.
    MalformedStatusLine,
    /// A field line was not strict field-name and field-value syntax.
    MalformedHeader,
    /// A response declared mutually ambiguous framing.
    FramingAmbiguous,
    /// A content length was duplicated or invalid.
    DuplicateContentLength,
    /// A content length was not bounded decimal syntax.
    InvalidContentLength,
    /// A transfer coding was not the explicitly supported chunked coding.
    UnsupportedTransferCoding,
    /// A content coding was not identity.
    UnsupportedContentCoding,
    /// A chunk line or chunk terminator was malformed.
    MalformedChunk,
    /// The response contained too many chunks.
    ChunkLimitExceeded,
    /// The response contained too many trailers.
    TrailerLimitExceeded,
    /// Redirect metadata is retained for a later authority-bound adapter.
    RedirectNotSupported,
    /// The response trailer section exceeded its bound.
    TrailerSectionLimitExceeded,
    /// The response body exceeded its decoded-byte budget.
    BodyLimitExceeded,
    /// The peer closed or truncated a response before its declared boundary.
    IncompleteResponse,
    /// The total exchange deadline elapsed.
    ExchangeTimedOut,
    /// A bounded socket operation failed.
    Io {
        /// The operation being performed.
        operation: &'static str,
        /// The underlying I/O failure.
        source: io::Error,
    },
    /// A socket timeout could not be configured.
    TimeoutConfiguration {
        /// The operation whose timeout was being configured.
        operation: &'static str,
        /// The underlying configuration failure.
        source: io::Error,
    },
}

impl HttpError {
    #[inline(always)]
    pub(crate) fn io(operation: &'static str, source: io::Error) -> Self {
        if matches!(
            source.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ) {
            Self::ExchangeTimedOut
        } else {
            Self::Io { operation, source }
        }
    }
}

#[inline(always)]
pub(crate) fn io_result<T>(result: io::Result<T>, operation: &'static str) -> Result<T, HttpError> {
    result.map_err(|source| HttpError::io(operation, source))
}

#[inline(always)]
pub(crate) fn timeout_result<T>(result: io::Result<T>, op: &'static str) -> Result<T, HttpError> {
    result.map_err(|source| HttpError::TimeoutConfiguration {
        operation: op,
        source,
    })
}

impl Display for HttpError {
    #[inline(never)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::RequestAuthorityMismatch => "HTTP request origin does not match TLS evidence",
            Self::UnexpectedAlpn => "TLS negotiated an unexpected application protocol",
            Self::AbsentAlpnNotPermitted => "TLS did not negotiate an application protocol",
            Self::HeaderLineLimitExceeded => "HTTP line exceeds its byte budget",
            Self::HeaderSectionLimitExceeded => "HTTP header section exceeds its byte budget",
            Self::HeaderFieldLimitExceeded => "HTTP header field count exceeds its budget",
            Self::MalformedStatusLine => "HTTP status line is malformed",
            Self::MalformedHeader => "HTTP header field is malformed",
            Self::FramingAmbiguous => "HTTP response framing is ambiguous",
            Self::DuplicateContentLength => "HTTP response contains duplicate content length",
            Self::InvalidContentLength => "HTTP response content length is invalid",
            Self::UnsupportedTransferCoding => "HTTP transfer coding is unsupported",
            Self::UnsupportedContentCoding => "HTTP content coding is unsupported",
            Self::MalformedChunk => "HTTP chunk framing is malformed",
            Self::ChunkLimitExceeded => "HTTP chunk count exceeds its budget",
            Self::TrailerLimitExceeded => "HTTP trailer count exceeds its budget",
            Self::TrailerSectionLimitExceeded => "HTTP trailer section exceeds its budget",
            Self::RedirectNotSupported => "HTTP redirects require a separate authority-bound plan",
            Self::BodyLimitExceeded => "HTTP response body exceeds its budget",
            Self::IncompleteResponse => "HTTP response ended before its boundary",
            Self::ExchangeTimedOut => "HTTP exchange deadline elapsed",
            Self::Io { .. } => "HTTP I/O failed",
            Self::TimeoutConfiguration { .. } => "HTTP timeout configuration failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::TimeoutConfiguration { source, .. } => Some(source),
            _ => None,
        }
    }
}
