use std::{fmt, io, net::SocketAddr, net::TcpStream, time::Duration};

use originweave_core::{
    VerifiedWebDriverBiDiSocketPeer, WebDriverBiDiSocketPeerVerificationError,
    WebDriverBiDiWebSocketConnectTarget,
};

use crate::connection::{MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS};

fn is_retryable_connect_error(kind: io::ErrorKind) -> bool {
    matches!(
        kind,
        io::ErrorKind::TimedOut
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::Interrupted
    )
}

/// Single-use authority to open one exact WebDriver BiDi loopback TCP destination.
///
/// The plan consumes a session-correlated, no-DNS [`WebDriverBiDiWebSocketConnectTarget`]
/// produced by `originweave-core`. It applies the same bounded per-attempt timeout and retry
/// ceilings as the general direct-network connector, opens only the exact [`SocketAddr`] carried by
/// that target, and does not expose the stream until the operating system's observed peer has been
/// verified by the consumed target.
///
/// This boundary performs no DNS lookup, proxy or PAC routing, Chromium/ChromeDriver process
/// authentication, TLS negotiation, WebSocket upgrade, BiDi framing, browser policy decision, or
/// Agent-authority grant.
#[derive(Debug)]
pub struct WebDriverBiDiTcpConnectionPlan {
    target: WebDriverBiDiWebSocketConnectTarget,
    connect_timeout: Duration,
    maximum_attempts: u8,
}

impl WebDriverBiDiTcpConnectionPlan {
    /// Validate one bounded exact-loopback connection plan without performing network I/O.
    pub fn new(
        target: WebDriverBiDiWebSocketConnectTarget,
        connect_timeout: Duration,
        maximum_attempts: u8,
    ) -> Result<Self, WebDriverBiDiTcpConnectionError> {
        if connect_timeout.is_zero() || connect_timeout > MAX_CONNECT_TIMEOUT {
            return Err(WebDriverBiDiTcpConnectionError::InvalidConnectTimeout {
                connect_timeout,
                maximum_timeout: MAX_CONNECT_TIMEOUT,
            });
        }
        if maximum_attempts == 0 || maximum_attempts > MAX_CONNECTION_ATTEMPTS {
            return Err(WebDriverBiDiTcpConnectionError::InvalidAttemptCount {
                attempt_count: maximum_attempts,
                maximum_attempts: MAX_CONNECTION_ATTEMPTS,
            });
        }

        Ok(Self {
            target,
            connect_timeout,
            maximum_attempts,
        })
    }

    /// Open the exact approved loopback socket and expose it only after peer verification.
    ///
    /// Retry is limited to transport errors that can occur transiently while a local browser driver
    /// listener is becoming ready. Peer-inspection and peer-mismatch failures are integrity failures
    /// and therefore fail closed without retry or fallback.
    pub fn connect(self) -> Result<WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError> {
        self.connect_with(&SystemWebDriverBiDiConnector)
    }

    fn connect_with(
        self,
        connector: &dyn WebDriverBiDiSocketConnector,
    ) -> Result<WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError> {
        let socket_address = self.target.socket_addr();
        let connect_timeout = self.connect_timeout;
        let maximum_attempts = self.maximum_attempts;
        let target = self.target;
        let mut attempt_number = 1;

        loop {
            match connector.connect_timeout(&socket_address, connect_timeout) {
                Ok(stream) => {
                    let observed_peer = connector.peer_addr(&stream).map_err(|source| {
                        WebDriverBiDiTcpConnectionError::PeerInspectionFailed {
                            socket_address,
                            attempt_number,
                            source,
                        }
                    })?;
                    let verified_peer = target.verify_connected_peer(observed_peer).map_err(
                        |source| WebDriverBiDiTcpConnectionError::PeerMismatch {
                            attempt_number,
                            source,
                        },
                    )?;
                    return Ok(WebDriverBiDiTcpConnection {
                        stream,
                        verified_peer,
                        attempt_number,
                        connect_timeout,
                    });
                }
                Err(source)
                    if is_retryable_connect_error(source.kind())
                        && attempt_number < maximum_attempts =>
                {
                    attempt_number += 1;
                }
                Err(source) => {
                    if source.kind() == io::ErrorKind::TimedOut {
                        return Err(WebDriverBiDiTcpConnectionError::ConnectionTimedOut {
                            socket_address,
                            attempt_count: attempt_number,
                            connect_timeout,
                            source,
                        });
                    }
                    return Err(WebDriverBiDiTcpConnectionError::ConnectionFailed {
                        socket_address,
                        attempt_count: attempt_number,
                        source,
                    });
                }
            }
        }
    }
}

