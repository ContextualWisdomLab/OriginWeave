use std::{
    error::Error,
    fmt,
    io::{self, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

use originweave_core::VerifiedWebDriverBiDiSocketPeer;

use crate::{WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionEvidence};

const WEBSOCKET_CLIENT_KEY_LENGTH: usize = 24;

/// Maximum wall-clock budget accepted for writing one bounded WebSocket opening request.
///
/// This is an OriginWeave resource-safety ceiling, not an RFC 6455 protocol limit. The request is
/// already bounded before this budget is applied. Callers may choose any smaller nonzero deadline.
pub const MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT: Duration = Duration::from_secs(5);

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
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source)
                if matches!(
                    source.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                return Err(WebDriverBiDiWebSocketOpeningWriteError::WriteTimedOut {
                    bytes_written,
                    source,
                });
            }
            Err(source) => {
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

    #[test]
    fn bounded_writer_clears_real_socket_timeout_before_success() {
        let listener = match TcpListener::bind(("127.0.0.1", 0)) {
            Ok(listener) => listener,
            Err(error) => panic!("failed to bind loopback listener: {error}"),
        };
        let address = match listener.local_addr() {
            Ok(address) => address,
            Err(error) => panic!("failed to read loopback listener address: {error}"),
        };
        let server = thread::spawn(move || listener.accept().map(|_| ()));
        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(error) => panic!("failed to connect loopback stream: {error}"),
        };
        let start = Instant::now();
        let mut now = || start;

        let result =
            write_request_with_clock(&mut stream, b"opening", Duration::from_secs(1), &mut now);

        assert!(matches!(result, Ok(7)));
        let write_timeout = match stream.write_timeout() {
            Ok(write_timeout) => write_timeout,
            Err(error) => panic!("failed to inspect write timeout: {error}"),
        };
        assert_eq!(write_timeout, None);
        let server_result = match server.join() {
            Ok(server_result) => server_result,
            Err(_) => panic!("loopback server thread panicked"),
        };
        assert!(server_result.is_ok());
    }

    #[test]
    fn bounded_writer_rejects_cleanup_failure_without_success_handoff() {
        let mut writer = FakeWriter::new([WriteAction::Count(1)]);
        writer.clear_timeout_error = Some(io::ErrorKind::InvalidInput);
        let start = Instant::now();
        let mut now = || start;

        let result = write_request_with_clock(&mut writer, b"x", Duration::from_secs(1), &mut now);
        assert!(matches!(
            result,
            Err(
                WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutCleanupFailed {
                    bytes_written: 1,
                    ..
                }
            )
        ));
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
