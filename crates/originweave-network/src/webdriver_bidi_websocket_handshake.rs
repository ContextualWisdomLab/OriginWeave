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
const MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES: usize = 1024 * 1024;

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

/// Maximum payload bytes admitted for one WebSocket frame.
pub const MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE: usize = MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES;

/// Maximum wall-clock budget accepted for one bounded WebSocket frame I/O operation.
pub const MAX_WEBSOCKET_FRAME_TIMEOUT: Duration = Duration::from_secs(5);

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

/// Caller-supplied RFC 6455 mask key for one client-to-server frame.
///
/// RFC 6455 requires every client frame to carry a fresh, unpredictable four-byte key. This type
/// preserves that requirement at the API boundary without inventing an entropy source; callers must
/// obtain a fresh key from an approved randomness source for every frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketMaskKey([u8; 4]);

impl WebDriverBiDiWebSocketMaskKey {
    /// Admit one four-byte caller-supplied frame mask key.
    #[must_use]
    pub const fn new(value: [u8; 4]) -> Self {
        Self(value)
    }

    /// Borrow the exact four-byte key used on the wire.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 4] {
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
            .field("client_key", &"<retained for Sec-WebSocket-Accept validation>")
            .field("request_byte_count", &self.request_byte_count)
            .field("write_timeout", &self.write_timeout)
            .finish()
    }
}

impl WebDriverBiDiWebSocketOpeningRequestSent {
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        &self.transport_evidence
    }

    #[must_use]
    pub const fn client_key(&self) -> &WebDriverBiDiWebSocketClientKey {
        &self.client_key
    }

    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.request_byte_count
    }

    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.write_timeout
    }

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
            .field("client_key", &"<retained for WebSocket session correlation>")
            .field("response_status", &self.response_status)
            .field("response_byte_count", &self.response_byte_count)
            .field("response_timeout", &self.response_timeout)
            .field("request_byte_count", &self.request_byte_count)
            .field("write_timeout", &self.write_timeout)
            .finish()
    }
}

