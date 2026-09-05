#![allow(clippy::expect_used)]

use std::{
    cell::{Cell, RefCell},
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::atomic::AtomicU64,
    time::Duration,
};

use originweave_core::{WebDriverBiDiWebSocketConnectTarget, WebDriverBiDiWebSocketEndpoint};

use super::{
    WebDriverBiDiSocketConnector, WebDriverBiDiTcpConnectionError, WebDriverBiDiTcpConnectionPlan,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";

struct VerifiedConnector {
    stream: RefCell<Option<TcpStream>>,
    connect_calls: Cell<u8>,
    peer_calls: Cell<u8>,
}

impl VerifiedConnector {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: RefCell::new(Some(stream)),
            connect_calls: Cell::new(0),
            peer_calls: Cell::new(0),
        }
    }
}

impl WebDriverBiDiSocketConnector for VerifiedConnector {
    fn connect_timeout(
        &self,
        _socket_address: &SocketAddr,
        _timeout: Duration,
    ) -> io::Result<TcpStream> {
        self.connect_calls.set(self.connect_calls.get() + 1);
        self.stream
            .borrow_mut()
            .take()
            .ok_or_else(|| io::Error::other("test stream already consumed"))
    }

    fn peer_addr(&self, _stream: &TcpStream) -> io::Result<SocketAddr> {
        self.peer_calls.set(self.peer_calls.get() + 1);
        Ok(SocketAddr::from(([127, 0, 0, 1], 9515)))
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

fn connect_target() -> WebDriverBiDiWebSocketConnectTarget {
    let endpoint = format!("ws://127.0.0.1:9515/session/{SESSION_ID}");
    WebDriverBiDiWebSocketEndpoint::new(&endpoint)
        .expect("admit endpoint")
        .correlate_session_id(SESSION_ID)
        .expect("correlate endpoint")
        .into_explicit_connect_target()
        .expect("derive explicit connect target")
}

#[test]
fn verified_connection_fails_closed_when_generation_space_is_exhausted() {
    let connector = VerifiedConnector::new(loopback_stream());
    let exhausted_counter = AtomicU64::new(u64::MAX);
    let error =
        WebDriverBiDiTcpConnectionPlan::new(connect_target(), Duration::from_millis(250), 1)
            .expect("valid plan")
            .connect_with_generation_counter(&connector, &exhausted_counter)
            .expect_err("generation exhaustion must fail after exact peer verification");

    assert!(matches!(
        error,
        WebDriverBiDiTcpConnectionError::ConnectionGenerationExhausted
    ));
    assert_eq!(connector.connect_calls.get(), 1);
    assert_eq!(connector.peer_calls.get(), 1);
}
