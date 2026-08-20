use std::{
    error::Error,
    fmt,
    io::{self, Read},
    net::TcpStream,
    time::{Duration, Instant},
};

use crate::{
    WebDriverBiDiTcpConnectionEvidence, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketOpeningRequestSent,
};

const OPENING_RESPONSE_TERMINATOR: &[u8; 4] = b"\r\n\r\n";

/// Maximum wall-clock budget accepted for reading one WebSocket opening-response header section.
///
/// This is an OriginWeave resource-safety ceiling, not an RFC 6455 protocol limit. Callers may
/// choose any smaller nonzero deadline.
pub const MAX_WEBSOCKET_OPENING_READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum number of bytes accepted before the opening-response header terminator is observed.
///
/// This is an OriginWeave resource-safety budget, not an HTTP or RFC 6455 maximum. The terminating
/// `CRLF CRLF` bytes are included in the budget.
pub const MAX_WEBSOCKET_OPENING_RESPONSE_HEADER_BYTES: usize = 16 * 1024;

/// Fail-closed errors while reading one bounded WebDriver BiDi WebSocket opening response.
#[derive(Debug)]
pub enum WebDriverBiDiWebSocketOpeningReadError {
    /// The requested total read deadline was zero or above the reviewed resource ceiling.
    InvalidReadTimeout {
        /// Rejected caller-supplied deadline.
        read_timeout: Duration,
        /// Maximum reviewed deadline accepted by this boundary.
        maximum_timeout: Duration,
    },
    /// The monotonic total read deadline elapsed before the complete header section was received.
    ReadDeadlineExceeded {
        /// Number of response bytes received before the deadline elapsed.
        bytes_read: usize,
    },
    /// Applying the remaining operating-system read timeout failed.
    ReadTimeoutConfigurationFailed {
        /// Number of response bytes already received before configuration failed.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// A bounded socket read reported timeout or would-block before the header section completed.
    ReadTimedOut {
        /// Number of response bytes received before the timed-out operation.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// The peer closed the stream before the opening-response header section completed.
    UnexpectedEof {
        /// Number of response bytes received before EOF.
        bytes_read: usize,
    },
    /// The response exceeded the reviewed header-byte budget before its terminator was observed.
    ResponseHeaderTooLarge {
        /// Maximum reviewed header bytes, including the terminator.
        maximum_bytes: usize,
    },
    /// A non-recoverable socket read failed before the complete header section was received.
    ReadFailed {
        /// Number of response bytes received before the failure.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// Clearing the operation-local socket read timeout failed after the header was received.
    ReadTimeoutCleanupFailed {
        /// Number of response bytes already received before cleanup failed.
        bytes_read: usize,
        /// Underlying operating-system error.
        source: io::Error,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketOpeningReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidReadTimeout { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening read timeout is outside the reviewed bound",
            ),
            Self::ReadDeadlineExceeded { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening read exceeded its monotonic deadline",
            ),
            Self::ReadTimeoutConfigurationFailed { .. } => formatter.write_str(
                "failed to configure the bounded WebDriver BiDi WebSocket opening read timeout",
            ),
            Self::ReadTimedOut { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening read timed out before the response header completed",
            ),
            Self::UnexpectedEof { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket peer closed before the opening response header completed",
            ),
            Self::ResponseHeaderTooLarge { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening response header exceeded the reviewed byte bound",
            ),
            Self::ReadFailed { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket opening response read failed before the header completed",
            ),
            Self::ReadTimeoutCleanupFailed { .. } => formatter.write_str(
                "failed to clear the WebDriver BiDi WebSocket opening read timeout before handoff",
            ),
        }
    }
}

impl Error for WebDriverBiDiWebSocketOpeningReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadTimeoutConfigurationFailed { source, .. }
            | Self::ReadTimedOut { source, .. }
            | Self::ReadFailed { source, .. }
            | Self::ReadTimeoutCleanupFailed { source, .. } => Some(source),
            Self::InvalidReadTimeout { .. }
            | Self::ReadDeadlineExceeded { .. }
            | Self::UnexpectedEof { .. }
            | Self::ResponseHeaderTooLarge { .. } => None,
        }
    }
}