impl WebDriverBiDiWebSocketEstablished {
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        &self.transport_evidence
    }

    #[must_use]
    pub const fn client_key(&self) -> &WebDriverBiDiWebSocketClientKey {
        &self.client_key
    }

    #[must_use]
    pub const fn response_status(&self) -> u16 {
        self.response_status
    }

    #[must_use]
    pub const fn response_byte_count(&self) -> usize {
        self.response_byte_count
    }

    #[must_use]
    pub const fn response_timeout(&self) -> Duration {
        self.response_timeout
    }

    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.request_byte_count
    }

    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.write_timeout
    }

    pub fn write_text_frame(
        self,
        text: &str,
        masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<Self, WebDriverBiDiWebSocketFrameError> {
        validate_frame_timeout(frame_timeout)?;
        if text.len() > MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES {
            return Err(WebDriverBiDiWebSocketFrameError::FrameTooLarge {
                payload_bytes: text.len(),
                maximum_bytes: MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES,
            });
        }

        let frame = serialize_text_frame(text.as_bytes(), masking_key);
        let Self {
            mut stream,
            transport_evidence,
            client_key,
            response_status,
            response_byte_count,
            response_timeout,
            request_byte_count,
            write_timeout,
        } = self;
        let mut now = Instant::now;
        write_frame_with_clock(&mut stream, &frame, frame_timeout, &mut now)?;
        Ok(Self {
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

    pub fn read_frame(
        self,
        frame_timeout: Duration,
    ) -> Result<(Self, WebDriverBiDiWebSocketFrame), WebDriverBiDiWebSocketFrameError> {
        validate_frame_timeout(frame_timeout)?;
        let Self {
            mut stream,
            transport_evidence,
            client_key,
            response_status,
            response_byte_count,
            response_timeout,
            request_byte_count,
            write_timeout,
        } = self;
        let mut now = Instant::now;
        let frame = read_frame_with_clock(&mut stream, frame_timeout, &mut now)?;
        Ok((
            Self {
                stream,
                transport_evidence,
                client_key,
                response_status,
                response_byte_count,
                response_timeout,
                request_byte_count,
                write_timeout,
            },
            frame,
        ))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketFrame {
    fin: bool,
    opcode: u8,
    payload: Vec<u8>,
}

impl WebDriverBiDiWebSocketFrame {
    #[must_use]
    pub const fn fin(&self) -> bool {
        self.fin
    }

    #[must_use]
    pub const fn opcode(&self) -> u8 {
        self.opcode
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

fn validate_frame_timeout(frame_timeout: Duration) -> Result<(), WebDriverBiDiWebSocketFrameError> {
    if frame_timeout.is_zero() || frame_timeout > MAX_WEBSOCKET_FRAME_TIMEOUT {
        return Err(WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
            frame_timeout,
            maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
        });
    }
    Ok(())
}

fn is_valid_close_status_code(status_code: u16) -> bool {
    (1000..=4999).contains(&status_code) && !matches!(status_code, 1005 | 1006 | 1015)
}

#[derive(Debug)]
pub enum WebDriverBiDiWebSocketFrameError {
    InvalidFrameTimeout {
        frame_timeout: Duration,
        maximum_timeout: Duration,
    },
    FrameTooLarge {
        payload_bytes: usize,
        maximum_bytes: usize,
    },
    FrameReadModeConfigurationFailed {
        source: io::Error,
    },
    FrameReadTimedOut {
        bytes_read: usize,
        source: io::Error,
    },
    FrameReadFailed {
        bytes_read: usize,
        source: io::Error,
    },
    FrameEnded {
        bytes_read: usize,
    },
    MalformedFrame {
        reason: &'static str,
    },
    FrameWriteModeConfigurationFailed {
        bytes_written: usize,
        source: io::Error,
    },
    FrameWriteTimedOut {
        bytes_written: usize,
        source: io::Error,
    },
    FrameWriteFailed {
        bytes_written: usize,
        source: io::Error,
    },
    FrameWriteZero {
        bytes_written: usize,
    },
    FrameWriteCleanupFailed {
        source: io::Error,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFrameTimeout { .. } => formatter.write_str("WebDriver BiDi WebSocket frame timeout is outside the reviewed bound"),
            Self::FrameTooLarge { .. } => formatter.write_str("WebDriver BiDi WebSocket frame payload exceeded its bound"),
            Self::FrameReadModeConfigurationFailed { .. } => formatter.write_str("failed to configure bounded WebSocket frame reads"),
            Self::FrameReadTimedOut { .. } => formatter.write_str("WebDriver BiDi WebSocket frame read timed out"),
            Self::FrameReadFailed { .. } => formatter.write_str("WebDriver BiDi WebSocket frame read failed"),
            Self::FrameEnded { .. } => formatter.write_str("WebDriver BiDi WebSocket peer ended the frame stream"),
            Self::MalformedFrame { .. } => formatter.write_str("WebDriver BiDi WebSocket frame was malformed"),
            Self::FrameWriteModeConfigurationFailed { .. } => formatter.write_str("failed to configure bounded WebSocket frame writes"),
            Self::FrameWriteTimedOut { .. } => formatter.write_str("WebDriver BiDi WebSocket frame write timed out"),
            Self::FrameWriteFailed { .. } => formatter.write_str("WebDriver BiDi WebSocket frame write failed"),
            Self::FrameWriteZero { .. } => formatter.write_str("WebDriver BiDi WebSocket frame write made no progress"),
            Self::FrameWriteCleanupFailed { .. } => formatter.write_str("failed to clear the WebDriver BiDi WebSocket frame timeout"),
        }
    }
}

impl Error for WebDriverBiDiWebSocketFrameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FrameReadModeConfigurationFailed { source }
            | Self::FrameReadTimedOut { source, .. }
            | Self::FrameReadFailed { source, .. }
            | Self::FrameWriteModeConfigurationFailed { source, .. }
            | Self::FrameWriteTimedOut { source, .. }
            | Self::FrameWriteFailed { source, .. }
            | Self::FrameWriteCleanupFailed { source } => Some(source),
            Self::InvalidFrameTimeout { .. }
            | Self::FrameTooLarge { .. }
            | Self::FrameEnded { .. }
            | Self::MalformedFrame { .. }
            | Self::FrameWriteZero { .. } => None,
        }
    }
}

#[derive(Debug)]
pub enum WebDriverBiDiWebSocketHandshakeResponseError {
    InvalidResponseTimeout { response_timeout: Duration, maximum_timeout: Duration },
    ResponseDeadlineExceeded { bytes_read: usize },
    ResponseTooLarge { bytes_read: usize, maximum_bytes: usize },
    ResponseReadModeConfigurationFailed { bytes_read: usize, source: io::Error },
    ResponseReadTimedOut { bytes_read: usize, source: io::Error },
    ResponseReadFailed { bytes_read: usize, source: io::Error },
    ResponseEndedBeforeHeaders { bytes_read: usize },
    MalformedResponse { reason: &'static str },
    AcceptMismatch,
    ReadModeCleanupFailed { source: io::Error },
}

impl fmt::Display for WebDriverBiDiWebSocketHandshakeResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidResponseTimeout { .. } => formatter.write_str("WebDriver BiDi WebSocket opening response timeout is outside the reviewed bound"),
            Self::ResponseDeadlineExceeded { .. } => formatter.write_str("WebDriver BiDi WebSocket opening response exceeded its monotonic deadline"),
            Self::ResponseTooLarge { .. } => formatter.write_str("WebDriver BiDi WebSocket opening response exceeded its bounded header size"),
            Self::ResponseReadModeConfigurationFailed { .. } => formatter.write_str("failed to configure bounded nonblocking WebDriver BiDi WebSocket response reads"),
            Self::ResponseReadTimedOut { .. } => formatter.write_str("WebDriver BiDi WebSocket opening response timed out before completion"),
            Self::ResponseReadFailed { .. } => formatter.write_str("WebDriver BiDi WebSocket opening response read failed before completion"),
            Self::ResponseEndedBeforeHeaders { .. } => formatter.write_str("WebDriver BiDi WebSocket peer ended the stream before completing response headers"),
            Self::MalformedResponse { .. } => formatter.write_str("WebDriver BiDi WebSocket opening response was malformed or missing a required header"),
            Self::AcceptMismatch => formatter.write_str("WebDriver BiDi WebSocket opening response accept value did not match the client key"),
            Self::ReadModeCleanupFailed { .. } => formatter.write_str("failed to restore blocking WebDriver BiDi WebSocket response reads before handoff"),
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
            _ => None,
        }
    }
}

