use std::{
    error::Error,
    fmt,
    io::{self, Read, Write},
    net::TcpStream,
    thread,
    time::{Duration, Instant},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use originweave_core::VerifiedWebDriverBiDiSocketPeer;
use sha1::{Digest, Sha1};

use crate::{WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionEvidence};

const WEBSOCKET_CLIENT_KEY_LENGTH: usize = 24;
const RFC6455_WEBSOCKET_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const MAX_WEBSOCKET_OPENING_RESPONSE_BYTES: usize = 16 * 1024;

/// Maximum wall-clock budget accepted for writing one bounded WebSocket opening request.
///
/// This is an OriginWeave resource-safety ceiling, not an RFC 6455 protocol limit. The request is
/// already bounded before this budget is applied. Callers may choose any smaller nonzero deadline.
pub const MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum wall-clock budget accepted for reading one bounded WebSocket opening response.
///
/// This is an OriginWeave resource-safety ceiling, not an RFC 6455 protocol limit. Callers may
/// choose any smaller nonzero deadline.
pub const MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum bytes admitted while reading one WebSocket HTTP opening response.
///
/// The response is consumed only through its terminating `CRLF CRLF`; WebSocket frames are not
/// read or interpreted by this boundary.
pub const MAX_WEBSOCKET_OPENING_RESPONSE_SIZE: usize = MAX_WEBSOCKET_OPENING_RESPONSE_BYTES;

fn is_base64_data_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/')
}

fn is_canonical_16_byte_base64(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == WEBSOCKET_CLIENT_KEY_LENGTH
        && bytes[..22].iter().copied().all(is_base64_data_byte)
        && matches!(bytes[21], b'A' | b'Q' | b'g' | b'w')
        && bytes[22] == b'='
        && bytes[23] == b'='
}

/// Deterministic failures while preparing one WebDriver BiDi RFC 6455 opening request.
#[derive(Debug, Eq, PartialEq)]
pub enum WebDriverBiDiWebSocketHandshakeError {
    /// The supplied client key was not the canonical base64 representation of exactly 16 bytes.
    InvalidClientKey,
    /// The verified WebDriver BiDi target requires TLS before a WebSocket opening request is sent.
    TlsRequired,
}

impl fmt::Display for WebDriverBiDiWebSocketHandshakeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidClientKey => formatter.write_str(
                "WebDriver BiDi WebSocket client key is not canonical base64 for exactly 16 bytes",
            ),
            Self::TlsRequired => formatter.write_str(
                "WebDriver BiDi WebSocket target requires authenticated TLS before the opening request",
            ),
        }
    }
}

impl Error for WebDriverBiDiWebSocketHandshakeError {}

/// Canonical RFC 6455 client key for one WebDriver BiDi opening handshake.
///
/// RFC 6455 requires `Sec-WebSocket-Key` to be a nonce of 16 bytes encoded with base64. This type
/// validates only the canonical wire representation, including zero padding bits. It does not
/// generate entropy: callers remain responsible for supplying a fresh, unpredictable 16-byte nonce
/// for each connection attempt.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketClientKey(String);

impl WebDriverBiDiWebSocketClientKey {
    /// Admit one canonical base64 client key representing exactly 16 bytes.
    pub fn new(value: &str) -> Result<Self, WebDriverBiDiWebSocketHandshakeError> {
        if !is_canonical_16_byte_base64(value) {
            return Err(WebDriverBiDiWebSocketHandshakeError::InvalidClientKey);
        }
        Ok(Self(value.to_owned()))
    }

    /// Borrow the exact canonical value for `Sec-WebSocket-Key` serialization.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Inert RFC 6455 opening request bound to one already-verified plain BiDi TCP connection.
///
/// The plan consumes the verified TCP connection so the opening request cannot be detached from the
/// socket peer/session evidence that authorized its exact loopback destination. It serializes only
/// the fixed WebSocket version-13 request required for the admitted `/session/<session-id>` resource
/// and retains the exact client key required to validate a later `Sec-WebSocket-Accept` response.
/// Secure `wss` targets fail closed here and require a separate authenticated TLS transport boundary
/// before any WebSocket bytes may be written.
///
/// Construction performs no socket write, TLS operation, response parsing, `Sec-WebSocket-Accept`
/// validation, WebSocket framing, Chromium/ChromeDriver process authentication, browser action, or
/// Agent-authority grant.
#[derive(Debug)]
pub struct WebDriverBiDiWebSocketHandshakePlan {
    connection: WebDriverBiDiTcpConnection,
    client_key: WebDriverBiDiWebSocketClientKey,
    request: Vec<u8>,
}

impl WebDriverBiDiWebSocketHandshakePlan {
    /// Bind one canonical opening request to an already-verified plain BiDi TCP connection.
    pub fn new(
        connection: WebDriverBiDiTcpConnection,
        client_key: WebDriverBiDiWebSocketClientKey,
    ) -> Result<Self, WebDriverBiDiWebSocketHandshakeError> {
        if connection.verified_peer().requires_tls() {
            return Err(WebDriverBiDiWebSocketHandshakeError::TlsRequired);
        }

        let peer = connection.verified_peer();
        let request = format!(
            "GET /session/{} HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {}\r\nSec-WebSocket-Version: 13\r\n\r\n",
            peer.session_id(),
            peer.socket_addr(),
            client_key.as_str(),
        )
        .into_bytes();

        Ok(Self {
            connection,
            client_key,
            request,
        })
    }

    /// Borrow the exact serialized RFC 6455 opening-request bytes.
    #[must_use]
    pub fn request_bytes(&self) -> &[u8] {
        &self.request
    }

    /// Borrow the exact client key that a later server-handshake validator must correlate.
    #[must_use]
    pub const fn client_key(&self) -> &WebDriverBiDiWebSocketClientKey {
        &self.client_key
    }

    /// Borrow the exact peer/session evidence already verified before request construction.
    #[must_use]
    pub const fn verified_peer(&self) -> &VerifiedWebDriverBiDiSocketPeer {
        self.connection.verified_peer()
    }

