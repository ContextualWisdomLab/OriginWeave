use std::{
    error::Error,
    fmt,
    io::{self, Read, Write},
    net::TcpStream,
    thread,
    time::{Duration, Instant},
};

use originweave_core::VerifiedWebDriverBiDiSocketPeer;

use crate::{
    WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionEvidence,
    webdriver_bidi_websocket_handshake as handshake,
};

const MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES: usize = 1024 * 1024;
const MAX_WEBSOCKET_CONTROL_FRAME_PAYLOAD_BYTES: usize = 125;
const REUSED_CLIENT_MASK_KEY_REASON: &str =
    "client masking key was reused for consecutive frames on this established WebSocket";

/// Maximum payload bytes admitted for one WebSocket frame.
pub const MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE: usize = MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES;

/// Maximum wall-clock budget accepted for one bounded WebSocket frame I/O operation.
pub const MAX_WEBSOCKET_FRAME_TIMEOUT: Duration = Duration::from_secs(5);

/// Caller-supplied RFC 6455 mask key for one client-to-server frame.
///
/// RFC 6455 requires a fresh unpredictable four-byte key for every client frame. OriginWeave does
/// not invent an entropy source here: callers must obtain each value from an approved randomness
/// source. Debug output is deliberately redacted so diagnostics cannot disclose masking entropy.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketMaskKey([u8; 4]);

impl fmt::Debug for WebDriverBiDiWebSocketMaskKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted WebSocket masking key>")
    }
}

impl WebDriverBiDiWebSocketMaskKey {
    /// Admit one four-byte caller-supplied frame masking key.
    #[must_use]
    pub const fn new(value: [u8; 4]) -> Self {
        Self(value)
    }

    /// Borrow the exact four-byte key used by the reviewed framing boundary.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
}

#[derive(Default)]
struct ClientMaskKeyHistory {
    previous_key: Option<[u8; 4]>,
}

impl ClientMaskKeyHistory {
    fn reserve(
        &mut self,
        masking_key: WebDriverBiDiWebSocketMaskKey,
    ) -> Result<(), WebDriverBiDiWebSocketFrameError> {
        let key = *masking_key.as_bytes();
        if self.previous_key == Some(key) {
            return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
                reason: REUSED_CLIENT_MASK_KEY_REASON,
            });
        }
        self.previous_key = Some(key);
        Ok(())
    }
}

/// Inert RFC 6455 opening request bound to one already-verified plain BiDi TCP connection.
pub struct WebDriverBiDiWebSocketHandshakePlan(handshake::WebDriverBiDiWebSocketHandshakePlan);

impl fmt::Debug for WebDriverBiDiWebSocketHandshakePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl WebDriverBiDiWebSocketHandshakePlan {
    /// Bind one canonical opening request to an already-verified plain BiDi TCP connection.
    pub fn new(
        connection: WebDriverBiDiTcpConnection,
        client_key: handshake::WebDriverBiDiWebSocketClientKey,
    ) -> Result<Self, handshake::WebDriverBiDiWebSocketHandshakeError> {
        handshake::WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key).map(Self)
    }

    /// Borrow the exact serialized RFC 6455 opening-request bytes.
    #[must_use]
    pub fn request_bytes(&self) -> &[u8] {
        self.0.request_bytes()
    }

    /// Borrow the exact client key required to correlate the opening response.
    #[must_use]
    pub const fn client_key(&self) -> &handshake::WebDriverBiDiWebSocketClientKey {
        self.0.client_key()
    }

    /// Borrow the exact peer/session evidence already verified before request construction.
    #[must_use]
    pub const fn verified_peer(&self) -> &VerifiedWebDriverBiDiSocketPeer {
        self.0.verified_peer()
    }

    /// Write the complete bounded opening request on the exact verified stream within one deadline.
    pub fn write_opening_request(
        self,
        write_timeout: Duration,
    ) -> Result<
        WebDriverBiDiWebSocketOpeningRequestSent,
        handshake::WebDriverBiDiWebSocketOpeningWriteError,
    > {
        self.0
            .write_opening_request(write_timeout)
            .map(WebDriverBiDiWebSocketOpeningRequestSent)
    }
}

/// A live verified stream after the complete client WebSocket opening request has been written.
pub struct WebDriverBiDiWebSocketOpeningRequestSent(
    handshake::WebDriverBiDiWebSocketOpeningRequestSent,
);

impl fmt::Debug for WebDriverBiDiWebSocketOpeningRequestSent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl WebDriverBiDiWebSocketOpeningRequestSent {
    /// Borrow the exact verified transport evidence retained with this live stream.
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        self.0.transport_evidence()
    }

    /// Borrow the exact client key required to validate the later server accept value.
    #[must_use]
    pub const fn client_key(&self) -> &handshake::WebDriverBiDiWebSocketClientKey {
        self.0.client_key()
    }

    /// Return the exact number of opening-request bytes written before success was emitted.
    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.0.request_byte_count()
    }

    /// Return the total write deadline configured for this opening request.
    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.0.write_timeout()
    }

    /// Read and validate the bounded RFC 6455 server opening response on this exact stream.
    pub fn read_opening_response(
        self,
        response_timeout: Duration,
    ) -> Result<
        WebDriverBiDiWebSocketEstablished,
        handshake::WebDriverBiDiWebSocketHandshakeResponseError,
    > {
        self.0.read_opening_response(response_timeout).map(|raw| {
            WebDriverBiDiWebSocketEstablished {
                raw,
                client_mask_keys: ClientMaskKeyHistory::default(),
            }
        })
    }
}

