use std::{net::TcpListener, thread, time::Duration};

use originweave_core::WebDriverBiDiWebSocketEndpoint;

use crate::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CLIENT_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";

#[test]
fn handshake_plan_debug_redacts_client_nonce() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("test listener must bind");
    let address = listener
        .local_addr()
        .expect("test listener address must be available");
    let server = thread::spawn(move || {
        listener
            .accept()
            .map(|_| ())
            .expect("test loopback connection must be accepted");
    });

    let endpoint =
        WebDriverBiDiWebSocketEndpoint::new(&format!("ws://{address}/session/{SESSION_ID}"))
            .expect("test endpoint must be valid");
    let correlated = endpoint
        .correlate_session_id(SESSION_ID)
        .expect("test session must correlate");
    let target = correlated
        .into_explicit_connect_target()
        .expect("test target must be explicit");
    let connection = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)
        .expect("test connection plan must be valid")
        .connect()
        .expect("test connection must succeed");
    let client_key =
        WebDriverBiDiWebSocketClientKey::new(CLIENT_KEY).expect("test client key must be valid");
    let handshake = WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key)
        .expect("test handshake plan must be valid");

    let debug = format!("{handshake:?}");
    assert!(debug.contains("WebDriverBiDiWebSocketHandshakePlan"));
    assert!(debug.contains("<redacted WebSocket client nonce>"));
    assert!(!debug.contains(CLIENT_KEY));

    drop(handshake);
    server.join().expect("test server must not panic");
}
