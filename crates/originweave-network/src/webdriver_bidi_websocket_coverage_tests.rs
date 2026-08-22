use std::{
    io::{self, Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;

use crate::webdriver_bidi_websocket_handshake::WebDriverBiDiWebSocketHandshakePlan;
use crate::{
    MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE, MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT,
    MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketHandshakeError,
    WebDriverBiDiWebSocketHandshakeResponseError, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketOpeningRequestSent, WebDriverBiDiWebSocketOpeningWriteError,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";

fn client_key() -> WebDriverBiDiWebSocketClientKey {
    WebDriverBiDiWebSocketClientKey::new("dGhlIHNhbXBsZSBub25jZQ==")
        .expect("test client key must be valid")
}

fn loopback_plan(scheme: &str) -> (WebDriverBiDiTcpConnectionPlan, TcpListener) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("test listener must bind");
    let address = listener
        .local_addr()
        .expect("test listener address must be available");
    let endpoint =
        WebDriverBiDiWebSocketEndpoint::new(&format!("{scheme}://{address}/session/{SESSION_ID}"))
            .expect("test endpoint must be valid");
    let correlated = endpoint
        .correlate_session_id(SESSION_ID)
        .expect("test session must correlate");
    let target = correlated
        .into_explicit_connect_target()
        .expect("test target must be explicit");
    let plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)
        .expect("test connection plan must be valid");
    (plan, listener)
}

fn join_server(server: thread::JoinHandle<io::Result<()>>) {
    server
        .join()
        .expect("test loopback server must not panic")
        .expect("test loopback server must complete");
}

fn opening_sent() -> (
    WebDriverBiDiWebSocketOpeningRequestSent,
    thread::JoinHandle<io::Result<()>>,
) {
    let (plan, listener) = loopback_plan("ws");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept()?;
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request)?;
        Ok(())
    });
    let connection = plan.connect().expect("test connection must succeed");
    let sent = WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key())
        .expect("test handshake plan must be valid")
        .write_opening_request(Duration::from_secs(1))
        .expect("test opening request must be written");
    (sent, server)
}

fn established() -> (
    WebDriverBiDiWebSocketEstablished,
    thread::JoinHandle<io::Result<()>>,
) {
    let (plan, listener) = loopback_plan("ws");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept()?;
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        Ok(())
    });
    let connection = plan.connect().expect("test connection must succeed");
    let established = WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key())
        .expect("test handshake plan must be valid")
        .write_opening_request(Duration::from_secs(1))
        .expect("test opening request must be written")
        .read_opening_response(Duration::from_secs(1))
        .expect("test opening response must be valid");
    (established, server)
}

#[test]
fn public_client_key_guard_rejects_noncanonical_length() {
    assert!(matches!(
        WebDriverBiDiWebSocketClientKey::new("AAAAAAAAAAAAAAAAAAAA=="),
        Err(WebDriverBiDiWebSocketHandshakeError::InvalidClientKey)
    ));
}

#[test]
fn opening_plan_rejects_plain_transport_for_tls_required_target() {
    let (plan, listener) = loopback_plan("wss");
    let server = thread::spawn(move || listener.accept().map(|_| ()));
    let connection = plan.connect().expect("test connection must succeed");

    assert!(matches!(
        WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key()),
        Err(WebDriverBiDiWebSocketHandshakeError::TlsRequired)
    ));
    join_server(server);
}

#[test]
fn public_opening_write_guard_rejects_zero_and_over_ceiling_timeouts() {
    for timeout in [
        Duration::ZERO,
        MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT + Duration::from_nanos(1),
    ] {
        let (plan, listener) = loopback_plan("ws");
        let server = thread::spawn(move || listener.accept().map(|_| ()));
        let connection = plan.connect().expect("test connection must succeed");
        let handshake = WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key())
            .expect("test handshake plan must be valid");

        assert!(matches!(
            handshake.write_opening_request(timeout),
            Err(WebDriverBiDiWebSocketOpeningWriteError::InvalidWriteTimeout {
                write_timeout,
                maximum_timeout,
            }) if write_timeout == timeout && maximum_timeout == MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT
        ));
        join_server(server);
    }
}

#[test]
fn public_opening_response_guard_rejects_zero_and_over_ceiling_timeouts() {
    for timeout in [
        Duration::ZERO,
        MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT + Duration::from_nanos(1),
    ] {
        let (sent, server) = opening_sent();
        assert!(matches!(
            sent.read_opening_response(timeout),
            Err(WebDriverBiDiWebSocketHandshakeResponseError::InvalidResponseTimeout {
                response_timeout,
                maximum_timeout,
            }) if response_timeout == timeout && maximum_timeout == MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT
        ));
        join_server(server);
    }
}

#[test]
fn public_text_frame_guard_rejects_payload_above_reviewed_ceiling() {
    let (established, server) = established();
    let oversized = "x".repeat(MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE + 1);
    let masking_key = WebDriverBiDiWebSocketMaskKey::new([0x37, 0xfa, 0x21, 0x3d]);

    assert!(matches!(
        established.write_text_frame(&oversized, masking_key, Duration::from_secs(1)),
        Err(WebDriverBiDiWebSocketFrameError::FrameTooLarge {
            payload_bytes,
            maximum_bytes,
        }) if payload_bytes == oversized.len() && maximum_bytes == MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE
    ));
    join_server(server);
}