trait WebDriverBiDiSocketConnector {
    fn connect_timeout(
        &self,
        socket_address: &SocketAddr,
        timeout: Duration,
    ) -> io::Result<TcpStream>;

    fn peer_addr(&self, stream: &TcpStream) -> io::Result<SocketAddr>;
}

struct SystemWebDriverBiDiConnector;

impl WebDriverBiDiSocketConnector for SystemWebDriverBiDiConnector {
    fn connect_timeout(
        &self,
        socket_address: &SocketAddr,
        timeout: Duration,
    ) -> io::Result<TcpStream> {
        TcpStream::connect_timeout(socket_address, timeout)
    }

    fn peer_addr(&self, stream: &TcpStream) -> io::Result<SocketAddr> {
        stream.peer_addr()
    }
}

/// Established WebDriver BiDi TCP stream whose observed peer matched the approved target exactly.
///
/// This wrapper proves only exact transport-destination equality for one bounded connection. The
/// caller must still establish any required TLS channel, complete a WebSocket handshake, bind the
/// transport to the expected browser process/session, and pass separate action-policy checks.
#[derive(Debug)]
pub struct WebDriverBiDiTcpConnection {
    stream: TcpStream,
    verified_peer: VerifiedWebDriverBiDiSocketPeer,
    attempt_number: u8,
    connect_timeout: Duration,
}

impl WebDriverBiDiTcpConnection {
    /// Borrow the verified TCP stream.
    #[must_use]
    pub const fn stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Borrow the session-correlated exact peer evidence consumed by this connection.
    #[must_use]
    pub const fn verified_peer(&self) -> &VerifiedWebDriverBiDiSocketPeer {
        &self.verified_peer
    }

    /// Return the one-based bounded attempt on which the connection succeeded.
    #[must_use]
    pub const fn attempt_number(&self) -> u8 {
        self.attempt_number
    }

    /// Return the per-attempt timeout applied while establishing this connection.
    #[must_use]
    pub const fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }
}

/// Deterministic failures while establishing one bounded WebDriver BiDi TCP transport.
#[derive(Debug)]
pub enum WebDriverBiDiTcpConnectionError {
    /// The requested timeout was zero or exceeded [`MAX_CONNECT_TIMEOUT`].
    InvalidConnectTimeout {
        /// The rejected timeout.
        connect_timeout: Duration,
        /// The largest accepted per-attempt timeout.
        maximum_timeout: Duration,
    },
    /// The requested attempt count was outside `1..=MAX_CONNECTION_ATTEMPTS`.
    InvalidAttemptCount {
        /// The rejected attempt count.
        attempt_count: u8,
        /// The largest accepted attempt count.
        maximum_attempts: u8,
    },
    /// The final bounded connection attempt timed out.
    ConnectionTimedOut {
        /// Exact approved socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// Number of attempts completed before failure.
        attempt_count: u8,
        /// Per-attempt timeout used by the plan.
        connect_timeout: Duration,
        /// Final operating-system timeout error.
        source: io::Error,
    },
    /// The final bounded connection attempt failed without a timeout.
    ConnectionFailed {
        /// Exact approved socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// Number of attempts completed before failure.
        attempt_count: u8,
        /// Final operating-system connection error.
        source: io::Error,
    },
    /// The established stream did not reveal an operating-system peer address.
    PeerInspectionFailed {
        /// Exact approved socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// One-based attempt that established the stream.
        attempt_number: u8,
        /// Operating-system peer-inspection error.
        source: io::Error,
    },
    /// The established stream reported a peer other than the exact approved BiDi target.
    PeerMismatch {
        /// One-based attempt that established the stream.
        attempt_number: u8,
        /// Typed core peer-verification failure preserving expected and actual socket addresses.
        source: WebDriverBiDiSocketPeerVerificationError,
    },
}

