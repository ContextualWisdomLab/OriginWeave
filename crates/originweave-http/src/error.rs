use std::fmt;
use std::io;
use std::time::Duration;

use originweave_core::Origin;

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
    /// The HTTP target origin differs from the authenticated TLS origin.
    OriginAuthorityMismatch {
        /// Origin bound to the HTTP target.
        http_origin: Origin,
        /// Origin authenticated by TLS.
        tls_origin: Origin,
    },
    /// TLS ALPN evidence does not authorize HTTP/1.1 semantics.
    UnexpectedAlpn,
    /// The response contains a malformed or unsupported status line.
    InvalidResponseStatusLine,
    /// The response uses an HTTP version outside the first HTTP/1.1 slice.
    UnsupportedHttpVersion,
    /// A status line exceeds the configured byte limit.
    StatusLineTooLarge {
        /// Observed status-line byte count.
        byte_count: usize,
        /// Largest accepted status-line byte count.
        maximum_bytes: usize,
    },
    /// A response line uses bare CR or LF instead of CRLF.
    InvalidResponseLineEnding,
    /// A response header section exceeds the configured byte limit.
    HeaderSectionTooLarge {
        /// Observed or minimum possible section byte count.
        byte_count: usize,
        /// Largest accepted section byte count.
        maximum_bytes: usize,
    },
    /// The response contains more fields than the configured limit.
    ExcessiveResponseFieldCount {
        /// Observed response field count.
        field_count: usize,
        /// Largest accepted response field count.
        maximum_count: usize,
    },
    /// A response field name is empty or contains a non-token byte.
    InvalidResponseFieldName,
    /// A response field value contains a forbidden control byte.
    InvalidResponseFieldValue,
    /// A response field name exceeds the configured byte limit.
    ResponseFieldNameTooLarge {
        /// Field-name byte count.
        byte_count: usize,
        /// Largest accepted field-name byte count.
        maximum_bytes: usize,
    },
    /// A response field value exceeds the configured byte limit.
    ResponseFieldValueTooLarge {
        /// Field-value byte count.
        byte_count: usize,
        /// Largest accepted field-value byte count.
        maximum_bytes: usize,
    },
    /// A response attempts obsolete folded field syntax.
    ObsoleteFieldFolding,
    /// The peer sent too many informational responses.
    ExcessiveInterimResponses {
        /// Observed informational response count.
        response_count: usize,
        /// Largest accepted informational response count.
        maximum_count: usize,
    },
    /// The peer requested an HTTP protocol upgrade.
    SwitchingProtocolsUnsupported,
    /// Transfer-Encoding and Content-Length appear in the same response.
    TransferEncodingWithContentLength,
    /// The response uses a transfer coding outside the strict chunked profile.
    UnsupportedTransferCoding,
    /// A Content-Length field is malformed or outside the integer domain.
    InvalidContentLength,
    /// Multiple Content-Length values do not all identify the same length.
    ConflictingContentLength,
    /// A declared or accumulated encoded body exceeds the configured limit.
    EncodedContentTooLarge {
        /// Observed or declared encoded byte count.
        byte_count: u64,
        /// Largest accepted encoded byte count.
        maximum_bytes: usize,
    },
    /// A chunk-size line or chunk boundary is malformed.
    MalformedChunkedBody,
    /// A chunk-size line exceeds the configured local maximum.
    ChunkLineTooLarge {
        /// Observed chunk-line byte count.
        byte_count: usize,
        /// Largest accepted chunk-line byte count.
        maximum_bytes: usize,
    },
    /// The response contains more chunks than the configured limit.
    ExcessiveChunkCount {
        /// Observed chunk count.
        chunk_count: usize,
        /// Largest accepted chunk count.
        maximum_count: usize,
    },
    /// A trailer section is malformed or contains a forbidden field.
    InvalidTrailerSection,
    /// The trailer contains more fields than the configured limit.
    ExcessiveTrailerFieldCount {
        /// Observed trailer field count.
        field_count: usize,
        /// Largest accepted trailer field count.
        maximum_count: usize,
    },
    /// A trailer section exceeds the configured byte limit.
    TrailerSectionTooLarge {
        /// Observed or minimum possible trailer bytes.
        byte_count: usize,
        /// Largest accepted trailer-section bytes.
        maximum_bytes: usize,
    },
    /// The peer ended the stream before a complete response was available.
    IncompleteResponse,
    /// The response uses a content coding outside the first slice.
    UnsupportedContentCoding,
    /// Bounded content decoding failed.
    ContentDecodingFailed {
        /// Decoder failure without content bytes.
        source: io::Error,
    },
    /// Decoded content exceeds the configured byte limit.
    DecodedContentTooLarge {
        /// Observed decoded byte count.
        byte_count: usize,
        /// Largest accepted decoded byte count.
        maximum_bytes: usize,
    },
    /// Decoded content exceeds the configured expansion ratio.
    ContentExpansionRatioExceeded {
        /// Observed decoded byte count.
        decoded_bytes: usize,
        /// Encoded source byte count.
        encoded_bytes: usize,
        /// Largest accepted ratio.
        maximum_ratio: usize,
    },
    /// An RFC 9530 digest field is syntactically invalid.
    InvalidDigestField,
    /// A supported digest does not match the applicable bytes.
    DigestMismatch {
        /// Stable supported algorithm key.
        algorithm: &'static str,
    },
    /// Policy requires a supported digest but none is available.
    SupportedDigestRequired,
    /// Supplied Content-Type syntax is invalid.
    InvalidMimeType,
    /// Content-Disposition metadata is malformed or unsafe.
    InvalidContentDisposition,
    /// Redirect metadata is duplicated, malformed, or unsafe.
    InvalidRedirectMetadata,
    /// The total HTTP exchange deadline elapsed.
    HttpExchangeTimedOut {
        /// Configured total exchange timeout.
        timeout: Duration,
    },
    /// A blocking HTTP exchange I/O operation failed.
    HttpExchangeIoFailed {
        /// Operating-system or TLS stream failure.
        source: io::Error,
    },
    /// Original socket timeouts could not be restored.
    TimeoutRestorationFailed {
        /// Operating-system failure.
        source: io::Error,
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
            Self::OriginAuthorityMismatch {
                http_origin,
                tls_origin,
            } => write!(
                formatter,
                "HTTP origin {http_origin} does not match authenticated TLS origin {tls_origin}"
            ),
            Self::UnexpectedAlpn => {
                formatter.write_str("TLS ALPN evidence does not authorize HTTP/1.1")
            }
            Self::InvalidResponseStatusLine => {
                formatter.write_str("HTTP response status line is invalid")
            }
            Self::UnsupportedHttpVersion => {
                formatter.write_str("HTTP response version is not HTTP/1.1")
            }
            Self::StatusLineTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP status line has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::InvalidResponseLineEnding => {
                formatter.write_str("HTTP response uses an invalid line ending")
            }
            Self::HeaderSectionTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP response header section has at least {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::ExcessiveResponseFieldCount {
                field_count,
                maximum_count,
            } => write!(
                formatter,
                "HTTP response has {field_count} fields; maximum is {maximum_count}"
            ),
            Self::InvalidResponseFieldName => {
                formatter.write_str("HTTP response field name is invalid")
            }
            Self::InvalidResponseFieldValue => {
                formatter.write_str("HTTP response field value contains a forbidden control byte")
            }
            Self::ResponseFieldNameTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP response field name has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::ResponseFieldValueTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP response field value has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::ObsoleteFieldFolding => {
                formatter.write_str("HTTP response uses obsolete folded field syntax")
            }
            Self::ExcessiveInterimResponses {
                response_count,
                maximum_count,
            } => write!(
                formatter,
                "HTTP peer sent {response_count} informational responses; maximum is {maximum_count}"
            ),
            Self::SwitchingProtocolsUnsupported => {
                formatter.write_str("HTTP protocol upgrade is outside this authority")
            }
            Self::TransferEncodingWithContentLength => formatter.write_str(
                "HTTP response contains both Transfer-Encoding and Content-Length",
            ),
            Self::UnsupportedTransferCoding => {
                formatter.write_str("HTTP response transfer coding is unsupported")
            }
            Self::InvalidContentLength => {
                formatter.write_str("HTTP response Content-Length is invalid")
            }
            Self::ConflictingContentLength => {
                formatter.write_str("HTTP response Content-Length values conflict")
            }
            Self::EncodedContentTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP encoded content has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::MalformedChunkedBody => {
                formatter.write_str("HTTP chunked content is malformed")
            }
            Self::ChunkLineTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP chunk-size line has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::ExcessiveChunkCount {
                chunk_count,
                maximum_count,
            } => write!(
                formatter,
                "HTTP response has {chunk_count} chunks; maximum is {maximum_count}"
            ),
            Self::InvalidTrailerSection => {
                formatter.write_str("HTTP trailer section is invalid")
            }
            Self::ExcessiveTrailerFieldCount {
                field_count,
                maximum_count,
            } => write!(
                formatter,
                "HTTP trailer has {field_count} fields; maximum is {maximum_count}"
            ),
            Self::TrailerSectionTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP trailer section has at least {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::IncompleteResponse => {
                formatter.write_str("HTTP response ended before message completion")
            }
            Self::UnsupportedContentCoding => {
                formatter.write_str("HTTP content coding is unsupported")
            }
            Self::ContentDecodingFailed { .. } => {
                formatter.write_str("HTTP content decoding failed")
            }
            Self::DecodedContentTooLarge {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "HTTP decoded content has {byte_count} bytes; maximum is {maximum_bytes}"
            ),
            Self::ContentExpansionRatioExceeded {
                decoded_bytes,
                encoded_bytes,
                maximum_ratio,
            } => write!(
                formatter,
                "HTTP content expanded from {encoded_bytes} to {decoded_bytes} bytes; maximum ratio is {maximum_ratio}"
            ),
            Self::InvalidDigestField => {
                formatter.write_str("HTTP digest field is invalid")
            }
            Self::DigestMismatch { algorithm } => {
                write!(formatter, "HTTP {algorithm} digest does not match content")
            }
            Self::SupportedDigestRequired => {
                formatter.write_str("HTTP policy requires a supported digest")
            }
            Self::InvalidMimeType => {
                formatter.write_str("HTTP Content-Type metadata is invalid")
            }
            Self::InvalidContentDisposition => {
                formatter.write_str("HTTP Content-Disposition metadata is invalid or unsafe")
            }
            Self::InvalidRedirectMetadata => {
                formatter.write_str("HTTP redirect metadata is invalid or ambiguous")
            }
            Self::HttpExchangeTimedOut { timeout } => {
                write!(formatter, "HTTP exchange exceeded total timeout {timeout:?}")
            }
            Self::HttpExchangeIoFailed { .. } => {
                formatter.write_str("HTTP exchange I/O failed")
            }
            Self::TimeoutRestorationFailed { .. } => {
                formatter.write_str("HTTP socket timeout restoration failed")
            }
        }
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ContentDecodingFailed { source }
            | Self::HttpExchangeIoFailed { source }
            | Self::TimeoutRestorationFailed { source } => Some(source),
            _other => None,
        }
    }
}