struct ParsedOpeningResponse { status_code: u16, byte_count: usize }

fn expected_accept_value(client_key: &WebDriverBiDiWebSocketClientKey) -> String {
    let mut digest = Sha1::new();
    digest.update(client_key.as_str().as_bytes());
    digest.update(RFC6455_WEBSOCKET_GUID);
    STANDARD.encode(digest.finalize())
}

fn is_http_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~')
}

fn has_header_token(value: &str, expected: &str) -> bool {
    value.split(',').map(str::trim).any(|token| token.eq_ignore_ascii_case(expected))
}

#[allow(clippy::collapsible_if)]
fn parse_opening_response(response: &[u8], client_key: &WebDriverBiDiWebSocketClientKey) -> Result<ParsedOpeningResponse, WebDriverBiDiWebSocketHandshakeResponseError> {
    if !response.ends_with(b"\r\n\r\n") {
        return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "response is missing its CRLF header terminator" });
    }
    let response_text = std::str::from_utf8(response).map_err(|_| WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "response headers are not valid UTF-8" })?;
    let header_text = &response_text[..response_text.len()-4];
    let (status_line, header_lines) = header_text.split_once("\r\n").map_or((header_text, ""), |(line, rest)| (line, rest));
    if status_line.bytes().any(|byte| byte < 0x20 || byte == 0x7f) { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "status line contains a control byte" }); }
    let status_code = status_line.strip_prefix("HTTP/1.1 ").and_then(|rest| rest.split_whitespace().next()).and_then(|value| value.parse::<u16>().ok());
    if status_code != Some(101) { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "status line is not HTTP/1.1 101" }); }
    let mut upgrade_has_websocket = false;
    let mut connection_has_upgrade = false;
    let mut accept = None;
    for line in header_lines.split("\r\n") {
        if line.is_empty() || line.as_bytes().first().is_some_and(|byte| matches!(byte, b' ' | b'\t')) { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "header line is empty or folded" }); }
        let (name, value) = line.split_once(':').ok_or(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "header line has no colon" })?;
        if name.is_empty() || !name.bytes().all(is_http_token_byte) { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "header name is not an HTTP token" }); }
        let value = value.trim_matches([' ', '\t']);
        if value.bytes().any(|byte| byte < 0x20 || byte == 0x7f) { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "header value contains a control byte" }); }
        if name.eq_ignore_ascii_case("upgrade") { upgrade_has_websocket |= has_header_token(value, "websocket"); }
        else if name.eq_ignore_ascii_case("connection") { connection_has_upgrade |= has_header_token(value, "upgrade"); }
        else if name.eq_ignore_ascii_case("sec-websocket-accept") { if accept.is_some() { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "response repeats the Sec-WebSocket-Accept header" }); } accept = Some(value); }
    }
    if !upgrade_has_websocket { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "Upgrade header does not contain websocket" }); }
    if !connection_has_upgrade { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "Connection header does not contain Upgrade" }); }
    let Some(accept) = accept else { return Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { reason: "response has no Sec-WebSocket-Accept header" }); };
    if accept != expected_accept_value(client_key) { return Err(WebDriverBiDiWebSocketHandshakeResponseError::AcceptMismatch); }
    Ok(ParsedOpeningResponse { status_code: 101, byte_count: response.len() })
}

