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

        let has_unapproved_ipv6_metadata = match socket_address {
            SocketAddr::V4(_) => false,
            SocketAddr::V6(ipv6_socket) => {
                ipv6_socket.flowinfo() != 0 || ipv6_socket.scope_id() != 0
            }
        };
        let connection_evidence = resolution
            .authorize_connection(socket_address.ip())
            .map_err(|source| NetworkError::DestinationNotApproved {
                socket_address,
                source,
            })?;
        if connection_evidence.canonical_address() != socket_address.ip()
            || has_unapproved_ipv6_metadata
        {
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

    fn connect_with(
        self,
        connector: &dyn SocketConnector,
    ) -> Result<(TcpStream, SocketConnectionEvidence), NetworkError> {
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
                Err(source)
                    if is_retryable_connect_error(source.kind())
                        && attempt_number < self.maximum_attempts =>
                {
                    attempt_number += 1;
                }
                Err(source) => {
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
            }
        }
    }
}

trait SocketConnector {
    fn connect_timeout(
        &self,
        socket_address: &SocketAddr,
        timeout: Duration,
    ) -> io::Result<TcpStream>;

    fn peer_addr(&self, stream: &TcpStream) -> io::Result<SocketAddr>;
}

struct SystemConnector;

impl SocketConnector for SystemConnector {
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
    /// The socket used a non-canonical IP or unapproved IPv6 transport metadata.
    NonCanonicalSocketAddress {
        /// The rejected non-canonical socket address.
        socket_address: SocketAddr,
        /// The canonical IP address required by the snapshot.
        canonical_address: IpAddr,
    },
    /// The final bounded attempt ended with a timeout error.
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
    /// The final attempt ended with a non-timeout connection error.
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
                "socket {socket_address} is not canonical; use IP address {canonical_address} with zero IPv6 flow and scope metadata",
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
    use std::io::{Read, Write};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV6, TcpListener};
    use std::thread;

    use originweave_destination::{DestinationPolicy, ResolutionSnapshot};

    use super::*;

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

    impl SocketConnector for FakeConnector {
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

    fn requested_socket() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
    }

    fn connected_stream() -> TcpStream {
        let listener =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("test listener must bind");
        let socket_address = listener.local_addr().expect("listener address");
        let client = TcpStream::connect_timeout(&socket_address, Duration::from_secs(1))
            .expect("test client must connect");
        let (_server, _peer) = listener.accept().expect("listener must accept");
        client
    }

    fn loopback_snapshot() -> ResolutionSnapshot {
        ResolutionSnapshot::approve(
            Origin::parse("http://localhost").expect("loopback origin"),
            [IpAddr::V4(Ipv4Addr::LOCALHOST)],
            &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        )
        .expect("managed loopback snapshot")
    }

    fn ipv6_loopback_snapshot() -> ResolutionSnapshot {
        ResolutionSnapshot::approve(
            Origin::parse("http://[::1]").expect("IPv6 loopback origin"),
            [IpAddr::V6(Ipv6Addr::LOCALHOST)],
            &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        )
        .expect("managed IPv6 loopback snapshot")
    }

    fn plan(maximum_attempts: u8) -> ConnectionPlan {
        ConnectionPlan::new(
            &loopback_snapshot(),
            requested_socket(),
            Duration::from_secs(2),
            maximum_attempts,
        )
        .expect("valid connection plan")
    }

    #[test]
    fn retryable_connect_error_kinds_are_explicit_and_conservative() {
        for kind in [
            io::ErrorKind::TimedOut,
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::ConnectionAborted,
            io::ErrorKind::Interrupted,
        ] {
            assert!(is_retryable_connect_error(kind), "{kind:?} must retry");
        }
        for kind in [
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::InvalidInput,
            io::ErrorKind::AddrNotAvailable,
        ] {
            assert!(!is_retryable_connect_error(kind), "{kind:?} must fail fast");
        }
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

        assert_eq!(connector.connect_calls.get(), 2);
        assert_eq!(error.attempt_count(), Some(2));
        assert!(error.source().is_some());
        assert!(error.to_string().contains("failed after 2 attempts"));
    }

    #[test]
    fn non_retryable_connection_errors_stop_immediately() {
        for kind in [io::ErrorKind::PermissionDenied, io::ErrorKind::InvalidInput] {
            let connector = FakeConnector::new(
                vec![
                    ConnectOutcome::Error(kind),
                    ConnectOutcome::Error(io::ErrorKind::ConnectionRefused),
                ],
                vec![],
            );
            let error = plan(4)
                .connect_with(&connector)
                .expect_err("deterministic connection failure must stop immediately");

            assert_eq!(connector.connect_calls.get(), 1);
            assert_eq!(connector.peer_calls.get(), 0);
            assert_eq!(error.attempt_count(), Some(1));
            let source = error
                .source()
                .and_then(|source| source.downcast_ref::<io::Error>())
                .expect("connection error must retain its operating-system source");
            assert_eq!(source.kind(), kind);
        }
    }

    #[test]
    fn a_later_success_records_the_successful_attempt() {
        let socket = requested_socket();
        let connector = FakeConnector::new(
            vec![
                ConnectOutcome::Error(io::ErrorKind::ConnectionRefused),
                ConnectOutcome::Success(connected_stream()),
            ],
            vec![PeerOutcome::Address(socket)],
        );
        let (_stream, evidence) = plan(2)
            .connect_with(&connector)
            .expect("second attempt succeeds");

        assert_eq!(connector.connect_calls.get(), 2);
        assert_eq!(connector.peer_calls.get(), 1);
        assert_eq!(evidence.requested_socket(), socket);
        assert_eq!(evidence.observed_peer(), socket);
        assert_eq!(evidence.attempt_number(), 2);
    }

    #[test]
    fn peer_inspection_failure_is_not_retried() {
        let connector = FakeConnector::new(
            vec![ConnectOutcome::Success(connected_stream())],
            vec![PeerOutcome::Error(io::ErrorKind::AddrNotAvailable)],
        );
        let error = plan(4)
            .connect_with(&connector)
            .expect_err("peer inspection fails");

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
            vec![ConnectOutcome::Success(connected_stream())],
            vec![PeerOutcome::Address(observed_peer)],
        );
        let error = plan(4)
            .connect_with(&connector)
            .expect_err("different peer must fail");

        assert!(error.to_string().contains(&observed_peer.to_string()));
        assert_eq!(connector.connect_calls.get(), 1);
        assert_eq!(connector.peer_calls.get(), 1);
        assert_eq!(error.attempt_count(), Some(1));
        assert!(error.source().is_none());
        assert!(error.to_string().contains("peer mismatch"));
    }

    #[test]
    fn validation_errors_cover_every_public_contract() {
        let snapshot = loopback_snapshot();
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80);
        let validation_errors = [
            ConnectionPlan::new(
                &snapshot,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
                Duration::from_secs(1),
                1,
            )
            .expect_err("port zero must fail"),
            ConnectionPlan::new(&snapshot, socket, Duration::ZERO, 1)
                .expect_err("zero timeout must fail"),
            ConnectionPlan::new(
                &snapshot,
                socket,
                MAX_CONNECT_TIMEOUT + Duration::from_nanos(1),
                1,
            )
            .expect_err("excessive timeout must fail"),
            ConnectionPlan::new(&snapshot, socket, Duration::from_secs(1), 0)
                .expect_err("zero attempts must fail"),
            ConnectionPlan::new(
                &snapshot,
                socket,
                Duration::from_secs(1),
                MAX_CONNECTION_ATTEMPTS + 1,
            )
            .expect_err("excessive attempts must fail"),
        ];
        for error in validation_errors {
            assert!(!error.to_string().is_empty());
            assert!(error.source().is_none());
            assert_eq!(error.attempt_count(), None);
        }

        let denied_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 443);
        let denied = ConnectionPlan::new(&snapshot, denied_socket, Duration::from_secs(1), 1)
            .expect_err("address absent from snapshot must fail");
        assert!(denied.to_string().contains("not approved"));
        assert!(denied.source().is_some());
        assert_eq!(denied.attempt_count(), None);

        let mapped = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 1);
        let noncanonical = ConnectionPlan::new(
            &snapshot,
            SocketAddr::new(IpAddr::V6(mapped), 443),
            Duration::from_secs(1),
            1,
        )
        .expect_err("mapped address must fail canonical authority");
        assert!(noncanonical.to_string().contains("canonical"));
        assert!(noncanonical.source().is_none());
        assert_eq!(noncanonical.attempt_count(), None);

        let ipv6_snapshot = ipv6_loopback_snapshot();
        let canonical_ipv6_socket =
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 443, 0, 0));
        assert!(
            ConnectionPlan::new(
                &ipv6_snapshot,
                canonical_ipv6_socket,
                Duration::from_secs(1),
                1,
            )
            .is_ok()
        );
        for (flowinfo, scope_id) in [(1, 0), (0, 1)] {
            let socket_with_metadata = SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::LOCALHOST,
                443,
                flowinfo,
                scope_id,
            ));
            let metadata_error = ConnectionPlan::new(
                &ipv6_snapshot,
                socket_with_metadata,
                Duration::from_secs(1),
                1,
            )
            .expect_err("IPv6 transport metadata requires separate authority");
            assert!(
                metadata_error
                    .to_string()
                    .contains("zero IPv6 flow and scope")
            );
            assert!(metadata_error.source().is_none());
            assert_eq!(metadata_error.attempt_count(), None);
        }
    }

    #[test]
    fn public_destination_policy_denies_loopback_before_network_authority() {
        let error = ResolutionSnapshot::approve(
            Origin::parse("https://example.com").expect("public origin"),
            [IpAddr::V4(Ipv4Addr::LOCALHOST)],
            &DestinationPolicy::public_web(),
        )
        .expect_err("public policy must deny loopback");
        assert!(error.to_string().contains("denied as Loopback"));
    }

    #[test]
    fn approved_loopback_socket_becomes_the_exact_operating_system_peer() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("listener must bind");
        let socket = listener.local_addr().expect("listener address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("listener must accept");
            stream.write_all(b"ok").expect("server response must write");
        });

        let origin = Origin::parse("http://localhost").expect("loopback origin");
        let snapshot = loopback_snapshot();
        let connection = ConnectionPlan::new(&snapshot, socket, Duration::from_secs(1), 1)
            .expect("plan must validate")
            .connect()
            .expect("exact loopback peer must connect");

        assert_eq!(
            connection.stream().peer_addr().expect("peer address"),
            socket
        );
        assert_eq!(connection.evidence().origin(), &origin);
        assert_eq!(connection.evidence().requested_socket(), socket);
        assert_eq!(connection.evidence().observed_peer(), socket);
        assert_eq!(
            connection.evidence().address_class(),
            AddressClass::Loopback
        );
        assert_eq!(connection.evidence().attempt_number(), 1);
        assert_eq!(
            connection.evidence().connect_timeout(),
            Duration::from_secs(1)
        );

        let (mut stream, evidence) = connection.into_parts();
        let mut body = [0_u8; 2];
        stream.read_exact(&mut body).expect("response must read");
        assert_eq!(&body, b"ok");
        assert_eq!(evidence.observed_peer(), socket);
        server.join().expect("server thread must finish");
    }

    #[test]
    fn refused_loopback_connection_stops_at_the_exact_attempt_bound() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("port must reserve");
        let socket = listener.local_addr().expect("reserved address");
        drop(listener);

        let error = ConnectionPlan::new(&loopback_snapshot(), socket, Duration::from_secs(1), 3)
            .expect("plan must validate")
            .connect()
            .expect_err("closed loopback port must fail");

        assert_eq!(error.attempt_count(), Some(3));
        assert!(error.source().is_some());
        assert!(error.to_string().contains("failed after 3 attempts"));
    }
}