impl WebDriverBiDiTcpConnectionError {
    /// Return the number of transport attempts associated with this failure, when applicable.
    #[must_use]
    pub const fn attempt_count(&self) -> Option<u8> {
        match self {
            Self::ConnectionTimedOut { attempt_count, .. }
            | Self::ConnectionFailed { attempt_count, .. } => Some(*attempt_count),
            Self::PeerInspectionFailed { attempt_number, .. }
            | Self::PeerMismatch { attempt_number, .. } => Some(*attempt_number),
            Self::InvalidConnectTimeout { .. } | Self::InvalidAttemptCount { .. } => None,
        }
    }
}

impl fmt::Display for WebDriverBiDiTcpConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConnectTimeout {
                connect_timeout,
                maximum_timeout,
            } => write!(
                formatter,
                "WebDriver BiDi connect timeout {connect_timeout:?} is outside 1ns..={maximum_timeout:?}",
            ),
            Self::InvalidAttemptCount {
                attempt_count,
                maximum_attempts,
            } => write!(
                formatter,
                "WebDriver BiDi connection attempt count {attempt_count} is outside 1..={maximum_attempts}",
            ),
            Self::ConnectionTimedOut {
                socket_address,
                attempt_count,
                connect_timeout,
                ..
            } => write!(
                formatter,
                "WebDriver BiDi TCP connection to {socket_address} timed out after {attempt_count} attempts with per-attempt timeout {connect_timeout:?}",
            ),
            Self::ConnectionFailed {
                socket_address,
                attempt_count,
                ..
            } => write!(
                formatter,
                "WebDriver BiDi TCP connection to {socket_address} failed after {attempt_count} attempts",
            ),
            Self::PeerInspectionFailed {
                socket_address,
                attempt_number,
                ..
            } => write!(
                formatter,
                "WebDriver BiDi TCP peer inspection failed for {socket_address} on attempt {attempt_number}",
            ),
            Self::PeerMismatch { attempt_number, .. } => write!(
                formatter,
                "WebDriver BiDi TCP peer did not match the approved target on attempt {attempt_number}",
            ),
        }
    }
}