trait OpeningResponseReader {
    fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()>;
    fn read_response_bytes(&mut self, bytes: &mut [u8]) -> io::Result<usize>;
}
impl OpeningResponseReader for TcpStream {
    fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> { TcpStream::set_nonblocking(self, nonblocking) }
    fn read_response_bytes(&mut self, bytes: &mut [u8]) -> io::Result<usize> { self.read(bytes) }
}

fn serialize_text_frame(payload: &[u8], masking_key: WebDriverBiDiWebSocketMaskKey) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len()+14);
    frame.push(0x81);
    match payload.len() {
        0..=125 => frame.push(0x80 | payload.len() as u8),
        126..=65_535 => { frame.push(0x80 | 126); frame.extend_from_slice(&(payload.len() as u16).to_be_bytes()); }
        length => { frame.push(0x80 | 127); frame.extend_from_slice(&(length as u64).to_be_bytes()); }
    }
    frame.extend_from_slice(masking_key.as_bytes());
    frame.extend(payload.iter().enumerate().map(|(index, byte)| byte ^ masking_key.as_bytes()[index % masking_key.as_bytes().len()]));
    frame
}

trait FrameWriter { fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()>; fn write_frame_bytes(&mut self, bytes: &[u8]) -> io::Result<usize>; }
impl FrameWriter for TcpStream { fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> { TcpStream::set_write_timeout(self, timeout) } fn write_frame_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> { self.write(bytes) } }

fn write_frame_with_clock(writer: &mut dyn FrameWriter, frame: &[u8], frame_timeout: Duration, now: &mut dyn FnMut() -> Instant) -> Result<usize, WebDriverBiDiWebSocketFrameError> {
    let deadline = now()+frame_timeout;
    let mut bytes_written = 0;
    while bytes_written < frame.len() {
        let remaining = deadline.saturating_duration_since(now());
        if remaining.is_zero() { return Err(WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut { bytes_written, source: io::Error::new(io::ErrorKind::TimedOut, "frame write deadline elapsed") }); }
        writer.set_write_timeout(Some(remaining)).map_err(|source| WebDriverBiDiWebSocketFrameError::FrameWriteModeConfigurationFailed { bytes_written, source })?;
        match writer.write_frame_bytes(&frame[bytes_written..]) {
            Ok(0) => return Err(WebDriverBiDiWebSocketFrameError::FrameWriteZero { bytes_written }),
            Ok(written) => bytes_written += written,
            Err(source) if source.kind()==io::ErrorKind::Interrupted => continue,
            Err(source) if matches!(source.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock) => { if deadline.saturating_duration_since(now()).is_zero() { return Err(WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut { bytes_written, source }); } thread::sleep(Duration::from_millis(1)); continue; },
            Err(source) => return Err(WebDriverBiDiWebSocketFrameError::FrameWriteFailed { bytes_written, source }),
        }
    }
    writer.set_write_timeout(None).map_err(|source| WebDriverBiDiWebSocketFrameError::FrameWriteCleanupFailed { source })?;
    Ok(bytes_written)
}