/// A live verified stream after both RFC 6455 opening messages were validated.
///
/// Client text and Pong writes are masked and bounded. The caller remains responsible for fresh
/// cryptographically strong masking keys; OriginWeave additionally rejects adjacent key repetition
/// as a bounded stuck-randomness defense without imposing impossible lifetime uniqueness on a
/// 32-bit RFC 6455 value. Frame I/O never grants browser, page, policy, origin, or Agent authority.
pub struct WebDriverBiDiWebSocketEstablished {
    raw: handshake::WebDriverBiDiWebSocketEstablished,
    client_mask_keys: ClientMaskKeyHistory,
}

impl fmt::Debug for WebDriverBiDiWebSocketEstablished {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(formatter)
    }
}

impl WebDriverBiDiWebSocketEstablished {
    /// Borrow the exact verified transport evidence retained with this live stream.
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        self.raw.transport_evidence()
    }

    /// Borrow the exact client key correlated with the validated server accept value.
    #[must_use]
    pub const fn client_key(&self) -> &handshake::WebDriverBiDiWebSocketClientKey {
        self.raw.client_key()
    }

    /// Return the validated HTTP status code, currently always `101` on success.
    #[must_use]
    pub const fn response_status(&self) -> u16 {
        self.raw.response_status()
    }

    /// Return the number of HTTP opening-response bytes consumed through its header terminator.
    #[must_use]
    pub const fn response_byte_count(&self) -> usize {
        self.raw.response_byte_count()
    }

    /// Return the total response deadline configured for this opening response.
    #[must_use]
    pub const fn response_timeout(&self) -> Duration {
        self.raw.response_timeout()
    }

    /// Return the number of request bytes written before the response was read.
    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.raw.request_byte_count()
    }

    /// Return the total write deadline configured for the preceding opening request.
    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.raw.write_timeout()
    }

    /// Write one final masked UTF-8 text frame on this verified stream.
    ///
    /// The state is consumed. Invalid bounds, adjacent masking-key reuse, partial writes, deadline
    /// expiry, I/O failure, and timeout-cleanup failure return no reusable stream. No retry changes
    /// destination or connection authority.
    pub fn write_text_frame(
        mut self,
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
        self.client_mask_keys.reserve(masking_key)?;
        let frame = serialize_client_frame(0x1, text.as_bytes(), masking_key);
        let mut now = Instant::now;
        write_frame_with_clock(&mut self.raw.stream, &frame, frame_timeout, &mut now).map(|_| self)
    }

    /// Write one final masked RFC 6455 Pong control frame on this verified stream.
    ///
    /// Payloads above 125 bytes fail closed. The same adjacent masking-key guard used for text
    /// frames applies across frame types so switching to Pong cannot bypass stuck-key detection.
    pub fn write_pong_frame(
        mut self,
        payload: &[u8],
        masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<Self, WebDriverBiDiWebSocketFrameError> {
        validate_frame_timeout(frame_timeout)?;
        if payload.len() > MAX_WEBSOCKET_CONTROL_FRAME_PAYLOAD_BYTES {
            return Err(WebDriverBiDiWebSocketFrameError::FrameTooLarge {
                payload_bytes: payload.len(),
                maximum_bytes: MAX_WEBSOCKET_CONTROL_FRAME_PAYLOAD_BYTES,
            });
        }
        self.client_mask_keys.reserve(masking_key)?;
        let frame = serialize_client_frame(0xa, payload, masking_key);
        let mut now = Instant::now;
        write_frame_with_clock(&mut self.raw.stream, &frame, frame_timeout, &mut now).map(|_| self)
    }

    /// Read one bounded RFC 6455 frame from this verified stream.
    ///
    /// Server frames must be unmasked. Reserved bits/opcodes, non-minimal lengths, oversized
    /// payloads, fragmented control frames, malformed Close payloads, forbidden Close status codes,
    /// deadline expiry, and I/O failures fail closed. Data/continuation frames are returned one at a
    /// time so a later message layer can own fragmentation and JSON semantics.
    pub fn read_frame(
        mut self,
        frame_timeout: Duration,
    ) -> Result<(Self, WebDriverBiDiWebSocketFrame), WebDriverBiDiWebSocketFrameError> {
        validate_frame_timeout(frame_timeout)?;
        let mut now = Instant::now;
        read_frame_with_clock(&mut self.raw.stream, frame_timeout, &mut now)
            .and_then(|frame| validate_close_frame(&frame).map(|()| (self, frame)))
    }
}

/// One validated bounded WebSocket frame received from the established peer.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketFrame {
    fin: bool,
    opcode: u8,
    payload: Vec<u8>,
}

impl WebDriverBiDiWebSocketFrame {
    /// Return whether this is the final frame in its message.
    #[must_use]
    pub const fn fin(&self) -> bool {
        self.fin
    }

    /// Return the RFC 6455 opcode without interpreting application semantics.
    #[must_use]
    pub const fn opcode(&self) -> u8 {
        self.opcode
    }

