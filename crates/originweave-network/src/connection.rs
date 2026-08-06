use std::fmt;
use std::io;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationError, ResolutionSnapshot};

/// The largest timeout accepted for one direct TCP connection attempt.
pub const MAX_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// The largest number of direct TCP connection attempts in one plan.
pub const MAX_CONNECTION_ATTEMPTS: u8 = 4;

/// A validated, single-use authority to connect to one exact socket address.
///
/// The plan deliberately does not implement `Clone` or `Copy`. Calling
/// [`ConnectionPlan::connect`] consumes the value, so the same authority cannot
/// be replayed accidentally.
#[derive(Debug)]
pub struct ConnectionPlan {
    origin: Origin,
    requested_socket: SocketAddr,
    address_class: AddressClass,
    connect_timeout: Duration,
    maximum_attempts: u8,
}

impl ConnectionPlan {
    /// Validate one exact direct-connection request without performing I/O.
    ///
    /// The supplied IP address must already use the canonical form authorized
    /// by `resolution`. Hostnames, resolver callbacks, ambient proxy settings,
    /// TLS, and HTTP are outside this crate.
    pub fn new(
        resolution: &ResolutionSnapshot,
        socket_address: SocketAddr,
        connect_timeout: Duration,
        maximum_attempts: u8,
    ) -> Result<Self, NetworkError> {
        if socket_address.port() == 0 {
            return Err(NetworkError::InvalidPort);
        }
        if connect_timeout.is_zero() || connect_timeout > MAX_CONNECT_TIMEOUT {
            return Err(NetworkError::InvalidConnectTimeout {
                connect_timeout,
                maximum_timeout: MAX_CONNECT_TIMEOUT,
            });
        }
        if maximum_attempts == 0 || maximum_attempts > MAX_CONNECTION_ATTEMPTS {
            return Err(NetworkError::InvalidAttemptCount {
                attempt_count: maximum_attempts,
                maximum_attempts: MAX_CONNECTION_ATTEMPTS,
            });
        }

        let connection_evidence = resolution
            .authorize_connection(socket_address.ip())
            .map_err(|source| NetworkError::DestinationNotApproved {
                socket_address,
                source,
            })?;
        if connection_evidence.canonical_address() != socket_address.ip() {
            return Err(NetworkError::NonCanonicalSocketAddress {
                socket_address,
                canonical_address: connection_evidence.canonical_address(),
            });
        }

        Ok(Self {
            origin: connection_evidence.origin().clone(),
            requested_socket: socket_address,
            address_class: connection_evidence.address_class(),
            connect_timeout,
            maximum_attempts,
        })
    }

    /// Open the exact approved socket and expose it only after peer verification.
    ///
    /// ```compile_fail
    /// # fn replay(plan: originweave_network::ConnectionPlan) {
    /// let _first_attempt = plan.connect();
    /// let _replayed_attempt = plan.connect();
    /// # }
    /// ```
    pub fn connect(self) -> Result<DirectTcpConnection, NetworkError> {
        let (stream, evidence) = self.connect_with(&SystemConnector)?;
        Ok(DirectTcpConnection { stream, evidence })
    }

    fn connect_with<C: SocketConnector>(
        self,
        connector: &C,
    ) -> Result<(C::Stream, SocketConnectionEvidence), NetworkError> {
        let mut attempt_number = 1;
        loop {
            match connector.connect_timeout(&self.requested_socket, self.connect_timeout) {
                Ok(stream) => {
                    let observed_peer = connector.peer_addr(&stream).map_err(|source| {
                        NetworkError::PeerInspectionFailed {
                            socket_address: self.requested_socket,
                            attempt_number,
                            source,
                        }
                    })?;
                    if observed_peer != self.requested_socket {
                        return Err(NetworkError::PeerMismatch {
                            socket_address: self.requested_socket,
                            observed_peer,
                            attempt_number,
                        });
                    }
                    let evidence = SocketConnectionEvidence {
                        origin: self.origin,
                        requested_socket: self.requested_socket,
                        observed_peer,
                        address_class: self.address_class,
                        attempt_number,
                        connect_timeout: self.connect_timeout,
                    };
                    return Ok((stream, evidence));
                }
                Err(source) if attempt_number >= self.maximum_attempts => {
                    if source.kind() == io::ErrorKind::TimedOut {
                        return Err(NetworkError::ConnectionTimedOut {
                            socket_address: self.requested_socket,
                            attempt_count: attempt_number,
                            connect_timeout: self.connect_timeout,
                            source,
                        });
                    }
                    return Err(NetworkError::ConnectionFailed {
                        socket_address: self.requested_socket,
                        attempt_count: attempt_number,
                        source,
                    });
                }
                Err(_source) => {
                    attempt_number += 1;
                }
            }
        }
    }
}