impl std::error::Error for WebDriverBiDiTcpConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ConnectionTimedOut { source, .. }
            | Self::ConnectionFailed { source, .. }
            | Self::PeerInspectionFailed { source, .. } => Some(source),
            Self::PeerMismatch { source, .. } => Some(source),
            Self::InvalidConnectTimeout { .. } | Self::InvalidAttemptCount { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::{
        cell::{Cell, RefCell},
        collections::VecDeque,
        error::Error,
        net::{TcpListener, TcpStream},
    };

    use originweave_core::WebDriverBiDiWebSocketEndpoint;

    use super::*;

    const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
    const SOCKET_ADDRESS: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 9515));

    enum ConnectOutcome {
        Success(TcpStream),
        Error(io::ErrorKind),
    }

    enum PeerOutcome {
        Address(SocketAddr),
        Error(io::ErrorKind),
    }

    struct FakeConnector {
        connect_outcomes: RefCell<VecDeque<ConnectOutcome>>,
        peer_outcomes: RefCell<VecDeque<PeerOutcome>>,
        connect_calls: Cell<u8>,
        peer_calls: Cell<u8>,
    }

    impl FakeConnector {
        fn new(connect_outcomes: Vec<ConnectOutcome>, peer_outcomes: Vec<PeerOutcome>) -> Self {
            Self {
                connect_outcomes: RefCell::new(connect_outcomes.into()),
                peer_outcomes: RefCell::new(peer_outcomes.into()),
                connect_calls: Cell::new(0),
                peer_calls: Cell::new(0),
            }
        }
    }

    impl WebDriverBiDiSocketConnector for FakeConnector {
        fn connect_timeout(
            &self,
            _socket_address: &SocketAddr,
            _timeout: Duration,
        ) -> io::Result<TcpStream> {
            self.connect_calls.set(self.connect_calls.get() + 1);
            match self
                .connect_outcomes
                .borrow_mut()
                .pop_front()
                .expect("test must provide a connection outcome")
            {
                ConnectOutcome::Success(stream) => Ok(stream),
                ConnectOutcome::Error(kind) => Err(io::Error::from(kind)),
            }
        }

        fn peer_addr(&self, _stream: &TcpStream) -> io::Result<SocketAddr> {
            self.peer_calls.set(self.peer_calls.get() + 1);
            match self
                .peer_outcomes
                .borrow_mut()
                .pop_front()
                .expect("test must provide a peer outcome")
            {
                PeerOutcome::Address(address) => Ok(address),
                PeerOutcome::Error(kind) => Err(io::Error::from(kind)),
            }
        }
    }

    fn loopback_stream() -> TcpStream {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
        let address = listener.local_addr().expect("read loopback listener address");
        let client = TcpStream::connect(address).expect("connect loopback client");
        let (server, _) = listener.accept().expect("accept loopback client");
        drop(server);
        client
    }

    fn connect_target(secure: bool) -> WebDriverBiDiWebSocketConnectTarget {
        let scheme = if secure { "wss" } else { "ws" };
        let endpoint = format!("{scheme}://127.0.0.1:9515/session/{SESSION_ID}");
        let admitted = WebDriverBiDiWebSocketEndpoint::new(&endpoint).expect("admit endpoint");
        let correlated = admitted
            .correlate_session_id(SESSION_ID)
            .expect("correlate endpoint");
        correlated
            .into_explicit_connect_target()
            .expect("derive explicit connect target")
    }

    fn plan(maximum_attempts: u8) -> WebDriverBiDiTcpConnectionPlan {
        WebDriverBiDiTcpConnectionPlan::new(
            connect_target(false),
            Duration::from_millis(250),
            maximum_attempts,
        )
        .expect("valid test plan")
    }

    #[test]
    fn validates_timeout_and_attempt_bounds_before_io() {
        let zero_timeout = WebDriverBiDiTcpConnectionPlan::new(
            connect_target(false),
            Duration::ZERO,
            1,
        );
        assert!(matches!(
            zero_timeout,
            Err(WebDriverBiDiTcpConnectionError::InvalidConnectTimeout { .. })
        ));

        let excessive_timeout = WebDriverBiDiTcpConnectionPlan::new(
            connect_target(false),
            MAX_CONNECT_TIMEOUT + Duration::from_nanos(1),
            1,
        );
        assert!(matches!(
            excessive_timeout,
            Err(WebDriverBiDiTcpConnectionError::InvalidConnectTimeout { .. })
        ));

        let zero_attempts = WebDriverBiDiTcpConnectionPlan::new(
            connect_target(false),
            Duration::from_millis(250),
            0,
        );
        assert!(matches!(
            zero_attempts,
            Err(WebDriverBiDiTcpConnectionError::InvalidAttemptCount { .. })
        ));

        let excessive_attempts = WebDriverBiDiTcpConnectionPlan::new(
            connect_target(false),
            Duration::from_millis(250),
            MAX_CONNECTION_ATTEMPTS + 1,
        );
        assert!(matches!(
            excessive_attempts,
            Err(WebDriverBiDiTcpConnectionError::InvalidAttemptCount { .. })
        ));
    }

    #[test]
    fn verified_peer_is_required_before_stream_exposure() {
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Success(loopback_stream())],
            vec![PeerOutcome::Address(SOCKET_ADDRESS)],
        );
        let connection = WebDriverBiDiTcpConnectionPlan::new(
            connect_target(true),
            Duration::from_millis(250),
            1,
        )
        .expect("valid plan")
        .connect_with(&connector)
        .expect("verified connection");

        assert!(connection.stream().peer_addr().is_ok());
        assert_eq!(connection.verified_peer().socket_addr(), SOCKET_ADDRESS);
        assert!(connection.verified_peer().requires_tls());
        assert_eq!(connection.verified_peer().session_id(), SESSION_ID);
        assert_eq!(connection.attempt_number(), 1);
        assert_eq!(connection.connect_timeout(), Duration::from_millis(250));
        assert_eq!(connector.connect_calls.get(), 1);
        assert_eq!(connector.peer_calls.get(), 1);
    }

    #[test]
    fn all_recoverable_connect_kinds_can_retry_once() {
        for kind in [
            io::ErrorKind::TimedOut,
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::ConnectionAborted,
            io::ErrorKind::Interrupted,
        ] {
            assert!(is_retryable_connect_error(kind));
            let connector = FakeConnector::new(
                vec![
                    ConnectOutcome::Error(kind),
                    ConnectOutcome::Success(loopback_stream()),
                ],
                vec![PeerOutcome::Address(SOCKET_ADDRESS)],
            );
            let connection = plan(2)
                .connect_with(&connector)
                .expect("second bounded attempt succeeds");
            assert_eq!(connection.attempt_number(), 2);
            assert_eq!(connector.connect_calls.get(), 2);
            assert_eq!(connector.peer_calls.get(), 1);
        }
        assert!(!is_retryable_connect_error(io::ErrorKind::PermissionDenied));
    }

    #[test]
    fn final_timeout_preserves_source_and_attempt_count() {
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Error(io::ErrorKind::TimedOut)],
            Vec::new(),
        );
        let error = plan(1)
            .connect_with(&connector)
            .expect_err("timeout must fail closed");
        assert!(matches!(
            error,
            WebDriverBiDiTcpConnectionError::ConnectionTimedOut {
                attempt_count: 1,
                ..
            }
        ));
        assert!(error.source().is_some());
        assert_eq!(error.attempt_count(), Some(1));
    }

    #[test]
    fn exhausted_retryable_non_timeout_error_is_connection_failure() {
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Error(io::ErrorKind::ConnectionRefused)],
            Vec::new(),
        );
        let error = plan(1)
            .connect_with(&connector)
            .expect_err("refusal must fail after the bounded final attempt");
        assert!(matches!(
            error,
            WebDriverBiDiTcpConnectionError::ConnectionFailed {
                attempt_count: 1,
                ..
            }
        ));
        assert!(error.source().is_some());
    }

    #[test]
    fn non_retryable_connection_error_fails_without_retry() {
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Error(io::ErrorKind::PermissionDenied)],
            Vec::new(),
        );
        let error = plan(MAX_CONNECTION_ATTEMPTS)
            .connect_with(&connector)
            .expect_err("permission failure must not retry");
        assert!(matches!(
            error,
            WebDriverBiDiTcpConnectionError::ConnectionFailed {
                attempt_count: 1,
                ..
            }
        ));
        assert_eq!(connector.connect_calls.get(), 1);
    }

    #[test]
    fn peer_inspection_failure_is_not_retried() {
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Success(loopback_stream())],
            vec![PeerOutcome::Error(io::ErrorKind::NotConnected)],
        );
        let error = plan(MAX_CONNECTION_ATTEMPTS)
            .connect_with(&connector)
            .expect_err("peer inspection failure must fail closed");
        assert!(matches!(
            error,
            WebDriverBiDiTcpConnectionError::PeerInspectionFailed {
                attempt_number: 1,
                ..
            }
        ));
        assert_eq!(connector.connect_calls.get(), 1);
        assert_eq!(connector.peer_calls.get(), 1);
        assert!(error.source().is_some());
    }

    #[test]
    fn peer_mismatch_is_not_retried_or_converted_to_success() {
        let wrong_peer = SocketAddr::from(([127, 0, 0, 1], 9516));
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Success(loopback_stream())],
            vec![PeerOutcome::Address(wrong_peer)],
        );
        let error = plan(MAX_CONNECTION_ATTEMPTS)
            .connect_with(&connector)
            .expect_err("peer mismatch must fail closed");
        assert!(matches!(
            error,
            WebDriverBiDiTcpConnectionError::PeerMismatch {
                attempt_number: 1,
                ..
            }
        ));
        assert_eq!(connector.connect_calls.get(), 1);
        assert_eq!(connector.peer_calls.get(), 1);
        assert!(error.source().is_some());
    }

    #[test]
    fn error_display_source_and_attempt_contracts_cover_every_variant() {
        let mismatch = connect_target(false)
            .verify_connected_peer(SocketAddr::from(([127, 0, 0, 1], 9516)))
            .expect_err("wrong peer must fail");
        let errors = [
            WebDriverBiDiTcpConnectionError::InvalidConnectTimeout {
                connect_timeout: Duration::ZERO,
                maximum_timeout: MAX_CONNECT_TIMEOUT,
            },
            WebDriverBiDiTcpConnectionError::InvalidAttemptCount {
                attempt_count: 0,
                maximum_attempts: MAX_CONNECTION_ATTEMPTS,
            },
            WebDriverBiDiTcpConnectionError::ConnectionTimedOut {
                socket_address: SOCKET_ADDRESS,
                attempt_count: 2,
                connect_timeout: Duration::from_millis(250),
                source: io::Error::from(io::ErrorKind::TimedOut),
            },
            WebDriverBiDiTcpConnectionError::ConnectionFailed {
                socket_address: SOCKET_ADDRESS,
                attempt_count: 3,
                source: io::Error::from(io::ErrorKind::ConnectionRefused),
            },
            WebDriverBiDiTcpConnectionError::PeerInspectionFailed {
                socket_address: SOCKET_ADDRESS,
                attempt_number: 1,
                source: io::Error::from(io::ErrorKind::NotConnected),
            },
            WebDriverBiDiTcpConnectionError::PeerMismatch {
                attempt_number: 1,
                source: mismatch,
            },
        ];

        let messages: Vec<String> = errors.iter().map(ToString::to_string).collect();
        assert!(messages[0].contains("outside 1ns"));
        assert!(messages[1].contains("attempt count 0"));
        assert!(messages[2].contains("timed out after 2 attempts"));
        assert!(messages[3].contains("failed after 3 attempts"));
        assert!(messages[4].contains("peer inspection failed"));
        assert!(messages[5].contains("did not match the approved target"));

        assert_eq!(errors[0].attempt_count(), None);
        assert_eq!(errors[1].attempt_count(), None);
        assert_eq!(errors[2].attempt_count(), Some(2));
        assert_eq!(errors[3].attempt_count(), Some(3));
        assert_eq!(errors[4].attempt_count(), Some(1));
        assert_eq!(errors[5].attempt_count(), Some(1));
        assert!(errors[0].source().is_none());
        assert!(errors[1].source().is_none());
        assert!(errors[2].source().is_some());
        assert!(errors[3].source().is_some());
        assert!(errors[4].source().is_some());
        assert!(errors[5].source().is_some());
    }
}