fn read_frame_with_clock(reader: &mut dyn OpeningResponseReader, frame_timeout: Duration, now: &mut dyn FnMut() -> Instant) -> Result<WebDriverBiDiWebSocketFrame, WebDriverBiDiWebSocketFrameError> {
    let deadline = now()+frame_timeout;
    reader.set_nonblocking(true).map_err(|source| WebDriverBiDiWebSocketFrameError::FrameReadModeConfigurationFailed { source })?;
    let mut bytes_read=0;
    let mut header=[0_u8;2];
    read_frame_bytes_with_clock(reader,&mut header,&mut bytes_read,deadline,now)?;
    let first=header[0]; let second=header[1];
    if first & 0x70 != 0 { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "reserved frame bits are not negotiated" }); }
    let fin=first & 0x80 !=0; let opcode=first & 0x0f;
    match opcode { 0x0..=0x2 => {}, 0x8..=0xa => { if !fin { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "control frames must not be fragmented" }); } }, _ => return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "frame opcode is reserved or unsupported" }) }
    if second & 0x80 != 0 { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "server-to-client frames must not be masked" }); }
    let length_code=second & 0x7f;
    let payload_length=match length_code {
        0..=125 => u64::from(length_code),
        126 => { let mut extended=[0_u8;2]; read_frame_bytes_with_clock(reader,&mut extended,&mut bytes_read,deadline,now)?; let length=u64::from(u16::from_be_bytes(extended)); if length<126 { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "frame length encoding is not minimal" }); } length },
        _ => { let mut extended=[0_u8;8]; read_frame_bytes_with_clock(reader,&mut extended,&mut bytes_read,deadline,now)?; if extended[0]&0x80!=0 { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "frame length uses the reserved high bit" }); } let length=u64::from_be_bytes(extended); if length<65_536 { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "frame length encoding is not minimal" }); } length }
    };
    if payload_length > MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES as u64 { return Err(WebDriverBiDiWebSocketFrameError::FrameTooLarge { payload_bytes: payload_length.min(usize::MAX as u64) as usize, maximum_bytes: MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES }); }
    if opcode>=0x8 && payload_length>125 { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "control frame payload exceeds 125 bytes" }); }
    let payload_length=payload_length as usize;
    let mut payload=vec![0_u8;payload_length];
    read_frame_bytes_with_clock(reader,&mut payload,&mut bytes_read,deadline,now)?;
    if opcode==0x8 {
        if payload.len()==1 { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "Close frame payload must be empty or begin with a two-byte status code" }); }
        if payload.len()>1 {
            let status_code=u16::from_be_bytes([payload[0],payload[1]]);
            if !is_valid_close_status_code(status_code) { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "Close frame status code is not valid on the wire" }); }
            if std::str::from_utf8(&payload[2..]).is_err() { return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "Close frame reason is not valid UTF-8" }); }
        }
    }
    reader.set_nonblocking(false).map_err(|source| WebDriverBiDiWebSocketFrameError::FrameReadFailed { bytes_read, source })?;
    Ok(WebDriverBiDiWebSocketFrame { fin, opcode, payload })
}