trait SocketConnector {
    type Stream;

    fn connect_timeout(
        &self,
        socket_address: &SocketAddr,
        timeout: Duration,
    ) -> io::Result<Self::Stream>;

    fn peer_addr(&self, stream: &Self::Stream) -> io::Result<SocketAddr>;
}

struct SystemConnector;

impl SocketConnector for SystemConnector {
    type Stream = TcpStream;

    fn connect_timeout(
        &self,
        socket_address: &SocketAddr,
        timeout: Duration,
    ) -> io::Result<Self::Stream> {
        TcpStream::connect_timeout(socket_address, timeout)
    }

    fn peer_addr(&self, stream: &Self::Stream) -> io::Result<SocketAddr> {
        stream.peer_addr()
    }
}

/// An established direct TCP stream and its verified peer evidence.
#[derive(Debug)]
pub struct DirectTcpConnection {
    stream: TcpStream,
    evidence: SocketConnectionEvidence,
}

impl DirectTcpConnection {
    /// Borrow the verified TCP stream.
    #[must_use]
    pub const fn stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Borrow the credential-free connection evidence.
    #[must_use]
    pub const fn evidence(&self) -> &SocketConnectionEvidence {
        &self.evidence
    }

    /// Consume the wrapper and return the verified stream and evidence.
    #[must_use]
    pub fn into_parts(self) -> (TcpStream, SocketConnectionEvidence) {
        (self.stream, self.evidence)
    }
}

/// Credential-free evidence for one verified direct TCP connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketConnectionEvidence {
    origin: Origin,
    requested_socket: SocketAddr,
    observed_peer: SocketAddr,
    address_class: AddressClass,
    attempt_number: u8,
    connect_timeout: Duration,
}

impl SocketConnectionEvidence {
    /// Return the logical origin whose resolution authorized the connection.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the exact socket address submitted to the operating system.
    #[must_use]
    pub const fn requested_socket(&self) -> SocketAddr {
        self.requested_socket
    }

    /// Return the remote peer address reported by the established stream.
    #[must_use]
    pub const fn observed_peer(&self) -> SocketAddr {
        self.observed_peer
    }

    /// Return the security class recorded for the approved address.
    #[must_use]
    pub const fn address_class(&self) -> AddressClass {
        self.address_class
    }

    /// Return the one-based attempt on which the connection succeeded.
    #[must_use]
    pub const fn attempt_number(&self) -> u8 {
        self.attempt_number
    }

    /// Return the timeout applied to each direct connection attempt.
    #[must_use]
    pub const fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }
}

/// A deterministic reason that direct TCP connection authority failed.
#[derive(Debug)]
pub enum NetworkError {
    /// The requested destination port was zero.
    InvalidPort,
    /// The timeout was zero or exceeded [`MAX_CONNECT_TIMEOUT`].
    InvalidConnectTimeout {
        /// The rejected timeout.
        connect_timeout: Duration,
        /// The largest accepted timeout.
        maximum_timeout: Duration,
    },
    /// The requested attempt count was outside the accepted range.
    InvalidAttemptCount {
        /// The rejected attempt count.
        attempt_count: u8,
        /// The largest accepted attempt count.
        maximum_attempts: u8,
    },
    /// The requested IP address was not authorized by the resolution snapshot.
    DestinationNotApproved {
        /// The rejected socket address.
        socket_address: SocketAddr,
        /// The underlying destination-policy decision.
        source: DestinationError,
    },
    /// The requested IP was authorized only after canonicalization.
    NonCanonicalSocketAddress {
        /// The rejected non-canonical socket address.
        socket_address: SocketAddr,
        /// The canonical IP address required by the snapshot.
        canonical_address: IpAddr,
    },
    /// Every bounded attempt ended with a timeout error.
    ConnectionTimedOut {
        /// The exact socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// The number of attempts that completed before failure.
        attempt_count: u8,
        /// The timeout applied to each attempt.
        connect_timeout: Duration,
        /// The final operating-system error.
        source: io::Error,
    },
    /// Every bounded attempt ended with a non-timeout connection error.
    ConnectionFailed {
        /// The exact socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// The number of attempts that completed before failure.
        attempt_count: u8,
        /// The final operating-system error.
        source: io::Error,
    },
    /// The established stream did not reveal its remote peer address.
    PeerInspectionFailed {
        /// The exact socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// The one-based attempt that established the stream.
        attempt_number: u8,
        /// The operating-system peer-inspection error.
        source: io::Error,
    },
    /// The established stream reported a peer other than the approved socket.
    PeerMismatch {
        /// The exact approved socket address.
        socket_address: SocketAddr,
        /// The different peer reported by the stream.
        observed_peer: SocketAddr,
        /// The one-based attempt that established the stream.
        attempt_number: u8,
    },
}

