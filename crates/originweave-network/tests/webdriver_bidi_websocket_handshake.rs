use std::{net::TcpListener, thread, time::Duration};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakeError, WebDriverBiDiWebSocketHandshakePlan,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";

fn connect(endpoint: &str) -> originweave_network::WebDriverBiDiTcpConnection {
    let admitted = WebDriverBiDiWebSocketEndpoint::new(endpoint);
    assert!(admitted.is_ok(), "{admitted:?}");
    let Ok(admitted) = admitted else {
        unreachable!("asserted valid endpoint")
    };
    let correlated = admitted.correlate_session_id(SESSION_ID);
    assert!(correlated.is_ok(), "{correlated:?}");
    let Ok(correlated) = correlated else {
        unreachable!("asserted correlated endpoint")
    };
    let target = correlated.into_explicit_connect_target();
    assert!(target.is_ok(), "{target:?}");
    let Ok(target) = target else {
        unreachable!("asserted explicit target")
    };
    let plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1);
    assert!(plan.is_ok(), "{plan:?}");
    let Ok(plan) = plan else {
        unreachable!("asserted connection plan")
    };
    let connection = plan.connect();
    assert!(connection.is_ok(), "{connection:?}");
    let Ok(connection) = connection else {
        unreachable!("asserted loopback connection")
    };
    connection
}

#[test]
fn plain_bidi_connection_serializes_exact_rfc6455_opening_request() {
    let listener = TcpListener::bind(("127.0.0.1", 0));
    assert!(listener.is_ok(), "{listener:?}");
    let Ok(listener) = listener else {
        return;
    };
    let local_addr = listener.local_addr();
    assert!(local_addr.is_ok(), "{local_addr:?}");
    let Ok(local_addr) = local_addr else {
        return;
    };
    let server = thread::spawn(move || listener.accept().map(|_| ()));

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let connection = connect(&endpoint);
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY);
    assert!(key.is_ok(), "{key:?}");
    let Ok(key) = key else {
        return;
    };
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connection, key);
    assert!(plan.is_ok(), "{plan:?}");
    let Ok(plan) = plan else {
        return;
    };

    let expected = format!(
        "GET /session/{SESSION_ID} HTTP/1.1\r\nHost: {local_addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {RFC6455_SAMPLE_KEY}\r\nSec-WebSocket-Version: 13\r\n\r\n"
    );
    assert_eq!(plan.request_bytes(), expected.as_bytes());
    assert_eq!(plan.verified_peer().socket_addr(), local_addr);
    assert_eq!(plan.verified_peer().session_id(), SESSION_ID);
    assert!(!plan.verified_peer().requires_tls());

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    if let Ok(accept_result) = server_result {
        assert!(accept_result.is_ok(), "{accept_result:?}");
    }
}

#[test]
fn handshake_errors_render_actionable_fail_closed_messages() {
    assert_eq!(
        WebDriverBiDiWebSocketHandshakeError::InvalidClientKey.to_string(),
        "WebDriver BiDi WebSocket client key is not canonical base64 for exactly 16 bytes"
    );
    assert_eq!(
        WebDriverBiDiWebSocketHandshakeError::TlsRequired.to_string(),
        "WebDriver BiDi WebSocket target requires authenticated TLS before the opening request"
    );
}

#[test]
fn handshake_plan_rejects_tls_required_stream_and_noncanonical_client_keys() {
    let invalid_length = WebDriverBiDiWebSocketClientKey::new("dGhlIHNhbXBsZSBub25jZQ=");
    assert!(matches!(
        invalid_length,
        Err(WebDriverBiDiWebSocketHandshakeError::InvalidClientKey)
    ));
    let invalid_character = WebDriverBiDiWebSocketClientKey::new("dGhlIHNhbXBsZSBub25jZ!==");
    assert!(matches!(
        invalid_character,
        Err(WebDriverBiDiWebSocketHandshakeError::InvalidClientKey)
    ));
    let invalid_padding_bits = WebDriverBiDiWebSocketClientKey::new("dGhlIHNhbXBsZSBub25jZR==");
    assert!(matches!(
        invalid_padding_bits,
        Err(WebDriverBiDiWebSocketHandshakeError::InvalidClientKey)
    ));
    let invalid_padding_character =
        WebDriverBiDiWebSocketClientKey::new("dGhlIHNhbXBsZSBub25jZQA=");
    assert!(matches!(
        invalid_padding_character,
        Err(WebDriverBiDiWebSocketHandshakeError::InvalidClientKey)
    ));

    let listener = TcpListener::bind(("127.0.0.1", 0));
    assert!(listener.is_ok(), "{listener:?}");
    let Ok(listener) = listener else {
        return;
    };
    let local_addr = listener.local_addr();
    assert!(local_addr.is_ok(), "{local_addr:?}");
    let Ok(local_addr) = local_addr else {
        return;
    };
    let server = thread::spawn(move || listener.accept().map(|_| ()));

    let endpoint = format!("wss://{local_addr}/session/{SESSION_ID}");
    let connection = connect(&endpoint);
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY);
    assert!(key.is_ok(), "{key:?}");
    let Ok(key) = key else {
        return;
    };
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connection, key);
    assert!(matches!(
        plan,
        Err(WebDriverBiDiWebSocketHandshakeError::TlsRequired)
    ));

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    if let Ok(accept_result) = server_result {
        assert!(accept_result.is_ok(), "{accept_result:?}");
    }
}