    /// Write the complete bounded opening request on the exact verified stream within one deadline.
    ///
    /// The plan is consumed. Zero and over-ceiling deadlines fail closed. The writer retries only an
    /// interrupted system call; it never reconnects, resolves a name, selects a proxy, changes the
    /// destination, or retries after any other I/O failure. A partial write that cannot finish before
    /// the same monotonic deadline is an error and yields no successful handoff. Before success, the
    /// operation-local socket write timeout is cleared so the next separately reviewed protocol stage
    /// cannot inherit stale timeout authority. Success preserves the live stream, exact transport
    /// evidence, and client key for a separately reviewed server handshake validator. It does not
    /// read or validate the server response and therefore does not establish WebSocket protocol state
    /// or browser/Agent authority.
    pub fn write_opening_request(
        self,
        write_timeout: Duration,
    ) -> Result<WebDriverBiDiWebSocketOpeningRequestSent, WebDriverBiDiWebSocketOpeningWriteError>
    {
        if write_timeout.is_zero() || write_timeout > MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT {
            return Err(
                WebDriverBiDiWebSocketOpeningWriteError::InvalidWriteTimeout {
                    write_timeout,
                    maximum_timeout: MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT,
                },
            );
        }

        let Self {
            connection,
            client_key,
            request,
        } = self;
        let (mut stream, transport_evidence) = connection.into_parts();
        let mut now = Instant::now;
        let request_byte_count =
            write_request_with_clock(&mut stream, &request, write_timeout, &mut now)?;

        Ok(WebDriverBiDiWebSocketOpeningRequestSent {
            stream,
            transport_evidence,
            client_key,
            request_byte_count,
            write_timeout,
        })
    }
}

/// A live verified stream after the complete client opening request has been written.
///
/// This state proves only that the exact bounded RFC 6455 client request reached the operating
/// system's verified TCP stream before the configured deadline and that this operation's socket write
/// timeout was cleared before handoff. It deliberately does not claim that the peer returned `101
/// Switching Protocols`, that `Sec-WebSocket-Accept` is valid, that a WebSocket is established, or
/// that the peer is the expected Chromium/ChromeDriver process. Those remain separate fail-closed
/// boundaries.
pub struct WebDriverBiDiWebSocketOpeningRequestSent {
    pub(crate) stream: TcpStream,
    transport_evidence: WebDriverBiDiTcpConnectionEvidence,
    client_key: WebDriverBiDiWebSocketClientKey,
    request_byte_count: usize,
    write_timeout: Duration,
}

impl fmt::Debug for WebDriverBiDiWebSocketOpeningRequestSent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiWebSocketOpeningRequestSent")
            .field("stream_local_addr", &self.stream.local_addr().ok())
            .field("transport_evidence", &self.transport_evidence)
            .field(
                "client_key",
                &"<retained for Sec-WebSocket-Accept validation>",
            )
            .field("request_byte_count", &self.request_byte_count)
            .field("write_timeout", &self.write_timeout)
            .finish()
    }
}

impl WebDriverBiDiWebSocketOpeningRequestSent {
    /// Borrow the exact verified transport evidence retained with this live stream.
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        &self.transport_evidence
    }

    /// Borrow the exact client key required to validate the later server accept value.
    #[must_use]
    pub const fn client_key(&self) -> &WebDriverBiDiWebSocketClientKey {
        &self.client_key
    }

    /// Return the exact number of opening-request bytes written before success was emitted.
    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.request_byte_count
    }

    /// Return the total write deadline configured for this opening request.
    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.write_timeout
    }

    /// Read and validate the bounded RFC 6455 server opening response on this exact stream.
    ///
    /// Success proves only an HTTP/1.1 `101 Switching Protocols` response with the required
    /// `Upgrade`, `Connection`, and client-key-correlated `Sec-WebSocket-Accept` headers. The
    /// response body, WebSocket frames, browser process identity, TLS, and browser/Agent authority
    /// remain separate boundaries.
    pub fn read_opening_response(
        self,
        response_timeout: Duration,
    ) -> Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketHandshakeResponseError>
    {
        if response_timeout.is_zero() || response_timeout > MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT {
            return Err(
                WebDriverBiDiWebSocketHandshakeResponseError::InvalidResponseTimeout {
                    response_timeout,
                    maximum_timeout: MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT,
                },
            );
        }

        let Self {
            mut stream,
            transport_evidence,
            client_key,
            request_byte_count,
            write_timeout,
        } = self;
        let mut now = Instant::now;
        let (response_status, response_byte_count) =
            read_opening_response_with_clock(&mut stream, &client_key, response_timeout, &mut now)?;

        Ok(WebDriverBiDiWebSocketEstablished {
            stream,
            transport_evidence,
            client_key,
            response_status,
            response_byte_count,
            response_timeout,
            request_byte_count,
            write_timeout,
        })
    }
}

/// A live verified stream after both RFC 6455 opening messages were validated.
///
/// This state does not implement WebSocket framing or grant browser, page, policy, or Agent
/// authority. It retains the exact transport evidence and client key so later protocol stages can
/// remain correlated with the verified peer and opening handshake.
pub struct WebDriverBiDiWebSocketEstablished {
    pub(crate) stream: TcpStream,
    transport_evidence: WebDriverBiDiTcpConnectionEvidence,
    client_key: WebDriverBiDiWebSocketClientKey,
    response_status: u16,
    response_byte_count: usize,
    response_timeout: Duration,
    request_byte_count: usize,
    write_timeout: Duration,
}

impl fmt::Debug for WebDriverBiDiWebSocketEstablished {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiWebSocketEstablished")
            .field("stream_local_addr", &self.stream.local_addr().ok())
            .field("transport_evidence", &self.transport_evidence)
            .field(
                "client_key",
                &"<retained for WebSocket session correlation>",
            )
            .field("response_status", &self.response_status)
            .field("response_byte_count", &self.response_byte_count)
            .field("response_timeout", &self.response_timeout)
            .field("request_byte_count", &self.request_byte_count)
            .field("write_timeout", &self.write_timeout)
            .finish()
    }
}