impl NetworkError {
    /// Return the number of transport attempts associated with this failure.
    #[must_use]
    pub const fn attempt_count(&self) -> Option<u8> {
        match self {
            Self::ConnectionTimedOut { attempt_count, .. }
            | Self::ConnectionFailed { attempt_count, .. } => Some(*attempt_count),
            Self::PeerInspectionFailed { attempt_number, .. }
            | Self::PeerMismatch { attempt_number, .. } => Some(*attempt_number),
            Self::InvalidPort
            | Self::InvalidConnectTimeout { .. }
            | Self::InvalidAttemptCount { .. }
            | Self::DestinationNotApproved { .. }
            | Self::NonCanonicalSocketAddress { .. } => None,
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPort => formatter.write_str("connection port must be within 1..=65535"),
            Self::InvalidConnectTimeout {
                connect_timeout,
                maximum_timeout,
            } => write!(
                formatter,
                "connect timeout {connect_timeout:?} is outside 1ns..={maximum_timeout:?}",
            ),
            Self::InvalidAttemptCount {
                attempt_count,
                maximum_attempts,
            } => write!(
                formatter,
                "connection attempt count {attempt_count} is outside 1..={maximum_attempts}",
            ),
            Self::DestinationNotApproved {
                socket_address,
                source,
            } => write!(
                formatter,
                "socket {socket_address} is not approved by the resolution snapshot: {source}",
            ),
            Self::NonCanonicalSocketAddress {
                socket_address,
                canonical_address,
            } => write!(
                formatter,
                "socket {socket_address} is not canonical; use IP address {canonical_address}",
            ),
            Self::ConnectionTimedOut {
                socket_address,
                attempt_count,
                connect_timeout,
                ..
            } => write!(
                formatter,
                "TCP connection to {socket_address} timed out after {attempt_count} attempts with per-attempt timeout {connect_timeout:?}",
            ),
            Self::ConnectionFailed {
                socket_address,
                attempt_count,
                ..
            } => write!(
                formatter,
                "TCP connection to {socket_address} failed after {attempt_count} attempts",
            ),
            Self::PeerInspectionFailed {
                socket_address,
                attempt_number,
                ..
            } => write!(
                formatter,
                "TCP peer inspection failed for {socket_address} on attempt {attempt_number}",
            ),
            Self::PeerMismatch {
                socket_address,
                observed_peer,
                attempt_number,
            } => write!(
                formatter,
                "TCP peer mismatch on attempt {attempt_number}: requested {socket_address}, observed {observed_peer}",
            ),
        }
    }
}

