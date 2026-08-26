use std::{net::TcpListener, thread, time::Duration};

use originweave_core::WebDriverBiDiWebSocketEndpoint;

use crate::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    webdriver_bidi_websocket_handshake_raw::WebDriverBiDiWebSocketHandshakePlan as RawWebDriverBiDiWebSocketHandshakePlan,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CLIENT_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";

#[test]
fn client_key_debug_redacts_client_nonce() {
    let client_key =
        WebDriverBiDiWebSocketClientKey::new(CLIENT_KEY).expect("test client key must be valid");

    let debug = format!("{client_key:?}");
    assert!(debug.contains("<redacted WebSocket client nonce>"));
    assert!(!debug.contains(CLIENT_KEY));
}

#[test]
fn masking_key_debug_redacts_frame_entropy() {
    let masking_key = WebDriverBiDiWebSocketMaskKey::new([17, 34, 51, 68]);

    let debug = format!("{masking_key:?}");
    assert!(debug.contains("<redacted WebSocket masking key>"));
    assert!(!debug.contains("17"));
    assert!(!debug.contains("34"));
    assert!(!debug.contains("51"));
    assert!(!debug.contains("68"));
}

#[test]
fn raw_handshake_plan_debug_omits_serialized_request() {
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
    let handshake = RawWebDriverBiDiWebSocketHandshakePlan::new(connection, client_key)
        .expect("test raw handshake plan must be valid");

    let debug = format!("{handshake:?}");
    assert!(debug.contains("<redacted WebSocket client nonce>"));
    assert!(!debug.contains("request: ["));
    assert!(!debug.contains(CLIENT_KEY));

    drop(handshake);
    server.join().expect("test server must not panic");
}

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