impl WebDriverBiDiWebSocketEstablished {
    /// Borrow the exact verified transport evidence retained with this live stream.
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        &self.transport_evidence
    }

    /// Borrow the exact client key correlated with the validated server accept value.
    #[must_use]
    pub const fn client_key(&self) -> &WebDriverBiDiWebSocketClientKey {
        &self.client_key
    }

    /// Return the validated HTTP status code, currently always `101` on success.
    #[must_use]
    pub const fn response_status(&self) -> u16 {
        self.response_status
    }

    /// Return the number of HTTP opening-response bytes consumed through its header terminator.
    #[must_use]
    pub const fn response_byte_count(&self) -> usize {
        self.response_byte_count
    }

    /// Return the total response deadline configured for this opening response.
    #[must_use]
    pub const fn response_timeout(&self) -> Duration {
        self.response_timeout
    }

    /// Return the number of request bytes written before the response was read.
    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.request_byte_count
    }

    /// Return the total write deadline configured for the preceding opening request.
    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.write_timeout
    }
}

/// Fail-closed errors while reading one bounded WebDriver BiDi WebSocket opening response.
#[derive(Debug)]
pub enum WebDriverBiDiWebSocketHandshakeResponseError {
    /// The requested total response deadline was zero or above the reviewed resource ceiling.
    InvalidResponseTimeout {
        /// Rejected caller-supplied deadline.
        response_timeout: Duration,
        /// Maximum reviewed deadline accepted by this boundary.
        maximum_timeout: Duration,
    },
    /// The monotonic total response deadline elapsed before validation completed.
    ResponseDeadlineExceeded {
        /// Number of response bytes consumed before the deadline elapsed.
        bytes_read: usize,
    },
    /// The response exceeded the reviewed header-size ceiling before its terminator was found.
    ResponseTooLarge {
        /// Number of response bytes consumed before rejection.
        bytes_read: usize,
        /// Maximum response bytes admitted by this boundary.
        maximum_bytes: usize,
    },
    /// Applying the operation-local nonblocking read mode failed.
    ResponseReadModeConfigurationFailed {
        /// Number of response bytes consumed before configuration failed.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A bounded socket read timed out before the opening response was complete.
    ResponseReadTimedOut {
        /// Number of response bytes consumed before the timed-out operation.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A non-recoverable socket read failed before the opening response was complete.
    ResponseReadFailed {
        /// Number of response bytes consumed before the failure.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// The peer closed the stream before sending a complete HTTP header block.
    ResponseEndedBeforeHeaders {
        /// Number of response bytes consumed before the peer closed the stream.
        bytes_read: usize,
    },
    /// The HTTP response was not a valid, required WebSocket opening response.
    MalformedResponse {
        /// Stable, non-secret reason for the rejected response shape.
        reason: &'static str,
    },
    /// The response's `Sec-WebSocket-Accept` did not correlate with the sent client key.
    AcceptMismatch,
    /// Restoring blocking mode failed after validation.
    ReadModeCleanupFailed {
        /// Underlying operating-system error.
        source: io::Error,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketHandshakeResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidResponseTimeout { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening response timeout is outside the reviewed bound",
            ),
            Self::ResponseDeadlineExceeded { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening response exceeded its monotonic deadline",
            ),
            Self::ResponseTooLarge { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening response exceeded its bounded header size",
            ),
            Self::ResponseReadModeConfigurationFailed { .. } => formatter.write_str(
                "failed to configure bounded nonblocking WebDriver BiDi WebSocket response reads",
            ),
            Self::ResponseReadTimedOut { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening response timed out before completion",
            ),
            Self::ResponseReadFailed { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening response read failed before completion",
            ),
            Self::ResponseEndedBeforeHeaders { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket peer ended the stream before completing response headers",
            ),
            Self::MalformedResponse { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening response was malformed or missing a required header",
            ),
            Self::AcceptMismatch => formatter.write_str(
                "WebDriver BiDi WebSocket opening response accept value did not match the client key",
            ),
            Self::ReadModeCleanupFailed { .. } => formatter.write_str(
                "failed to restore blocking WebDriver BiDi WebSocket response reads before handoff",
            ),
        }
    }
}

impl Error for WebDriverBiDiWebSocketHandshakeResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ResponseReadModeConfigurationFailed { source, .. }
            | Self::ResponseReadTimedOut { source, .. }
            | Self::ResponseReadFailed { source, .. }
            | Self::ReadModeCleanupFailed { source } => Some(source),
            Self::InvalidResponseTimeout { .. }
            | Self::ResponseDeadlineExceeded { .. }
            | Self::ResponseTooLarge { .. }
            | Self::ResponseEndedBeforeHeaders { .. }
            | Self::MalformedResponse { .. }
            | Self::AcceptMismatch => None,
        }
    }
}

struct ParsedOpeningResponse {
    status_code: u16,
    byte_count: usize,
}

fn expected_accept_value(client_key: &WebDriverBiDiWebSocketClientKey) -> String {
    let mut digest = Sha1::new();
    digest.update(client_key.as_str().as_bytes());
    digest.update(RFC6455_WEBSOCKET_GUID);
    STANDARD.encode(digest.finalize())
}

fn is_http_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

fn has_header_token(value: &str, expected: &str) -> bool {
    value
        .split(',')
        .map(str::trim)
        .any(|token| token.eq_ignore_ascii_case(expected))
}