    /// Borrow the bounded unmasked application payload.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// Fail-closed errors while reading or writing one bounded WebSocket frame.
#[derive(Debug)]
pub enum WebDriverBiDiWebSocketFrameError {
    /// The requested frame I/O deadline was zero or above the reviewed resource ceiling.
    InvalidFrameTimeout {
        /// Rejected caller-supplied deadline.
        frame_timeout: Duration,
        /// Maximum reviewed deadline accepted by this boundary.
        maximum_timeout: Duration,
    },
    /// The frame payload exceeded the reviewed memory ceiling.
    FrameTooLarge {
        /// Rejected payload length in bytes.
        payload_bytes: usize,
        /// Maximum payload length admitted by this boundary.
        maximum_bytes: usize,
    },
    /// Applying an operation-local bounded read timeout failed.
    FrameReadModeConfigurationFailed {
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A bounded socket read timed out before the frame was complete.
    FrameReadTimedOut {
        /// Number of frame bytes consumed before timeout.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A non-recoverable socket read or read-timeout cleanup failed.
    FrameReadFailed {
        /// Number of frame bytes consumed before failure.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// The peer ended the stream before the frame was complete.
    FrameEnded {
        /// Number of frame bytes consumed before EOF.
        bytes_read: usize,
    },
    /// The frame violated the RFC 6455 framing or control-frame contract.
    MalformedFrame {
        /// Stable, non-secret reason for rejection.
        reason: &'static str,
    },
    /// Applying the operation-local write timeout failed.
    FrameWriteModeConfigurationFailed {
        /// Number of frame bytes already written before configuration failed.
        bytes_written: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A bounded socket write timed out before the frame was complete.
    FrameWriteTimedOut {
        /// Number of frame bytes written before timeout.
        bytes_written: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A non-recoverable socket write failed before the frame was complete.
    FrameWriteFailed {
        /// Number of frame bytes written before failure.
        bytes_written: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// The stream reported zero progress before the frame was complete.
    FrameWriteZero {
        /// Number of frame bytes written before zero progress.
        bytes_written: usize,
    },
    /// Clearing the temporary write timeout failed before handoff.
    FrameWriteCleanupFailed {
        /// Underlying operating-system error.
        source: io::Error,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFrameTimeout { .. } => formatter
                .write_str("WebDriver BiDi WebSocket frame timeout is outside the reviewed bound"),
            Self::FrameTooLarge { .. } => {
                formatter.write_str("WebDriver BiDi WebSocket frame payload exceeded its bound")
            }
            Self::FrameReadModeConfigurationFailed { .. } => {
                formatter.write_str("failed to configure bounded WebSocket frame reads")
            }
            Self::FrameReadTimedOut { .. } => {
                formatter.write_str("WebDriver BiDi WebSocket frame read timed out")
            }
            Self::FrameReadFailed { .. } => {
                formatter.write_str("WebDriver BiDi WebSocket frame read failed")
            }
            Self::FrameEnded { .. } => {
                formatter.write_str("WebDriver BiDi WebSocket peer ended the frame stream")
            }
            Self::MalformedFrame { .. } => {
                formatter.write_str("WebDriver BiDi WebSocket frame was malformed")
            }
            Self::FrameWriteModeConfigurationFailed { .. } => {
                formatter.write_str("failed to configure bounded WebSocket frame writes")
            }
            Self::FrameWriteTimedOut { .. } => {
                formatter.write_str("WebDriver BiDi WebSocket frame write timed out")
            }
            Self::FrameWriteFailed { .. } => {
                formatter.write_str("WebDriver BiDi WebSocket frame write failed")
            }
            Self::FrameWriteZero { .. } => {
                formatter.write_str("WebDriver BiDi WebSocket frame write made no progress")
            }
            Self::FrameWriteCleanupFailed { .. } => {
                formatter.write_str("failed to clear the WebDriver BiDi WebSocket frame timeout")
            }
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

fn validate_frame_timeout(frame_timeout: Duration) -> Result<(), WebDriverBiDiWebSocketFrameError> {
    if frame_timeout.is_zero() || frame_timeout > MAX_WEBSOCKET_FRAME_TIMEOUT {
        return Err(WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
            frame_timeout,
            maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
        });
    }
    Ok(())
}

fn serialize_client_frame(
    opcode: u8,
    payload: &[u8],
    masking_key: WebDriverBiDiWebSocketMaskKey,
) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len() + 14);
    frame.push(0x80 | opcode);
    match payload.len() {
        0..=125 => frame.push(0x80 | payload.len() as u8),
        126..=65_535 => {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        }
        length => {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(length as u64).to_be_bytes());
        }
    }
    frame.extend_from_slice(masking_key.as_bytes());
    frame.extend(
        payload.iter().enumerate().map(|(index, byte)| {
            byte ^ masking_key.as_bytes()[index % masking_key.as_bytes().len()]
        }),
    );
    frame
}

trait FrameIo {
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
    fn read_frame_bytes(&mut self, bytes: &mut [u8]) -> io::Result<usize>;
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
    fn write_frame_bytes(&mut self, bytes: &[u8]) -> io::Result<usize>;
}

impl FrameIo for TcpStream {
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        TcpStream::set_read_timeout(self, timeout)
    }

    fn read_frame_bytes(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.read(bytes)
    }

    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        TcpStream::set_write_timeout(self, timeout)
    }

    fn write_frame_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.write(bytes)
    }
}

fn write_frame_with_clock(
    writer: &mut dyn FrameIo,
    frame: &[u8],
    frame_timeout: Duration,
    now: &mut dyn FnMut() -> Instant,
) -> Result<usize, WebDriverBiDiWebSocketFrameError> {
    let deadline = now() + frame_timeout;
    let mut bytes_written = 0;
    while bytes_written < frame.len() {
        let remaining = deadline.saturating_duration_since(now());
        if remaining.is_zero() {
            return Err(WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                bytes_written,
                source: io::Error::new(io::ErrorKind::TimedOut, "frame write deadline elapsed"),
            });
        }
        writer
            .set_write_timeout(Some(remaining))
            .map_err(|source| {
                WebDriverBiDiWebSocketFrameError::FrameWriteModeConfigurationFailed {
                    bytes_written,
                    source,
                }
            })?;
        match writer.write_frame_bytes(&frame[bytes_written..]) {
            Ok(0) => {
                return Err(WebDriverBiDiWebSocketFrameError::FrameWriteZero { bytes_written });
            }
            Ok(written) => {
                bytes_written += written;
                if deadline.saturating_duration_since(now()).is_zero() {
                    return Err(WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                        bytes_written,
                        source: io::Error::new(
                            io::ErrorKind::TimedOut,
                            "frame write completed after deadline",
                        ),
                    });
                }
            }
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source)
                if matches!(
                    source.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                if deadline.saturating_duration_since(now()).is_zero() {
                    return Err(WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                        bytes_written,
                        source,
                    });
                }
                thread::sleep(Duration::from_millis(1));
            }
            Err(source) => {
                return Err(WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
                    bytes_written,
                    source,
                });
            }
        }
    }
    writer
        .set_write_timeout(None)
        .map_err(|source| WebDriverBiDiWebSocketFrameError::FrameWriteCleanupFailed { source })?;
    Ok(bytes_written)
}