fn read_frame_bytes_with_clock(reader:&mut dyn OpeningResponseReader,destination:&mut [u8],bytes_read:&mut usize,deadline:Instant,now:&mut dyn FnMut()->Instant)->Result<(),WebDriverBiDiWebSocketFrameError>{
    let mut offset=0;
    while offset<destination.len(){let remaining=deadline.saturating_duration_since(now()); if remaining.is_zero(){return Err(WebDriverBiDiWebSocketFrameError::FrameReadTimedOut{bytes_read:*bytes_read,source:io::Error::new(io::ErrorKind::TimedOut,"frame read deadline elapsed")});} match reader.read_response_bytes(&mut destination[offset..]){Ok(0)=>return Err(WebDriverBiDiWebSocketFrameError::FrameEnded{bytes_read:*bytes_read}),Ok(read) if read>destination.len()-offset=>return Err(WebDriverBiDiWebSocketFrameError::FrameReadFailed{bytes_read:*bytes_read,source:io::Error::new(io::ErrorKind::InvalidData,"frame reader returned more bytes than requested")}),Ok(read)=>{offset+=read;*bytes_read+=read;},Err(source) if source.kind()==io::ErrorKind::Interrupted=>{},Err(source) if matches!(source.kind(),io::ErrorKind::TimedOut|io::ErrorKind::WouldBlock)=>{if deadline.saturating_duration_since(now()).is_zero(){return Err(WebDriverBiDiWebSocketFrameError::FrameReadTimedOut{bytes_read:*bytes_read,source});} thread::sleep(Duration::from_millis(1));},Err(source)=>return Err(WebDriverBiDiWebSocketFrameError::FrameReadFailed{bytes_read:*bytes_read,source})}}
    Ok(())
}

fn read_opening_response_with_clock(reader:&mut dyn OpeningResponseReader,client_key:&WebDriverBiDiWebSocketClientKey,response_timeout:Duration,now:&mut dyn FnMut()->Instant)->Result<(u16,usize),WebDriverBiDiWebSocketHandshakeResponseError>{
    let deadline=now()+response_timeout; let mut response=Vec::new(); reader.set_nonblocking(true).map_err(|source|WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadModeConfigurationFailed{bytes_read:0,source})?;
    loop { let remaining=deadline.saturating_duration_since(now()); if remaining.is_zero(){return Err(WebDriverBiDiWebSocketHandshakeResponseError::ResponseDeadlineExceeded{bytes_read:response.len()});} if response.len()>=MAX_WEBSOCKET_OPENING_RESPONSE_BYTES{return Err(WebDriverBiDiWebSocketHandshakeResponseError::ResponseTooLarge{bytes_read:response.len(),maximum_bytes:MAX_WEBSOCKET_OPENING_RESPONSE_BYTES});} let mut byte=[0_u8;1]; match reader.read_response_bytes(&mut byte){Ok(0)=>return Err(WebDriverBiDiWebSocketHandshakeResponseError::ResponseEndedBeforeHeaders{bytes_read:response.len()}),Ok(1)=>{response.push(byte[0]);if response.ends_with(b"\r\n\r\n"){if deadline.saturating_duration_since(now()).is_zero(){return Err(WebDriverBiDiWebSocketHandshakeResponseError::ResponseDeadlineExceeded{bytes_read:response.len()});}let parsed=parse_opening_response(&response,client_key)?;reader.set_nonblocking(false).map_err(|source|WebDriverBiDiWebSocketHandshakeResponseError::ReadModeCleanupFailed{source})?;return Ok((parsed.status_code,parsed.byte_count));}},Ok(_)=>return Err(WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadFailed{bytes_read:response.len(),source:io::Error::new(io::ErrorKind::InvalidData,"response reader returned more bytes than requested")}),Err(source) if source.kind()==io::ErrorKind::Interrupted=>{},Err(source) if matches!(source.kind(),io::ErrorKind::TimedOut|io::ErrorKind::WouldBlock)=>{if deadline.saturating_duration_since(now()).is_zero(){return Err(WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadTimedOut{bytes_read:response.len(),source});}thread::sleep(Duration::from_millis(1));},Err(source)=>return Err(WebDriverBiDiWebSocketHandshakeResponseError::ResponseReadFailed{bytes_read:response.len(),source})}}
}