#[allow(clippy::collapsible_if)]
fn parse_opening_response(
    response: &[u8],
    client_key: &WebDriverBiDiWebSocketClientKey,
) -> Result<ParsedOpeningResponse, WebDriverBiDiWebSocketHandshakeResponseError> {
    if !response.ends_with(b"\r\n\r\n") {
        return Err(
            WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                reason: "response is missing its CRLF header terminator",
            },
        );
    }
    let response_text = std::str::from_utf8(response).map_err(|_| {
        WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
            reason: "response headers are not valid UTF-8",
        }
    })?;
    let header_text = &response_text[..response_text.len() - 4];
    let (status_line, header_lines) = header_text
        .split_once("\r\n")
        .map_or((header_text, ""), |(line, rest)| (line, rest));
    if status_line.bytes().any(|byte| byte < 0x20 || byte == 0x7f) {
        return Err(
            WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                reason: "status line contains a control byte",
            },
        );
    }
    let status_code = status_line
        .strip_prefix("HTTP/1.1 ")
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|value| value.parse::<u16>().ok());
    if status_code != Some(101) {
        return Err(
            WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                reason: "status line is not HTTP/1.1 101",
            },
        );
    }

    let mut upgrade_has_websocket = false;
    let mut connection_has_upgrade = false;
    let mut accept = None;
    for line in header_lines.split("\r\n") {
        if line.is_empty()
            || line
                .as_bytes()
                .first()
                .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        {
            return Err(
                WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                    reason: "header line is empty or folded",
                },
            );
        }
        let (name, value) = line.split_once(':').ok_or(
            WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                reason: "header line has no colon",
            },
        )?;
        if name.is_empty() || !name.bytes().all(is_http_token_byte) {
            return Err(
                WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                    reason: "header name is not an HTTP token",
                },
            );
        }
        let value = value.trim_matches([' ', '\t']);
        if value.bytes().any(|byte| byte < 0x20 || byte == 0x7f) {
            return Err(
                WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                    reason: "header value contains a control byte",
                },
            );
        }
        if name.eq_ignore_ascii_case("upgrade") {
            upgrade_has_websocket |= has_header_token(value, "websocket");
        } else if name.eq_ignore_ascii_case("connection") {
            connection_has_upgrade |= has_header_token(value, "upgrade");
        } else if name.eq_ignore_ascii_case("sec-websocket-accept") {
            if accept.is_some() {
                return Err(
                    WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                        reason: "response repeats the Sec-WebSocket-Accept header",
                    },
                );
            }
            accept = Some(value);
        }
    }

    if !upgrade_has_websocket {
        return Err(
            WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                reason: "Upgrade header does not contain websocket",
            },
        );
    }
    if !connection_has_upgrade {
        return Err(
            WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                reason: "Connection header does not contain Upgrade",
            },
        );
    }
    let Some(accept) = accept else {
        return Err(
            WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse {
                reason: "response has no Sec-WebSocket-Accept header",
            },
        );
    };
    if accept != expected_accept_value(client_key) {
        return Err(WebDriverBiDiWebSocketHandshakeResponseError::AcceptMismatch);
    }

    Ok(ParsedOpeningResponse {
        status_code: 101,
        byte_count: response.len(),
    })
}

trait OpeningResponseReader {
    fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()>;
    fn read_response_bytes(&mut self, bytes: &mut [u8]) -> io::Result<usize>;
}

impl OpeningResponseReader for TcpStream {
    fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        TcpStream::set_nonblocking(self, nonblocking)
    }

    fn read_response_bytes(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.read(bytes)
    }
}

fn read_opening_response_with_clock(
    reader: &mut dyn OpeningResponseReader,
    client_key: &WebDriverBiDiWebSocketClientKey,
    response_timeout: Duration,
    now: &mut dyn FnMut() -> Instant,
) -> Result<(u16, usize), WebDriverBiDiWebSocketHandshakeResponseError> {
    let deadline = now() + response_timeout;
    let mut response = Vec::new();

    reader.set_nonblocking(true).map_err(|source| {
        WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadModeConfigurationFailed {
            bytes_read: 0,
            source,
        }
    })?;

    loop {
        let remaining = deadline.saturating_duration_since(now());
        if remaining.is_zero() {
            return Err(
                WebDriverBiDiWebSocketHandshakeResponseError::ResponseDeadlineExceeded {
                    bytes_read: response.len(),
                },
            );
        }
        if response.len() >= MAX_WEBSOCKET_OPENING_RESPONSE_BYTES {
            return Err(
                WebDriverBiDiWebSocketHandshakeResponseError::ResponseTooLarge {
                    bytes_read: response.len(),
                    maximum_bytes: MAX_WEBSOCKET_OPENING_RESPONSE_BYTES,
                },
            );
        }
        let mut byte = [0_u8; 1];
        match reader.read_response_bytes(&mut byte) {
            Ok(0) => {
                return Err(
                    WebDriverBiDiWebSocketHandshakeResponseError::ResponseEndedBeforeHeaders {
                        bytes_read: response.len(),
                    },
                );
            }
            Ok(1) => {
                response.push(byte[0]);
                if response.ends_with(b"\r\n\r\n") {
                    if deadline.saturating_duration_since(now()).is_zero() {
                        return Err(
                            WebDriverBiDiWebSocketHandshakeResponseError::ResponseDeadlineExceeded {
                                bytes_read: response.len(),
                            },
                        );
                    }
                    let parsed = parse_opening_response(&response, client_key)?;
                    reader.set_nonblocking(false).map_err(|source| {
                        WebDriverBiDiWebSocketHandshakeResponseError::ReadModeCleanupFailed {
                            source,
                        }
                    })?;
                    return Ok((parsed.status_code, parsed.byte_count));
                }
            }
            Ok(_) => {
                return Err(
                    WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadFailed {
                        bytes_read: response.len(),
                        source: io::Error::new(
                            io::ErrorKind::InvalidData,
                            "response reader returned more bytes than requested",
                        ),
                    },
                );
            }
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source)
                if matches!(
                    source.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                if deadline.saturating_duration_since(now()).is_zero() {
                    return Err(
                        WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadTimedOut {
                            bytes_read: response.len(),
                            source,
                        },
                    );
                }
                thread::sleep(Duration::from_millis(1));
            }
            Err(source) => {
                return Err(
                    WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadFailed {
                        bytes_read: response.len(),
                        source,
                    },
                );
            }
        }
    }
}