fn read_exact_with_clock(
    reader: &mut dyn FrameIo,
    destination: &mut [u8],
    bytes_read: &mut usize,
    deadline: Instant,
    now: &mut dyn FnMut() -> Instant,
) -> Result<(), WebDriverBiDiWebSocketFrameError> {
    let mut offset = 0;
    while offset < destination.len() {
        let remaining = deadline.saturating_duration_since(now());
        if remaining.is_zero() {
            return Err(WebDriverBiDiWebSocketFrameError::FrameReadTimedOut {
                bytes_read: *bytes_read,
                source: io::Error::new(io::ErrorKind::TimedOut, "frame read deadline elapsed"),
            });
        }
        reader.set_read_timeout(Some(remaining)).map_err(|source| {
            WebDriverBiDiWebSocketFrameError::FrameReadModeConfigurationFailed { source }
        })?;
        match reader.read_frame_bytes(&mut destination[offset..]) {
            Ok(0) => {
                return Err(WebDriverBiDiWebSocketFrameError::FrameEnded {
                    bytes_read: *bytes_read,
                });
            }
            Ok(read) if read > destination.len() - offset => {
                return Err(WebDriverBiDiWebSocketFrameError::FrameReadFailed {
                    bytes_read: *bytes_read,
                    source: io::Error::new(
                        io::ErrorKind::InvalidData,
                        "frame reader returned more bytes than requested",
                    ),
                });
            }
            Ok(read) => {
                offset += read;
                *bytes_read += read;
                if deadline.saturating_duration_since(now()).is_zero() {
                    return Err(WebDriverBiDiWebSocketFrameError::FrameReadTimedOut {
                        bytes_read: *bytes_read,
                        source: io::Error::new(
                            io::ErrorKind::TimedOut,
                            "frame read completed after deadline",
                        ),
                    });
                }
            }
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source)
                if matches!(
                    source.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                if deadline.saturating_duration_since(now()).is_zero() {
                    return Err(WebDriverBiDiWebSocketFrameError::FrameReadTimedOut {
                        bytes_read: *bytes_read,
                        source,
                    });
                }
                thread::sleep(Duration::from_millis(1));
            }
            Err(source) => {
                return Err(WebDriverBiDiWebSocketFrameError::FrameReadFailed {
                    bytes_read: *bytes_read,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn read_frame_with_clock(
    reader: &mut dyn FrameIo,
    frame_timeout: Duration,
    now: &mut dyn FnMut() -> Instant,
) -> Result<WebDriverBiDiWebSocketFrame, WebDriverBiDiWebSocketFrameError> {
    let deadline = now() + frame_timeout;
    let mut bytes_read = 0;
    let mut header = [0_u8; 2];
    read_exact_with_clock(reader, &mut header, &mut bytes_read, deadline, now)?;

    let first = header[0];
    let second = header[1];
    if first & 0x70 != 0 {
        return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "reserved frame bits are not negotiated",
        });
    }
    let fin = first & 0x80 != 0;
    let opcode = first & 0x0f;
    match opcode {
        0x0..=0x2 => {}
        0x8..=0xa => {
            if !fin {
                return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
                    reason: "control frames must not be fragmented",
                });
            }
        }
        _ => {
            return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
                reason: "frame opcode is reserved or unsupported",
            });
        }
    }
    if second & 0x80 != 0 {
        return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "server-to-client frames must not be masked",
        });
    }

    let length_code = second & 0x7f;
    let payload_length = match length_code {
        0..=125 => u64::from(length_code),
        126 => {
            let mut extended = [0_u8; 2];
            read_exact_with_clock(reader, &mut extended, &mut bytes_read, deadline, now)?;
            let length = u64::from(u16::from_be_bytes(extended));
            if length < 126 {
                return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
                    reason: "frame length encoding is not minimal",
                });
            }
            length
        }
        _ => {
            let mut extended = [0_u8; 8];
            read_exact_with_clock(reader, &mut extended, &mut bytes_read, deadline, now)?;
            if extended[0] & 0x80 != 0 {
                return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
                    reason: "frame length uses the reserved high bit",
                });
            }
            let length = u64::from_be_bytes(extended);
            if length < 65_536 {
                return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
                    reason: "frame length encoding is not minimal",
                });
            }
            length
        }
    };
    if payload_length > MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES as u64 {
        return Err(WebDriverBiDiWebSocketFrameError::FrameTooLarge {
            payload_bytes: payload_length.min(usize::MAX as u64) as usize,
            maximum_bytes: MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES,
        });
    }
    if opcode >= 0x8 && payload_length > MAX_WEBSOCKET_CONTROL_FRAME_PAYLOAD_BYTES as u64 {
        return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "control frame payload exceeds 125 bytes",
        });
    }

    let mut payload = vec![0_u8; payload_length as usize];
    read_exact_with_clock(reader, &mut payload, &mut bytes_read, deadline, now)?;
    reader.set_read_timeout(None).map_err(|source| {
        WebDriverBiDiWebSocketFrameError::FrameReadFailed { bytes_read, source }
    })?;
    Ok(WebDriverBiDiWebSocketFrame {
        fin,
        opcode,
        payload,
    })
}