#[derive(Debug)]
pub enum WebDriverBiDiWebSocketOpeningWriteError { InvalidWriteTimeout{write_timeout:Duration,maximum_timeout:Duration},WriteDeadlineExceeded{bytes_written:usize},WriteTimeoutConfigurationFailed{bytes_written:usize,source:io::Error},WriteTimedOut{bytes_written:usize,source:io::Error},WriteZero{bytes_written:usize},WriteFailed{bytes_written:usize,source:io::Error},WriteTimeoutCleanupFailed{bytes_written:usize,source:io::Error} }
impl fmt::Display for WebDriverBiDiWebSocketOpeningWriteError{fn fmt(&self,formatter:&mut fmt::Formatter<'_>)->fmt::Result{match self{Self::InvalidWriteTimeout{..}=>formatter.write_str("WebDriver BiDi WebSocket opening write timeout is outside the reviewed bound"),Self::WriteDeadlineExceeded{..}=>formatter.write_str("WebDriver BiDi WebSocket opening write exceeded its monotonic deadline"),Self::WriteTimeoutConfigurationFailed{..}=>formatter.write_str("failed to configure the bounded WebDriver BiDi WebSocket opening write timeout"),Self::WriteTimedOut{..}=>formatter.write_str("WebDriver BiDi WebSocket opening write timed out before the request was complete"),Self::WriteZero{..}=>formatter.write_str("WebDriver BiDi WebSocket opening write returned zero before the request was complete"),Self::WriteFailed{..}=>formatter.write_str("WebDriver BiDi WebSocket opening write failed before the request was complete"),Self::WriteTimeoutCleanupFailed{..}=>formatter.write_str("failed to clear the WebDriver BiDi WebSocket opening write timeout before handoff")}}}
impl Error for WebDriverBiDiWebSocketOpeningWriteError{fn source(&self)->Option<&(dyn Error+'static)>{match self{Self::WriteTimeoutConfigurationFailed{source,..}|Self::WriteTimedOut{source,..}|Self::WriteFailed{source,..}|Self::WriteTimeoutCleanupFailed{source,..}=>Some(source),_=>None}}}

trait OpeningRequestWriter{fn set_write_timeout(&self,timeout:Duration)->io::Result<()>;fn clear_write_timeout(&self)->io::Result<()>;fn write_request_bytes(&mut self,bytes:&[u8])->io::Result<usize>;}
impl OpeningRequestWriter for TcpStream{fn set_write_timeout(&self,timeout:Duration)->io::Result<()>{TcpStream::set_write_timeout(self,Some(timeout))}fn clear_write_timeout(&self)->io::Result<()>{TcpStream::set_write_timeout(self,None)}fn write_request_bytes(&mut self,bytes:&[u8])->io::Result<usize>{self.write(bytes)}}
fn write_request_with_clock(writer:&mut dyn OpeningRequestWriter,request:&[u8],write_timeout:Duration,now:&mut dyn FnMut()->Instant)->Result<usize,WebDriverBiDiWebSocketOpeningWriteError>{let deadline=now()+write_timeout;let mut bytes_written=0;while bytes_written<request.len(){let remaining=deadline.saturating_duration_since(now());if remaining.is_zero(){return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded{bytes_written});}writer.set_write_timeout(remaining).map_err(|source|WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutConfigurationFailed{bytes_written,source})?;match writer.write_request_bytes(&request[bytes_written..]){Ok(0)=>return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteZero{bytes_written}),Ok(count)=>{bytes_written+=count;if deadline.saturating_duration_since(now()).is_zero(){return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded{bytes_written});}},Err(source) if source.kind()==io::ErrorKind::Interrupted=>continue,Err(source) if matches!(source.kind(),io::ErrorKind::TimedOut|io::ErrorKind::WouldBlock)=>return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteTimedOut{bytes_written,source}),Err(source)=>return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteFailed{bytes_written,source})}}writer.clear_write_timeout().map_err(|source|WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutCleanupFailed{bytes_written,source})?;Ok(bytes_written)}

#[cfg(test)]
mod opening_write_tests {
    use super::*;

    #[test]
    fn close_status_code_validation_covers_wire_bounds_and_forbidden_sentinels() {
        for code in [999_u16, 1005, 1006, 1015, 5000] {
            assert!(!is_valid_close_status_code(code));
        }
        for code in [1000_u16, 3000, 4000, 4999] {
            assert!(is_valid_close_status_code(code));
        }
    }
}