impl std::error::Error for NetworkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DestinationNotApproved { source, .. } => Some(source),
            Self::ConnectionTimedOut { source, .. }
            | Self::ConnectionFailed { source, .. }
            | Self::PeerInspectionFailed { source, .. } => Some(source),
            Self::InvalidPort
            | Self::InvalidConnectTimeout { .. }
            | Self::InvalidAttemptCount { .. }
            | Self::NonCanonicalSocketAddress { .. }
            | Self::PeerMismatch { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::error::Error;
    use std::net::{IpAddr, Ipv4Addr};

    use originweave_destination::{DestinationPolicy, ResolutionSnapshot};

    use super::*;

    enum ConnectOutcome {
        Success(u8),
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

    impl SocketConnector for FakeConnector {
        type Stream = u8;

        fn connect_timeout(
            &self,
            _socket_address: &SocketAddr,
            _timeout: Duration,
        ) -> io::Result<Self::Stream> {
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

        fn peer_addr(&self, _stream: &Self::Stream) -> io::Result<SocketAddr> {
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

    fn requested_socket() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
    }

    fn plan(maximum_attempts: u8) -> ConnectionPlan {
        let snapshot = ResolutionSnapshot::approve(
            Origin::parse("http://localhost").expect("loopback origin"),
            [IpAddr::V4(Ipv4Addr::LOCALHOST)],
            &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        )
        .expect("managed loopback snapshot");
        ConnectionPlan::new(
            &snapshot,
            requested_socket(),
            Duration::from_secs(2),
            maximum_attempts,
        )
        .expect("valid connection plan")
    }

    #[test]
    fn timeout_uses_every_bounded_attempt_and_retains_source() {
        let connector = FakeConnector::new(
            vec![
                ConnectOutcome::Error(io::ErrorKind::TimedOut),
                ConnectOutcome::Error(io::ErrorKind::TimedOut),
                ConnectOutcome::Error(io::ErrorKind::TimedOut),
            ],
            vec![],
        );
        let error = plan(3)
            .connect_with(&connector)
            .expect_err("all attempts time out");

        assert!(matches!(
            error,
            NetworkError::ConnectionTimedOut {
                attempt_count: 3,
                ..
            }
        ));
        assert_eq!(connector.connect_calls.get(), 3);
        assert_eq!(connector.peer_calls.get(), 0);
        assert_eq!(error.attempt_count(), Some(3));
        assert!(error.source().is_some());
        assert!(error.to_string().contains("timed out"));
    }

    #[test]
    fn ordinary_failure_uses_every_bounded_attempt_and_retains_source() {
        let connector = FakeConnector::new(
            vec![
                ConnectOutcome::Error(io::ErrorKind::ConnectionRefused),
                ConnectOutcome::Error(io::ErrorKind::ConnectionRefused),
            ],
            vec![],
        );
        let error = plan(2)
            .connect_with(&connector)
            .expect_err("all attempts fail");

        assert!(matches!(
            error,
            NetworkError::ConnectionFailed {
                attempt_count: 2,
                ..
            }
        ));
        assert_eq!(connector.connect_calls.get(), 2);
        assert_eq!(error.attempt_count(), Some(2));
        assert!(error.source().is_some());
        assert!(error.to_string().contains("failed after 2 attempts"));
    }

    #[test]
    fn a_later_success_records_the_successful_attempt() {
        let socket = requested_socket();
        let connector = FakeConnector::new(
            vec![
                ConnectOutcome::Error(io::ErrorKind::ConnectionRefused),
                ConnectOutcome::Success(7),
            ],
            vec![PeerOutcome::Address(socket)],
        );
        let (stream, evidence) = plan(2)
            .connect_with(&connector)
            .expect("second attempt succeeds");

        assert_eq!(stream, 7);
        assert_eq!(connector.connect_calls.get(), 2);
        assert_eq!(connector.peer_calls.get(), 1);
        assert_eq!(evidence.requested_socket(), socket);
        assert_eq!(evidence.observed_peer(), socket);
        assert_eq!(evidence.attempt_number(), 2);
    }

    #[test]
    fn peer_inspection_failure_is_not_retried() {
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Success(1)],
            vec![PeerOutcome::Error(io::ErrorKind::AddrNotAvailable)],
        );
        let error = plan(4)
            .connect_with(&connector)
            .expect_err("peer inspection fails");

        assert!(matches!(
            error,
            NetworkError::PeerInspectionFailed {
                attempt_number: 1,
                ..
            }
        ));
        assert_eq!(connector.connect_calls.get(), 1);
        assert_eq!(connector.peer_calls.get(), 1);
        assert_eq!(error.attempt_count(), Some(1));
        assert!(error.source().is_some());
        assert!(error.to_string().contains("peer inspection failed"));
    }

    #[test]
    fn mismatched_peer_is_not_exposed_or_retried() {
        let observed_peer = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081);
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Success(1)],
            vec![PeerOutcome::Address(observed_peer)],
        );
        let error = plan(4)
            .connect_with(&connector)
            .expect_err("different peer must fail");

        assert!(matches!(
            error,
            NetworkError::PeerMismatch {
                attempt_number: 1,
                observed_peer: peer,
                ..
            } if peer == observed_peer
        ));
        assert_eq!(connector.connect_calls.get(), 1);
        assert_eq!(connector.peer_calls.get(), 1);
        assert_eq!(error.attempt_count(), Some(1));
        assert!(error.source().is_none());
        assert!(error.to_string().contains("peer mismatch"));
    }
}