/// A live verified stream after one bounded opening-response header section has been received.
///
/// This state preserves the exact client key and transport/request evidence from the preceding
/// opening write together with the exact `CRLF CRLF`-terminated response header bytes. It does not
/// parse or accept the server handshake: `101 Switching Protocols`, `Upgrade`, `Connection`, and
/// `Sec-WebSocket-Accept` remain untrusted bytes until a later fail-closed validator consumes this
/// value. No WebSocket OPEN state, browser identity, policy decision, browser action, or Agent
/// authority is established here.
pub struct WebDriverBiDiWebSocketOpeningResponseRead {
    sent: WebDriverBiDiWebSocketOpeningRequestSent,
    header: Vec<u8>,
    read_timeout: Duration,
}

impl fmt::Debug for WebDriverBiDiWebSocketOpeningResponseRead {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiWebSocketOpeningResponseRead")
            .field("transport_evidence", self.sent.transport_evidence())
            .field(
                "client_key",
                &"<retained for Sec-WebSocket-Accept validation>",
            )
            .field("request_byte_count", &self.sent.request_byte_count())
            .field("write_timeout", &self.sent.write_timeout())
            .field("header_byte_count", &self.header.len())
            .field("read_timeout", &self.read_timeout)
            .finish()
    }
}

impl WebDriverBiDiWebSocketOpeningResponseRead {
    /// Borrow the exact unparsed opening-response header bytes, including the final `CRLF CRLF`.
    #[must_use]
    pub fn header_bytes(&self) -> &[u8] {
        &self.header
    }

    /// Return the total read deadline configured for this opening response.
    #[must_use]
    pub const fn read_timeout(&self) -> Duration {
        self.read_timeout
    }

    /// Borrow the exact verified transport evidence retained from the opening request.
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        self.sent.transport_evidence()
    }

    /// Borrow the exact client key required by a later `Sec-WebSocket-Accept` validator.
    #[must_use]
    pub const fn client_key(&self) -> &WebDriverBiDiWebSocketClientKey {
        self.sent.client_key()
    }

    /// Return the exact number of opening-request bytes written before this read boundary.
    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.sent.request_byte_count()
    }

    /// Return the total write deadline used by the preceding opening request.
    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.sent.write_timeout()
    }
}

impl WebDriverBiDiWebSocketOpeningRequestSent {
    /// Read one complete bounded RFC 6455 server opening-response header section.
    ///
    /// The already-verified stream is consumed into the result on success. Zero and over-ceiling
    /// deadlines fail closed before reading. Reads use one total monotonic deadline and retry only an
    /// interrupted system call. The reader consumes exactly one byte at a time until `CRLF CRLF`, so
    /// it never discards bytes belonging to the first WebSocket frame. A response that closes early,
    /// times out, exceeds the product header budget, or raises any other I/O failure yields no success
    /// evidence. Before success, the operation-local socket read timeout is cleared so the next stage
    /// cannot inherit stale timeout authority.
    pub fn read_opening_response(
        mut self,
        read_timeout: Duration,
    ) -> Result<WebDriverBiDiWebSocketOpeningResponseRead, WebDriverBiDiWebSocketOpeningReadError>
    {
        if read_timeout.is_zero() || read_timeout > MAX_WEBSOCKET_OPENING_READ_TIMEOUT {
            return Err(WebDriverBiDiWebSocketOpeningReadError::InvalidReadTimeout {
                read_timeout,
                maximum_timeout: MAX_WEBSOCKET_OPENING_READ_TIMEOUT,
            });
        }

        let mut now = Instant::now;
        let header = read_response_header_with_clock(&mut self.stream, read_timeout, &mut now)?;
        Ok(WebDriverBiDiWebSocketOpeningResponseRead {
            sent: self,
            header,
            read_timeout,
        })
    }
}

trait OpeningResponseReader {
    fn set_read_timeout(&self, timeout: Duration) -> io::Result<()>;
    fn clear_read_timeout(&self) -> io::Result<()>;
    fn read_response_byte(&mut self, byte: &mut [u8; 1]) -> io::Result<usize>;
}

impl OpeningResponseReader for TcpStream {
    fn set_read_timeout(&self, timeout: Duration) -> io::Result<()> {
        TcpStream::set_read_timeout(self, Some(timeout))
    }

    fn clear_read_timeout(&self) -> io::Result<()> {
        TcpStream::set_read_timeout(self, None)
    }

