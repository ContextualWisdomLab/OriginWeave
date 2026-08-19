#![allow(clippy::expect_used)]

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    error::Error,
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use originweave_core::{WebDriverBiDiWebSocketConnectTarget, WebDriverBiDiWebSocketEndpoint};

use super::{
    is_retryable_connect_error, WebDriverBiDiSocketConnector, WebDriverBiDiTcpConnectionError,
    WebDriverBiDiTcpConnectionPlan,
};
use crate::connection::{MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";

fn socket_address() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 9515))
}

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
    let address = listener
        .local_addr()
        .expect("read loopback listener address");
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
    let zero_timeout =
        WebDriverBiDiTcpConnectionPlan::new(connect_target(false), Duration::ZERO, 1);
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
        vec![PeerOutcome::Address(socket_address())],
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
    assert_eq!(connection.verified_peer().socket_addr(), socket_address());
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
            vec![PeerOutcome::Address(socket_address())],
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
            socket_address: socket_address(),
            attempt_count: 2,
            connect_timeout: Duration::from_millis(250),
            source: io::Error::from(io::ErrorKind::TimedOut),
        },
        WebDriverBiDiTcpConnectionError::ConnectionFailed {
            socket_address: socket_address(),
            attempt_count: 3,
            source: io::Error::from(io::ErrorKind::ConnectionRefused),
        },
        WebDriverBiDiTcpConnectionError::PeerInspectionFailed {
            socket_address: socket_address(),
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