/// Fail-closed errors while writing one bounded WebDriver BiDi WebSocket opening request.
#[derive(Debug)]
pub enum WebDriverBiDiWebSocketOpeningWriteError {
    /// The requested total write deadline was zero or above the reviewed resource ceiling.
    InvalidWriteTimeout {
        /// Rejected caller-supplied deadline.
        write_timeout: Duration,
        /// Maximum reviewed deadline accepted by this boundary.
        maximum_timeout: Duration,
    },
    /// The monotonic total write deadline elapsed before the complete request was written.
    WriteDeadlineExceeded {
        /// Number of request bytes written before the deadline elapsed.
        bytes_written: usize,
    },
    /// Applying the remaining operating-system write timeout failed.
    WriteTimeoutConfigurationFailed {
        /// Number of request bytes already written before configuration failed.
        bytes_written: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A bounded socket write reported timeout or would-block before completion.
    WriteTimedOut {
        /// Number of request bytes written before the timed-out operation.
        bytes_written: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A socket write returned zero bytes before the request was complete.
    WriteZero {
        /// Number of request bytes written before the zero-length write.
        bytes_written: usize,
    },
    /// A non-recoverable socket write failed before the complete request was emitted.
    WriteFailed {
        /// Number of request bytes written before the failure.
        bytes_written: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// Clearing the operation-local socket write timeout failed after all request bytes were sent.
    WriteTimeoutCleanupFailed {
        /// Number of request bytes already written before cleanup failed.
        bytes_written: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketOpeningWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWriteTimeout { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening write timeout is outside the reviewed bound",
            ),
            Self::WriteDeadlineExceeded { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening write exceeded its monotonic deadline",
            ),
            Self::WriteTimeoutConfigurationFailed { .. } => formatter.write_str(
                "failed to configure the bounded WebDriver BiDi WebSocket opening write timeout",
            ),
            Self::WriteTimedOut { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening write timed out before the request was complete",
            ),
            Self::WriteZero { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening write returned zero before the request was complete",
            ),
            Self::WriteFailed { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening write failed before the request was complete",
            ),
            Self::WriteTimeoutCleanupFailed { .. } => formatter.write_str(
                "failed to clear the WebDriver BiDi WebSocket opening write timeout before handoff",
            ),
        }
    }
}

impl Error for WebDriverBiDiWebSocketOpeningWriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::WriteTimeoutConfigurationFailed { source, .. }
            | Self::WriteTimedOut { source, .. }
            | Self::WriteFailed { source, .. }
            | Self::WriteTimeoutCleanupFailed { source, .. } => Some(source),
            Self::InvalidWriteTimeout { .. }
            | Self::WriteDeadlineExceeded { .. }
            | Self::WriteZero { .. } => None,
        }
    }
}

trait OpeningRequestWriter {
    fn set_write_timeout(&self, timeout: Duration) -> io::Result<()>;
    fn clear_write_timeout(&self) -> io::Result<()>;
    fn write_request_bytes(&mut self, bytes: &[u8]) -> io::Result<usize>;
}

impl OpeningRequestWriter for TcpStream {
    fn set_write_timeout(&self, timeout: Duration) -> io::Result<()> {
        TcpStream::set_write_timeout(self, Some(timeout))
    }

    fn clear_write_timeout(&self) -> io::Result<()> {
        TcpStream::set_write_timeout(self, None)
    }

    fn write_request_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.write(bytes)
    }
}