    fn read_response_byte(&mut self, byte: &mut [u8; 1]) -> io::Result<usize> {
        self.read(byte)
    }
}

fn read_response_header_with_clock(
    reader: &mut dyn OpeningResponseReader,
    read_timeout: Duration,
    now: &mut dyn FnMut() -> Instant,
) -> Result<Vec<u8>, WebDriverBiDiWebSocketOpeningReadError> {
    let deadline = now() + read_timeout;
    let mut response = Vec::with_capacity(512);
    let mut byte = [0_u8; 1];

    loop {
        let remaining = deadline.saturating_duration_since(now());
        if remaining.is_zero() {
            return Err(
                WebDriverBiDiWebSocketOpeningReadError::ReadDeadlineExceeded {
                    bytes_read: response.len(),
                },
            );
        }
        reader.set_read_timeout(remaining).map_err(|source| {
            WebDriverBiDiWebSocketOpeningReadError::ReadTimeoutConfigurationFailed {
                bytes_read: response.len(),
                source,
            }
        })?;

        match reader.read_response_byte(&mut byte) {
            Ok(0) => {
                return Err(WebDriverBiDiWebSocketOpeningReadError::UnexpectedEof {
                    bytes_read: response.len(),
                });
            }
            Ok(_) => {
                response.push(byte[0]);
                if deadline.saturating_duration_since(now()).is_zero() {
                    return Err(
                        WebDriverBiDiWebSocketOpeningReadError::ReadDeadlineExceeded {
                            bytes_read: response.len(),
                        },
                    );
                }
                if response.ends_with(OPENING_RESPONSE_TERMINATOR) {
                    reader.clear_read_timeout().map_err(|source| {
                        WebDriverBiDiWebSocketOpeningReadError::ReadTimeoutCleanupFailed {
                            bytes_read: response.len(),
                            source,
                        }
                    })?;
                    return Ok(response);
                }
                if response.len() >= MAX_WEBSOCKET_OPENING_RESPONSE_HEADER_BYTES {
                    return Err(
                        WebDriverBiDiWebSocketOpeningReadError::ResponseHeaderTooLarge {
                            maximum_bytes: MAX_WEBSOCKET_OPENING_RESPONSE_HEADER_BYTES,
                        },
                    );
                }
            }
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source)
                if matches!(
                    source.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                return Err(WebDriverBiDiWebSocketOpeningReadError::ReadTimedOut {
                    bytes_read: response.len(),
                    source,
                });
            }
            Err(source) => {
                return Err(WebDriverBiDiWebSocketOpeningReadError::ReadFailed {
                    bytes_read: response.len(),
                    source,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug)]
    enum ReadStep {
        Byte(u8),
        Eof,
        Error(io::ErrorKind),
    }

    struct FakeReader {
        steps: VecDeque<ReadStep>,
        set_timeout_error: Option<io::ErrorKind>,
        clear_timeout_error: Option<io::ErrorKind>,
    }

    impl FakeReader {
        fn from_steps(steps: impl IntoIterator<Item = ReadStep>) -> Self {
            Self {
                steps: steps.into_iter().collect(),
                set_timeout_error: None,
                clear_timeout_error: None,
            }
        }
    }

    impl OpeningResponseReader for FakeReader {
        fn set_read_timeout(&self, _timeout: Duration) -> io::Result<()> {
            match self.set_timeout_error {
                Some(kind) => Err(io::Error::from(kind)),
                None => Ok(()),
            }
        }

        fn clear_read_timeout(&self) -> io::Result<()> {
            match self.clear_timeout_error {
                Some(kind) => Err(io::Error::from(kind)),
                None => Ok(()),
            }
        }

        fn read_response_byte(&mut self, byte: &mut [u8; 1]) -> io::Result<usize> {
            match self.steps.pop_front() {
                Some(ReadStep::Byte(value)) => {
                    byte[0] = value;
                    Ok(1)
                }
                Some(ReadStep::Eof) | None => Ok(0),
                Some(ReadStep::Error(kind)) => Err(io::Error::from(kind)),
            }
        }
    }

    fn fixed_clock(instants: impl IntoIterator<Item = Instant>) -> impl FnMut() -> Instant {
        let mut instants: VecDeque<_> = instants.into_iter().collect();
        let fallback = Instant::now();
        move || instants.pop_front().unwrap_or(fallback)
    }

    fn assert_source_kind(error: &WebDriverBiDiWebSocketOpeningReadError, expected: io::ErrorKind) {
        let source = error.source();
        assert!(source.is_some());
        if let Some(source) = source {
            assert_eq!(
                source.downcast_ref::<io::Error>().map(io::Error::kind),
                Some(expected)
            );
        }
    }

    #[test]
    fn bounded_reader_retries_interrupted_and_returns_exact_header() {
        let mut reader = FakeReader::from_steps([
            ReadStep::Error(io::ErrorKind::Interrupted),
            ReadStep::Byte(b'X'),
            ReadStep::Byte(b'\r'),
            ReadStep::Byte(b'\n'),
            ReadStep::Byte(b'\r'),
            ReadStep::Byte(b'\n'),
        ]);
        let start = Instant::now();
        let mut now = move || start;
        let result = read_response_header_with_clock(&mut reader, Duration::from_secs(1), &mut now);
        assert!(result.is_ok(), "{result:?}");
        if let Ok(header) = result {
            assert_eq!(header, b"X\r\n\r\n");
        }
    }

    #[test]
    fn bounded_reader_rejects_deadline_before_and_after_read() {
        let start = Instant::now();
        let mut before_reader = FakeReader::from_steps([]);
        let mut before_clock = fixed_clock([start, start + Duration::from_secs(1)]);
        let before = read_response_header_with_clock(
            &mut before_reader,
            Duration::from_secs(1),
            &mut before_clock,
        );
        assert!(matches!(
            before,
            Err(WebDriverBiDiWebSocketOpeningReadError::ReadDeadlineExceeded { bytes_read: 0 })
        ));

        let mut after_reader = FakeReader::from_steps([ReadStep::Byte(b'X')]);
        let mut after_clock = fixed_clock([start, start, start + Duration::from_secs(1)]);
        let after = read_response_header_with_clock(
            &mut after_reader,
            Duration::from_secs(1),
            &mut after_clock,
        );
        assert!(matches!(
            after,
            Err(WebDriverBiDiWebSocketOpeningReadError::ReadDeadlineExceeded { bytes_read: 1 })
        ));
    }

    #[test]
    fn bounded_reader_classifies_configuration_timeout_eof_and_io_failures() {
        let start = Instant::now();

        let mut configuration_reader = FakeReader::from_steps([]);
        configuration_reader.set_timeout_error = Some(io::ErrorKind::InvalidInput);
        let mut configuration_clock = move || start;
        let configuration = read_response_header_with_clock(
            &mut configuration_reader,
            Duration::from_secs(1),
            &mut configuration_clock,
        );
        assert!(matches!(
            &configuration,
            Err(
                WebDriverBiDiWebSocketOpeningReadError::ReadTimeoutConfigurationFailed {
                    bytes_read: 0,
                    ..
                }
            )
        ));
        if let Err(error) = configuration {
            assert_source_kind(&error, io::ErrorKind::InvalidInput);
        }

        for kind in [io::ErrorKind::TimedOut, io::ErrorKind::WouldBlock] {
            let mut timeout_reader = FakeReader::from_steps([ReadStep::Error(kind)]);
            let mut timeout_clock = move || start;
            let timeout = read_response_header_with_clock(
                &mut timeout_reader,
                Duration::from_secs(1),
                &mut timeout_clock,
            );
            assert!(matches!(
                &timeout,
                Err(WebDriverBiDiWebSocketOpeningReadError::ReadTimedOut { bytes_read: 0, .. })
            ));
            if let Err(error) = timeout {
                assert_source_kind(&error, kind);
            }
        }

        let mut eof_reader = FakeReader::from_steps([ReadStep::Eof]);
        let mut eof_clock = move || start;
        let eof = read_response_header_with_clock(
            &mut eof_reader,
            Duration::from_secs(1),
            &mut eof_clock,
        );
        assert!(matches!(
            eof,
            Err(WebDriverBiDiWebSocketOpeningReadError::UnexpectedEof { bytes_read: 0 })
        ));

        let mut failure_reader =
            FakeReader::from_steps([ReadStep::Error(io::ErrorKind::ConnectionReset)]);
        let mut failure_clock = move || start;
        let failure = read_response_header_with_clock(
            &mut failure_reader,
            Duration::from_secs(1),
            &mut failure_clock,
        );
        assert!(matches!(
            &failure,
            Err(WebDriverBiDiWebSocketOpeningReadError::ReadFailed { bytes_read: 0, .. })
        ));
        if let Err(error) = failure {
            assert_source_kind(&error, io::ErrorKind::ConnectionReset);
        }
    }

    #[test]
    fn bounded_reader_rejects_oversize_and_cleanup_failure() {
        let start = Instant::now();
        let mut oversize_reader = FakeReader::from_steps(
            (0..MAX_WEBSOCKET_OPENING_RESPONSE_HEADER_BYTES).map(|_| ReadStep::Byte(b'X')),
        );
        let mut oversize_clock = move || start;
        let oversize = read_response_header_with_clock(
            &mut oversize_reader,
            Duration::from_secs(1),
            &mut oversize_clock,
        );
        assert!(matches!(
            oversize,
            Err(
                WebDriverBiDiWebSocketOpeningReadError::ResponseHeaderTooLarge {
                    maximum_bytes: MAX_WEBSOCKET_OPENING_RESPONSE_HEADER_BYTES,
                }
            )
        ));

        let mut cleanup_reader = FakeReader::from_steps([
            ReadStep::Byte(b'\r'),
            ReadStep::Byte(b'\n'),
            ReadStep::Byte(b'\r'),
            ReadStep::Byte(b'\n'),
        ]);
        cleanup_reader.clear_timeout_error = Some(io::ErrorKind::InvalidInput);
        let mut cleanup_clock = move || start;
        let cleanup = read_response_header_with_clock(
            &mut cleanup_reader,
            Duration::from_secs(1),
            &mut cleanup_clock,
        );
        assert!(matches!(
            &cleanup,
            Err(
                WebDriverBiDiWebSocketOpeningReadError::ReadTimeoutCleanupFailed {
                    bytes_read: 4,
                    ..
                }
            )
        ));
        if let Err(error) = cleanup {
            assert_source_kind(&error, io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn read_error_display_and_source_contracts_are_complete() {
        let errors = [
            WebDriverBiDiWebSocketOpeningReadError::InvalidReadTimeout {
                read_timeout: Duration::ZERO,
                maximum_timeout: MAX_WEBSOCKET_OPENING_READ_TIMEOUT,
            },
            WebDriverBiDiWebSocketOpeningReadError::ReadDeadlineExceeded { bytes_read: 1 },
            WebDriverBiDiWebSocketOpeningReadError::ReadTimeoutConfigurationFailed {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::InvalidInput),
            },
            WebDriverBiDiWebSocketOpeningReadError::ReadTimedOut {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
            WebDriverBiDiWebSocketOpeningReadError::UnexpectedEof { bytes_read: 1 },
            WebDriverBiDiWebSocketOpeningReadError::ResponseHeaderTooLarge {
                maximum_bytes: MAX_WEBSOCKET_OPENING_RESPONSE_HEADER_BYTES,
            },
            WebDriverBiDiWebSocketOpeningReadError::ReadFailed {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::ConnectionReset),
            },
            WebDriverBiDiWebSocketOpeningReadError::ReadTimeoutCleanupFailed {
                bytes_read: 1,
                source: io::Error::from(io::ErrorKind::InvalidInput),
            },
        ];

        for error in errors {
            assert!(!error.to_string().is_empty());
            match &error {
                WebDriverBiDiWebSocketOpeningReadError::ReadTimeoutConfigurationFailed { .. }
                | WebDriverBiDiWebSocketOpeningReadError::ReadTimedOut { .. }
                | WebDriverBiDiWebSocketOpeningReadError::ReadFailed { .. }
                | WebDriverBiDiWebSocketOpeningReadError::ReadTimeoutCleanupFailed { .. } => {
                    assert!(error.source().is_some());
                }
                WebDriverBiDiWebSocketOpeningReadError::InvalidReadTimeout { .. }
                | WebDriverBiDiWebSocketOpeningReadError::ReadDeadlineExceeded { .. }
                | WebDriverBiDiWebSocketOpeningReadError::UnexpectedEof { .. }
                | WebDriverBiDiWebSocketOpeningReadError::ResponseHeaderTooLarge { .. } => {
                    assert!(error.source().is_none());
                }
            }
        }
    }
}