fn validate_close_frame(
    frame: &WebDriverBiDiWebSocketFrame,
) -> Result<(), WebDriverBiDiWebSocketFrameError> {
    if frame.opcode() != 0x8 {
        return Ok(());
    }
    if frame.payload().len() == 1 {
        return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "Close frame payload must be empty or begin with a two-byte status code",
        });
    }
    if frame.payload().len() < 2 {
        return Ok(());
    }
    if std::str::from_utf8(&frame.payload()[2..]).is_err() {
        return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "Close frame reason is not valid UTF-8",
        });
    }
    let status_code = u16::from_be_bytes([frame.payload()[0], frame.payload()[1]]);
    if !(1000..=4999).contains(&status_code) || matches!(status_code, 1004 | 1005 | 1006 | 1015) {
        return Err(WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "Close frame status code is not valid on the wire",
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Clone, Debug)]
    enum ReadAction {
        Bytes(Vec<u8>),
        Count(usize),
        End,
        Error(io::ErrorKind),
    }

    #[derive(Clone, Copy, Debug)]
    enum WriteAction {
        Count(usize),
        Error(io::ErrorKind),
    }

    #[derive(Debug)]
    struct FakeIo {
        reads: VecDeque<ReadAction>,
        writes: VecDeque<WriteAction>,
        read_mode_error: Option<io::ErrorKind>,
        read_cleanup_error: Option<io::ErrorKind>,
        write_mode_error: Option<io::ErrorKind>,
        write_cleanup_error: Option<io::ErrorKind>,
    }

    impl FakeIo {
        fn new() -> Self {
            Self {
                reads: VecDeque::new(),
                writes: VecDeque::new(),
                read_mode_error: None,
                read_cleanup_error: None,
                write_mode_error: None,
                write_cleanup_error: None,
            }
        }
    }

    impl FrameIo for FakeIo {
        fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
            let error = if timeout.is_some() {
                self.read_mode_error
            } else {
                self.read_cleanup_error
            };
            error.map_or(Ok(()), |kind| Err(io::Error::from(kind)))
        }

        fn read_frame_bytes(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            match self.reads.pop_front().unwrap_or(ReadAction::End) {
                ReadAction::Bytes(value) => {
                    let count = value.len().min(bytes.len());
                    bytes[..count].copy_from_slice(&value[..count]);
                    if count < value.len() {
                        self.reads
                            .push_front(ReadAction::Bytes(value[count..].to_vec()));
                    }
                    Ok(count)
                }
                ReadAction::Count(count) => Ok(count),
                ReadAction::End => Ok(0),
                ReadAction::Error(kind) => Err(io::Error::from(kind)),
            }
        }

        fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
            let error = if timeout.is_some() {
                self.write_mode_error
            } else {
                self.write_cleanup_error
            };
            error.map_or(Ok(()), |kind| Err(io::Error::from(kind)))
        }

        fn write_frame_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
            match self
                .writes
                .pop_front()
                .unwrap_or(WriteAction::Count(bytes.len()))
            {
                WriteAction::Count(count) => Ok(count.min(bytes.len())),
                WriteAction::Error(kind) => Err(io::Error::from(kind)),
            }
        }
    }

    fn frame_from_bytes(
        bytes: &[u8],
    ) -> Result<WebDriverBiDiWebSocketFrame, WebDriverBiDiWebSocketFrameError> {
        let start = Instant::now();
        let mut io = FakeIo::new();
        io.reads.push_back(ReadAction::Bytes(bytes.to_vec()));
        let mut now = || start;
        read_frame_with_clock(&mut io, Duration::from_secs(1), &mut now)
    }

    fn assert_error_variant<T>(
        result: Result<T, WebDriverBiDiWebSocketFrameError>,
        expected: WebDriverBiDiWebSocketFrameError,
    ) {
        let actual = result.err().expect("expected WebSocket frame error");
        assert_eq!(
            std::mem::discriminant(&actual),
            std::mem::discriminant(&expected)
        );
    }

    fn malformed_reason(error: &WebDriverBiDiWebSocketFrameError) -> Option<&'static str> {
        match error {
            WebDriverBiDiWebSocketFrameError::MalformedFrame { reason } => Some(*reason),
            _ => None,
        }
    }

    #[test]
    fn mask_key_debug_and_history_preserve_entropy_contract() {
        let first = WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]);
        let second = WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]);
        assert_eq!(first.as_bytes(), &[1, 2, 3, 4]);
        assert_eq!(format!("{first:?}"), "<redacted WebSocket masking key>");
        let mut history = ClientMaskKeyHistory::default();
        assert!(history.reserve(first).is_ok());
        let reused = history.reserve(first).expect_err("reused key must fail");
        assert_eq!(
            malformed_reason(&reused),
            Some(REUSED_CLIENT_MASK_KEY_REASON)
        );
        let non_malformed = WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
            frame_timeout: Duration::ZERO,
            maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
        };
        assert_eq!(malformed_reason(&non_malformed), None);
        assert!(history.reserve(second).is_ok());
        assert!(history.reserve(first).is_ok());
    }

    #[test]
    fn serializer_uses_minimal_lengths_and_masks_payloads() {
        let key = WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]);
        let small = serialize_client_frame(0x1, b"abc", key);
        assert_eq!(&small[..6], &[0x81, 0x83, 1, 2, 3, 4]);
        assert_eq!(&small[6..], &[b'a' ^ 1, b'b' ^ 2, b'c' ^ 3]);

        let medium_payload = [b'x'; 126];
        let medium = serialize_client_frame(0x1, &medium_payload, key);
        assert_eq!(&medium[..4], &[0x81, 0xfe, 0, 126]);
        let large_payload = vec![b'x'; 65_536].into_boxed_slice();
        let large = serialize_client_frame(0x1, &large_payload, key);
        assert_eq!(large[0], 0x81);
        assert_eq!(large[1], 0xff);
        assert_eq!(&large[2..10], &65_536_u64.to_be_bytes());
        let pong = serialize_client_frame(0xa, b"ok", key);
        assert_eq!(pong[0], 0x8a);
    }

    #[test]
    fn frame_timeout_validation_is_bounded() {
        assert!(validate_frame_timeout(Duration::from_nanos(1)).is_ok());
        assert_error_variant(
            validate_frame_timeout(Duration::ZERO),
            WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
                frame_timeout: Duration::ZERO,
                maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
            },
        );
        assert_error_variant(
            validate_frame_timeout(MAX_WEBSOCKET_FRAME_TIMEOUT + Duration::from_nanos(1)),
            WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
                frame_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT + Duration::from_nanos(1),
                maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
            },
        );
    }

    #[test]
    fn bounded_writer_covers_progress_retry_deadline_and_failures() {
        let start = Instant::now();

        let mut partial = FakeIo::new();
        partial
            .writes
            .extend([WriteAction::Count(2), WriteAction::Count(4)]);
        let mut now = || start;
        assert_eq!(
            write_frame_with_clock(&mut partial, b"abcdef", Duration::from_secs(1), &mut now)
                .expect("partial frame writes must finish"),
            6
        );

        let mut interrupted = FakeIo::new();
        interrupted.writes.extend([
            WriteAction::Error(io::ErrorKind::Interrupted),
            WriteAction::Count(6),
        ]);
        let mut now = || start;
        assert!(
            write_frame_with_clock(
                &mut interrupted,
                b"abcdef",
                Duration::from_secs(1),
                &mut now
            )
            .is_ok()
        );

        let mut would_block = FakeIo::new();
        would_block.writes.extend([
            WriteAction::Error(io::ErrorKind::WouldBlock),
            WriteAction::Count(6),
        ]);
        let mut now = || start;
        assert!(
            write_frame_with_clock(
                &mut would_block,
                b"abcdef",
                Duration::from_secs(1),
                &mut now
            )
            .is_ok()
        );

        let mut zero = FakeIo::new();
        zero.writes.push_back(WriteAction::Count(0));
        let mut now = || start;
        assert_error_variant(
            write_frame_with_clock(&mut zero, b"x", Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameWriteZero { bytes_written: 0 },
        );

        let mut configure = FakeIo::new();
        configure.write_mode_error = Some(io::ErrorKind::PermissionDenied);
        let mut now = || start;
        assert_error_variant(
            write_frame_with_clock(&mut configure, b"x", Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameWriteModeConfigurationFailed {
                bytes_written: 0,
                source: io::Error::from(io::ErrorKind::PermissionDenied),
            },
        );

        let mut failed = FakeIo::new();
        failed
            .writes
            .push_back(WriteAction::Error(io::ErrorKind::BrokenPipe));
        let mut now = || start;
        assert_error_variant(
            write_frame_with_clock(&mut failed, b"x", Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
                bytes_written: 0,
                source: io::Error::from(io::ErrorKind::BrokenPipe),
            },
        );

        let mut timed = FakeIo::new();
        timed
            .writes
            .push_back(WriteAction::Error(io::ErrorKind::TimedOut));
        let mut times = VecDeque::from([start, start, start + Duration::from_secs(1)]);
        let mut now = || times.pop_front().unwrap_or(start + Duration::from_secs(1));
        assert_error_variant(
            write_frame_with_clock(&mut timed, b"x", Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                bytes_written: 0,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
        );

        let mut before = FakeIo::new();
        let mut times = VecDeque::from([start, start + Duration::from_secs(1)]);
        let mut now = || times.pop_front().unwrap_or(start + Duration::from_secs(1));
        assert_error_variant(
            write_frame_with_clock(&mut before, b"x", Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                bytes_written: 0,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
        );

        let mut late = FakeIo::new();
        late.writes.push_back(WriteAction::Count(1));
        let mut times = VecDeque::from([start, start, start + Duration::from_secs(1)]);
        let mut now = || times.pop_front().unwrap_or(start + Duration::from_secs(1));
        assert_error_variant(
            write_frame_with_clock(&mut late, b"x", Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                bytes_written: 1,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
        );

        let mut cleanup = FakeIo::new();
        cleanup.write_cleanup_error = Some(io::ErrorKind::PermissionDenied);
        let mut now = || start;
        assert_error_variant(
            write_frame_with_clock(&mut cleanup, b"x", Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameWriteCleanupFailed {
                source: io::Error::from(io::ErrorKind::PermissionDenied),
            },
        );
    }

    #[test]
    fn bounded_reader_accepts_supported_frame_shapes() {
        let text = frame_from_bytes(&[0x81, 0x01, b'x']).expect("valid text frame");
        assert!(text.fin());
        assert_eq!(text.opcode(), 1);
        assert_eq!(text.payload(), b"x");

        let continuation = frame_from_bytes(&[0x00, 0x00]).expect("valid continuation");
        assert!(!continuation.fin());
        assert_eq!(continuation.opcode(), 0);

        let ping = frame_from_bytes(&[0x89, 0x00]).expect("valid ping");
        assert_eq!(ping.opcode(), 9);

        let mut extended_16 = vec![0x81, 126, 0, 126];
        extended_16.extend(vec![b'x'; 126]);
        assert_eq!(
            frame_from_bytes(&extended_16)
                .expect("valid 16-bit frame")
                .payload()
                .len(),
            126
        );

        let mut extended_64 = vec![0x81, 127];
        extended_64.extend_from_slice(&65_536_u64.to_be_bytes());
        extended_64.extend(vec![b'x'; 65_536]);
        assert_eq!(
            frame_from_bytes(&extended_64)
                .expect("valid 64-bit frame")
                .payload()
                .len(),
            65_536
        );
    }

    #[test]
    fn bounded_reader_rejects_protocol_violations() {
        let mut oversized = vec![0x81, 127];
        oversized
            .extend_from_slice(&((MAX_WEBSOCKET_FRAME_PAYLOAD_BYTES as u64) + 1).to_be_bytes());
        for bytes in [
            vec![0xc1, 0],
            vec![0x09, 0],
            vec![0x83, 0],
            vec![0x81, 0x80],
            vec![0x81, 126, 0, 1],
            vec![0x81, 127, 0x80, 0, 0, 0, 0, 0, 0, 0],
            vec![0x81, 127, 0, 0, 0, 0, 0, 0, 0xff, 0xff],
            vec![0x89, 126, 0, 126],
            oversized,
        ] {
            assert!(frame_from_bytes(&bytes).is_err());
        }
    }

    #[test]
    fn bounded_reader_covers_io_deadline_and_cleanup_failures() {
        let start = Instant::now();

        let mut ended = FakeIo::new();
        ended.reads.push_back(ReadAction::End);
        let mut now = || start;
        assert_error_variant(
            read_frame_with_clock(&mut ended, Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameEnded { bytes_read: 0 },
        );

        let mut impossible_count = FakeIo::new();
        impossible_count.reads.push_back(ReadAction::Count(3));
        let mut now = || start;
        assert_error_variant(
            read_frame_with_clock(&mut impossible_count, Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameReadFailed {
                bytes_read: 0,
                source: io::Error::from(io::ErrorKind::InvalidData),
            },
        );

        let mut configure = FakeIo::new();
        configure.read_mode_error = Some(io::ErrorKind::PermissionDenied);
        let mut now = || start;
        assert_error_variant(
            read_frame_with_clock(&mut configure, Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameReadModeConfigurationFailed {
                source: io::Error::from(io::ErrorKind::PermissionDenied),
            },
        );

        let mut interrupted = FakeIo::new();
        interrupted.reads.extend([
            ReadAction::Error(io::ErrorKind::Interrupted),
            ReadAction::Bytes(vec![0x81, 0]),
        ]);
        let mut now = || start;
        assert!(read_frame_with_clock(&mut interrupted, Duration::from_secs(1), &mut now).is_ok());

        let mut would_block = FakeIo::new();
        would_block.reads.extend([
            ReadAction::Error(io::ErrorKind::WouldBlock),
            ReadAction::Bytes(vec![0x81, 0]),
        ]);
        let mut now = || start;
        assert!(read_frame_with_clock(&mut would_block, Duration::from_secs(1), &mut now).is_ok());

        let mut failed = FakeIo::new();
        failed
            .reads
            .push_back(ReadAction::Error(io::ErrorKind::BrokenPipe));
        let mut now = || start;
        assert_error_variant(
            read_frame_with_clock(&mut failed, Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameReadFailed {
                bytes_read: 0,
                source: io::Error::from(io::ErrorKind::BrokenPipe),
            },
        );

        let mut timed = FakeIo::new();
        timed
            .reads
            .push_back(ReadAction::Error(io::ErrorKind::TimedOut));
        let mut times = VecDeque::from([start, start, start + Duration::from_secs(1)]);
        let mut now = || times.pop_front().unwrap_or(start + Duration::from_secs(1));
        assert_error_variant(
            read_frame_with_clock(&mut timed, Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameReadTimedOut {
                bytes_read: 0,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
        );

        let mut before = FakeIo::new();
        before.reads.push_back(ReadAction::Bytes(vec![0x81, 0]));
        let mut times = VecDeque::from([start, start + Duration::from_secs(1)]);
        let mut now = || times.pop_front().unwrap_or(start + Duration::from_secs(1));
        assert_error_variant(
            read_frame_with_clock(&mut before, Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameReadTimedOut {
                bytes_read: 0,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
        );

        let mut late = FakeIo::new();
        late.reads.push_back(ReadAction::Bytes(vec![0x81, 0]));
        let mut times = VecDeque::from([start, start, start + Duration::from_secs(1)]);
        let mut now = || times.pop_front().unwrap_or(start + Duration::from_secs(1));
        assert_error_variant(
            read_frame_with_clock(&mut late, Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameReadTimedOut {
                bytes_read: 2,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
        );

        let mut cleanup = FakeIo::new();
        cleanup.reads.push_back(ReadAction::Bytes(vec![0x81, 0]));
        cleanup.read_cleanup_error = Some(io::ErrorKind::PermissionDenied);
        let mut now = || start;
        assert_error_variant(
            read_frame_with_clock(&mut cleanup, Duration::from_secs(1), &mut now),
            WebDriverBiDiWebSocketFrameError::FrameReadFailed {
                bytes_read: 2,
                source: io::Error::from(io::ErrorKind::PermissionDenied),
            },
        );

        for prefix in [vec![0x81, 126], vec![0x81, 127], vec![0x81, 1]] {
            let mut truncated = FakeIo::new();
            truncated
                .reads
                .extend([ReadAction::Bytes(prefix), ReadAction::End]);
            let mut now = || start;
            assert_error_variant(
                read_frame_with_clock(&mut truncated, Duration::from_secs(1), &mut now),
                WebDriverBiDiWebSocketFrameError::FrameEnded { bytes_read: 2 },
            );
        }
    }

    #[test]
    fn close_validation_is_fail_closed_and_wire_compatible() {
        let data = WebDriverBiDiWebSocketFrame {
            fin: true,
            opcode: 1,
            payload: Vec::new(),
        };
        assert!(validate_close_frame(&data).is_ok());
        let empty = WebDriverBiDiWebSocketFrame {
            fin: true,
            opcode: 8,
            payload: Vec::new(),
        };
        assert!(validate_close_frame(&empty).is_ok());
        let one = WebDriverBiDiWebSocketFrame {
            fin: true,
            opcode: 8,
            payload: vec![0],
        };
        assert!(validate_close_frame(&one).is_err());
        let invalid_utf8 = WebDriverBiDiWebSocketFrame {
            fin: true,
            opcode: 8,
            payload: vec![0x03, 0xe8, 0xff],
        };
        assert!(validate_close_frame(&invalid_utf8).is_err());
        for status in [999_u16, 1004, 1005, 1006, 1015, 5000] {
            let payload = status.to_be_bytes().to_vec();
            let frame = WebDriverBiDiWebSocketFrame {
                fin: true,
                opcode: 8,
                payload,
            };
            assert!(validate_close_frame(&frame).is_err());
        }
        for status in [1000_u16, 3000, 4000] {
            let mut payload = status.to_be_bytes().to_vec();
            payload.extend_from_slice(b"ok");
            let frame = WebDriverBiDiWebSocketFrame {
                fin: true,
                opcode: 8,
                payload,
            };
            assert!(validate_close_frame(&frame).is_ok());
        }
    }

    #[test]
    fn frame_errors_have_stable_messages_and_sources() {
        let errors = [
            WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
                frame_timeout: Duration::ZERO,
                maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
            },
            WebDriverBiDiWebSocketFrameError::FrameTooLarge {
                payload_bytes: 2,
                maximum_bytes: 1,
            },
            WebDriverBiDiWebSocketFrameError::FrameReadModeConfigurationFailed {
                source: io::Error::from(io::ErrorKind::InvalidInput),
            },
            WebDriverBiDiWebSocketFrameError::FrameReadTimedOut {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
            WebDriverBiDiWebSocketFrameError::FrameReadFailed {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::BrokenPipe),
            },
            WebDriverBiDiWebSocketFrameError::FrameEnded { bytes_read: 1 },
            WebDriverBiDiWebSocketFrameError::MalformedFrame { reason: "test" },
            WebDriverBiDiWebSocketFrameError::FrameWriteModeConfigurationFailed {
                bytes_written: 1,
                source: io::Error::from(io::ErrorKind::InvalidInput),
            },
            WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                bytes_written: 1,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
            WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
                bytes_written: 1,
                source: io::Error::from(io::ErrorKind::BrokenPipe),
            },
            WebDriverBiDiWebSocketFrameError::FrameWriteZero { bytes_written: 1 },
            WebDriverBiDiWebSocketFrameError::FrameWriteCleanupFailed {
                source: io::Error::from(io::ErrorKind::InvalidInput),
            },
        ];
        for (error, has_source) in errors.iter().zip([
            false, false, true, true, true, false, false, true, true, true, false, true,
        ]) {
            assert!(!error.to_string().is_empty());
            assert_eq!(error.source().is_some(), has_source);
        }
    }
}