fn write_request_with_clock(
    writer: &mut dyn OpeningRequestWriter,
    request: &[u8],
    write_timeout: Duration,
    now: &mut dyn FnMut() -> Instant,
) -> Result<usize, WebDriverBiDiWebSocketOpeningWriteError> {
    let deadline = now() + write_timeout;
    let mut bytes_written = 0;

    while bytes_written < request.len() {
        let remaining = deadline.saturating_duration_since(now());
        if remaining.is_zero() {
            return Err(
                WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded { bytes_written },
            );
        }
        writer.set_write_timeout(remaining).map_err(|source| {
            WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutConfigurationFailed {
                bytes_written,
                source,
            }
        })?;

        match writer.write_request_bytes(&request[bytes_written..]) {
            Ok(0) => {
                return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written });
            }
            Ok(count) => {
                bytes_written += count;
                if deadline.saturating_duration_since(now()).is_zero() {
                    return Err(
                        WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded {
                            bytes_written,
                        },
                    );
                }
            }
            Err(source) => {
                if source.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if matches!(
                    source.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) {
                    return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteTimedOut {
                        bytes_written,
                        source,
                    });
                }
                return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteFailed {
                    bytes_written,
                    source,
                });
            }
        }
    }

    writer.clear_write_timeout().map_err(|source| {
        WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutCleanupFailed {
            bytes_written,
            source,
        }
    })?;

    Ok(bytes_written)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod opening_write_tests {
    use super::*;
    use std::{collections::VecDeque, net::TcpListener, thread};

    #[derive(Debug)]
    enum WriteAction {
        Count(usize),
        Error(io::ErrorKind),
    }

    #[derive(Debug)]
    struct FakeWriter {
        timeout_error: Option<io::ErrorKind>,
        clear_timeout_error: Option<io::ErrorKind>,
        actions: VecDeque<WriteAction>,
    }

    impl FakeWriter {
        fn new(actions: impl IntoIterator<Item = WriteAction>) -> Self {
            Self {
                timeout_error: None,
                clear_timeout_error: None,
                actions: actions.into_iter().collect(),
            }
        }
    }

    impl OpeningRequestWriter for FakeWriter {
        fn set_write_timeout(&self, _timeout: Duration) -> io::Result<()> {
            if let Some(kind) = self.timeout_error {
                return Err(io::Error::from(kind));
            }
            Ok(())
        }

        fn clear_write_timeout(&self) -> io::Result<()> {
            if let Some(kind) = self.clear_timeout_error {
                return Err(io::Error::from(kind));
            }
            Ok(())
        }

        fn write_request_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let action = self
                .actions
                .pop_front()
                .unwrap_or(WriteAction::Count(bytes.len()));
            match action {
                WriteAction::Count(count) => Ok(count.min(bytes.len())),
                WriteAction::Error(kind) => Err(io::Error::from(kind)),
            }
        }
    }

    #[derive(Clone, Debug)]
    enum ReadAction {
        Byte(u8),
        Count(usize),
        End,
        Error(io::ErrorKind),
    }

    #[derive(Debug)]
    struct FakeReader {
        actions: VecDeque<ReadAction>,
        mode_error: Option<io::ErrorKind>,
        cleanup_error: Option<io::ErrorKind>,
    }

    impl FakeReader {
        fn new(actions: impl IntoIterator<Item = ReadAction>) -> Self {
            Self {
                actions: actions.into_iter().collect(),
                mode_error: None,
                cleanup_error: None,
            }
        }
    }

    impl OpeningResponseReader for FakeReader {
        fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
            let error = if nonblocking {
                self.mode_error
            } else {
                self.cleanup_error
            };
            error.map_or(Ok(()), |kind| Err(io::Error::from(kind)))
        }

        fn read_response_bytes(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            match self.actions.pop_front().unwrap_or(ReadAction::End) {
                ReadAction::Byte(byte) => {
                    bytes[0] = byte;
                    Ok(1)
                }
                ReadAction::Count(count) => Ok(count),
                ReadAction::End => Ok(0),
                ReadAction::Error(kind) => Err(io::Error::from(kind)),
            }
        }
    }

    fn client_key() -> WebDriverBiDiWebSocketClientKey {
        WebDriverBiDiWebSocketClientKey::new("dGhlIHNhbXBsZSBub25jZQ==")
            .expect("test client key must be valid")
    }

    fn valid_response() -> Vec<u8> {
        b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n".to_vec()
    }

    fn byte_actions(bytes: &[u8]) -> Vec<ReadAction> {
        bytes.iter().copied().map(ReadAction::Byte).collect()
    }

    fn is_malformed_response(response: &[u8], key: &WebDriverBiDiWebSocketClientKey) -> bool {
        matches!(
            parse_opening_response(response, key),
            Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { .. })
        )
    }

    fn read_with_fake(
        reader: &mut FakeReader,
        now_values: impl IntoIterator<Item = Instant>,
    ) -> Result<(u16, usize), WebDriverBiDiWebSocketHandshakeResponseError> {
        let key = client_key();
        let fallback = Instant::now();
        let mut now_values = now_values.into_iter();
        let mut now = || now_values.next().unwrap_or(fallback);
        read_opening_response_with_clock(reader, &key, Duration::from_secs(1), &mut now)
    }

    #[test]
    fn parser_accepts_case_insensitive_upgrade_tokens_and_rejects_malformed_headers() {
        let key = client_key();
        let response = b"HTTP/1.1 101 Switching Protocols\r\nUpGrAdE: WebSocket\r\nConnection: keep-alive, Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\nX-Test: retained\r\n\r\n";
        let parsed = parse_opening_response(response, &key).expect("valid response");
        assert_eq!(parsed.status_code, 101);
        assert_eq!(parsed.byte_count, response.len());
        assert!(!is_malformed_response(response, &key));
        let same_length_mismatch = String::from_utf8(response.to_vec())
            .expect("valid response fixture")
            .replace(
                "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=",
                "s3pPLMBiTxaQ9kYGzzhZRbK+xOoX",
            );
        assert!(parse_opening_response(same_length_mismatch.as_bytes(), &key).is_err());

        let malformed_responses = [
            b"HTTP/1.1 101".to_vec(),
            vec![0xff, b'\r', b'\n', b'\r', b'\n'],
            b"HTTP/1.1 101\0 Switching Protocols\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\n Upgrade: websocket\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nBad Header: value\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\n: value\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: web\x01socket\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nUpgrade: websocket\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nConnection: Upgrade\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nSec-WebSocket-Accept: one\r\nSec-WebSocket-Accept: two\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: h2c\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: keep-alive\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n".to_vec(),
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n".to_vec(),
        ];
        for response in malformed_responses {
            assert!(is_malformed_response(&response, &key));
        }
    }

    #[test]
    fn bounded_response_reader_covers_deadlines_size_io_and_cleanup() {
        let start = Instant::now();

        let mut valid_reader = FakeReader::new(byte_actions(&valid_response()));
        let valid = read_with_fake(&mut valid_reader, [start]);
        assert!(valid.is_ok());

        let mut malformed_reader = FakeReader::new(byte_actions(b"HTTP/1.1 200 OK\r\n\r\n"));
        assert!(read_with_fake(&mut malformed_reader, [start]).is_err());

        let mut interrupted_reader = FakeReader::new(
            std::iter::once(ReadAction::Error(io::ErrorKind::Interrupted))
                .chain(byte_actions(&valid_response())),
        );
        assert!(read_with_fake(&mut interrupted_reader, [start]).is_ok());

        let mut mode_error_reader = FakeReader::new([]);
        mode_error_reader.mode_error = Some(io::ErrorKind::InvalidInput);
        assert!(read_with_fake(&mut mode_error_reader, [start]).is_err());

        let mut ended_reader = FakeReader::new([ReadAction::End]);
        assert!(read_with_fake(&mut ended_reader, [start]).is_err());

        let mut count_reader = FakeReader::new([ReadAction::Count(2)]);
        assert!(read_with_fake(&mut count_reader, [start]).is_err());

        let mut failed_reader = FakeReader::new([ReadAction::Error(io::ErrorKind::BrokenPipe)]);
        assert!(read_with_fake(&mut failed_reader, [start]).is_err());

        let mut retrying_reader = FakeReader::new(
            std::iter::once(ReadAction::Error(io::ErrorKind::WouldBlock))
                .chain(byte_actions(&valid_response())),
        );
        assert!(read_with_fake(&mut retrying_reader, [start]).is_ok());

        let mut timed_out_reader = FakeReader::new([ReadAction::Error(io::ErrorKind::TimedOut)]);
        assert!(
            read_with_fake(
                &mut timed_out_reader,
                [start, start, start + Duration::from_secs(1)]
            )
            .is_err()
        );

        let mut deadline_reader = FakeReader::new([ReadAction::End]);
        assert!(
            read_with_fake(
                &mut deadline_reader,
                [start, start + Duration::from_secs(1)]
            )
            .is_err()
        );

        let mut late_response_reader = FakeReader::new(byte_actions(&valid_response()));
        let mut late_response_times = vec![start; valid_response().len() + 1];
        late_response_times.push(start + Duration::from_secs(1));
        assert!(read_with_fake(&mut late_response_reader, late_response_times).is_err());

        let mut cleanup_reader = FakeReader::new(byte_actions(&valid_response()));
        cleanup_reader.cleanup_error = Some(io::ErrorKind::InvalidInput);
        assert!(read_with_fake(&mut cleanup_reader, [start]).is_err());

        let mut too_large_reader = FakeReader::new(std::iter::repeat_n(
            ReadAction::Byte(b'a'),
            MAX_WEBSOCKET_OPENING_RESPONSE_BYTES,
        ));
        assert!(read_with_fake(&mut too_large_reader, [start]).is_err());
    }

    #[test]
    fn response_errors_have_deterministic_messages_and_sources() {
        let source = io::Error::from(io::ErrorKind::InvalidInput);
        let errors = [
            WebDriverBiDiWebSocketHandshakeResponseError::InvalidResponseTimeout {
                response_timeout: Duration::ZERO,
                maximum_timeout: MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT,
            },
            WebDriverBiDiWebSocketHandshakeResponseError::ResponseDeadlineExceeded {
                bytes_read: 1,
            },
            WebDriverBiDiWebSocketHandshakeResponseError::ResponseTooLarge {
                bytes_read: 1,
                maximum_bytes: 1,
            },
            WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadModeConfigurationFailed {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::InvalidInput),
            },
            WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadTimedOut {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
            WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadFailed {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::BrokenPipe),
            },
            WebDriverBiDiWebSocketHandshakeResponseError::ResponseEndedBeforeHeaders {
                bytes_read: 1,
            },
            WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "test" },
            WebDriverBiDiWebSocketHandshakeResponseError::AcceptMismatch,
            WebDriverBiDiWebSocketHandshakeResponseError::ReadModeCleanupFailed { source },
        ];
        for (error, has_source) in errors.iter().zip([
            false, false, false, true, true, true, false, false, false, true,
        ]) {
            assert!(!error.to_string().is_empty());
            assert_eq!(error.source().is_some(), has_source);
        }
    }

    #[test]
    fn bounded_writer_completes_partial_and_interrupted_writes() {
        let mut writer = FakeWriter::new([
            WriteAction::Count(2),
            WriteAction::Error(io::ErrorKind::Interrupted),
            WriteAction::Count(3),
        ]);
        let start = Instant::now();
        let mut times = VecDeque::from([start, start, start, start]);
        let mut now = || times.pop_front().unwrap_or(start);
        let result =
            write_request_with_clock(&mut writer, b"hello", Duration::from_secs(1), &mut now);
        let is_five = |candidate: Result<usize, WebDriverBiDiWebSocketOpeningWriteError>| {
            matches!(candidate, Ok(5))
        };
        assert!(is_five(result));
        assert!(!is_five(Ok(4)));
    }

    fn join_loopback_server(server: thread::JoinHandle<io::Result<()>>) -> bool {
        match server.join() {
            Ok(result) => {
                result.expect("loopback server must accept the client");
                false
            }
            Err(_) => true,
        }
    }

    #[test]
    fn bounded_writer_clears_real_socket_timeout_before_success() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("test listener must bind");
        let address = listener
            .local_addr()
            .expect("test listener address must be available");
        let server = thread::spawn(move || listener.accept().map(|_| ()));
        let mut stream = TcpStream::connect(address).expect("test client must connect");
        let start = Instant::now();
        let mut now = || start;

        let request_byte_count =
            write_request_with_clock(&mut stream, b"opening", Duration::from_secs(1), &mut now)
                .expect("the opening request must be written");

        assert_eq!(request_byte_count, 7);
        assert_eq!(
            stream
                .write_timeout()
                .expect("the socket timeout must be inspectable"),
            None
        );
        assert!(!join_loopback_server(server));
    }

    #[test]
    fn panicked_loopback_server_is_reported() {
        let server = thread::spawn(|| -> io::Result<()> {
            std::panic::resume_unwind(Box::new("intentional test-only server panic"));
        });

        assert!(join_loopback_server(server));
    }

    #[test]
    fn bounded_writer_rejects_cleanup_failure_without_success_handoff() {
        let mut writer = FakeWriter::new([WriteAction::Count(1)]);
        writer.clear_timeout_error = Some(io::ErrorKind::InvalidInput);
        let start = Instant::now();
        let mut now = || start;

        let result = write_request_with_clock(&mut writer, b"x", Duration::from_secs(1), &mut now);
        let is_cleanup_failure =
            |candidate: Result<usize, WebDriverBiDiWebSocketOpeningWriteError>| {
                matches!(
                    candidate,
                    Err(
                        WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutCleanupFailed {
                            bytes_written: 1,
                            ..
                        }
                    )
                )
            };
        assert!(is_cleanup_failure(result));
        assert!(!is_cleanup_failure(Err(
            WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written: 1 }
        )));
    }

    #[test]
    fn bounded_writer_rejects_completion_observed_after_total_deadline() {
        let mut writer = FakeWriter::new([WriteAction::Count(1)]);
        let start = Instant::now();
        let mut times = VecDeque::from([start, start, start + Duration::from_secs(1)]);
        let mut now = || times.pop_front().unwrap_or(start + Duration::from_secs(1));
        let result = write_request_with_clock(&mut writer, b"x", Duration::from_secs(1), &mut now);
        let is_deadline_after_one =
            |candidate: Result<usize, WebDriverBiDiWebSocketOpeningWriteError>| {
                matches!(
                    candidate,
                    Err(
                        WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded {
                            bytes_written: 1
                        }
                    )
                )
            };
        assert!(is_deadline_after_one(result));
        assert!(!is_deadline_after_one(Err(
            WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written: 1 }
        )));
    }

    #[test]
    fn bounded_writer_classifies_deadline_timeout_zero_and_io_failures() {
        let start = Instant::now();

        let mut deadline_writer = FakeWriter::new([]);
        let mut deadline_times = VecDeque::from([start, start + Duration::from_secs(1)]);
        let mut deadline_now = || deadline_times.pop_front().unwrap_or(start);
        let deadline = write_request_with_clock(
            &mut deadline_writer,
            b"x",
            Duration::from_secs(1),
            &mut deadline_now,
        );
        let is_deadline_before_write =
            |candidate: Result<usize, WebDriverBiDiWebSocketOpeningWriteError>| {
                matches!(
                    candidate,
                    Err(
                        WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded {
                            bytes_written: 0
                        }
                    )
                )
            };
        assert!(is_deadline_before_write(deadline));
        assert!(!is_deadline_before_write(Err(
            WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written: 0 }
        )));

        let mut zero_writer = FakeWriter::new([WriteAction::Count(0)]);
        let mut zero_now = || start;
        let zero = write_request_with_clock(
            &mut zero_writer,
            b"x",
            Duration::from_secs(1),
            &mut zero_now,
        );
        let is_zero_write = |candidate: Result<usize, WebDriverBiDiWebSocketOpeningWriteError>| {
            matches!(
                candidate,
                Err(WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written: 0 })
            )
        };
        assert!(is_zero_write(zero));
        assert!(!is_zero_write(Err(
            WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded { bytes_written: 0 }
        )));

        for kind in [io::ErrorKind::TimedOut, io::ErrorKind::WouldBlock] {
            let mut writer = FakeWriter::new([WriteAction::Error(kind)]);
            let mut now = || start;
            let timed_out =
                write_request_with_clock(&mut writer, b"x", Duration::from_secs(1), &mut now);
            let is_timed_out =
                |candidate: Result<usize, WebDriverBiDiWebSocketOpeningWriteError>| {
                    matches!(
                        candidate,
                        Err(WebDriverBiDiWebSocketOpeningWriteError::WriteTimedOut {
                            bytes_written: 0,
                            ..
                        })
                    )
                };
            assert!(is_timed_out(timed_out));
            assert!(!is_timed_out(Err(
                WebDriverBiDiWebSocketOpeningWriteError::WriteFailed {
                    bytes_written: 0,
                    source: io::Error::from(kind),
                }
            )));
        }

        let mut failed_writer = FakeWriter::new([WriteAction::Error(io::ErrorKind::BrokenPipe)]);
        let mut failed_now = || start;
        let failed = write_request_with_clock(
            &mut failed_writer,
            b"x",
            Duration::from_secs(1),
            &mut failed_now,
        );
        let is_failed = |candidate: Result<usize, WebDriverBiDiWebSocketOpeningWriteError>| {
            matches!(
                candidate,
                Err(WebDriverBiDiWebSocketOpeningWriteError::WriteFailed {
                    bytes_written: 0,
                    ..
                })
            )
        };
        assert!(is_failed(failed));
        assert!(!is_failed(Err(
            WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written: 0 }
        )));

        let mut configuration_writer = FakeWriter::new([]);
        configuration_writer.timeout_error = Some(io::ErrorKind::InvalidInput);
        let mut configuration_now = || start;
        let configuration = write_request_with_clock(
            &mut configuration_writer,
            b"x",
            Duration::from_secs(1),
            &mut configuration_now,
        );
        let is_configuration_failure =
            |candidate: Result<usize, WebDriverBiDiWebSocketOpeningWriteError>| {
                matches!(
                    candidate,
                    Err(
                        WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutConfigurationFailed {
                            bytes_written: 0,
                            ..
                        }
                    )
                )
            };
        assert!(is_configuration_failure(configuration));
        assert!(!is_configuration_failure(Err(
            WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written: 0 }
        )));
    }

    #[test]
    fn opening_write_errors_have_deterministic_messages_and_sources() {
        let invalid = WebDriverBiDiWebSocketOpeningWriteError::InvalidWriteTimeout {
            write_timeout: Duration::ZERO,
            maximum_timeout: MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT,
        };
        let deadline =
            WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded { bytes_written: 1 };
        let configure = WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutConfigurationFailed {
            bytes_written: 1,
            source: io::Error::from(io::ErrorKind::InvalidInput),
        };
        let timed_out = WebDriverBiDiWebSocketOpeningWriteError::WriteTimedOut {
            bytes_written: 1,
            source: io::Error::from(io::ErrorKind::TimedOut),
        };
        let zero = WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written: 1 };
        let failed = WebDriverBiDiWebSocketOpeningWriteError::WriteFailed {
            bytes_written: 1,
            source: io::Error::from(io::ErrorKind::BrokenPipe),
        };
        let cleanup = WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutCleanupFailed {
            bytes_written: 1,
            source: io::Error::from(io::ErrorKind::InvalidInput),
        };

        assert!(!invalid.to_string().is_empty());
        assert!(!deadline.to_string().is_empty());
        assert!(!configure.to_string().is_empty());
        assert!(!timed_out.to_string().is_empty());
        assert!(!zero.to_string().is_empty());
        assert!(!failed.to_string().is_empty());
        assert!(!cleanup.to_string().is_empty());
        assert!(invalid.source().is_none());
        assert!(deadline.source().is_none());
        assert!(configure.source().is_some());
        assert!(timed_out.source().is_some());
        assert!(zero.source().is_none());
        assert!(failed.source().is_some());
        assert!(cleanup.source().is_some());
    }
}
