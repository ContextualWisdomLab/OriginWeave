use std::{
    io::{self, Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    MAX_WEBSOCKET_OPENING_READ_TIMEOUT, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketOpeningReadError,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const SERVER_OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";

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

fn read_opening_request(mut stream: &std::net::TcpStream) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    let mut byte = [0_u8; 1];
    while !request.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut byte)?;
        if count == 0 {
            break;
        }
        request.push(byte[0]);
    }
    stream.set_read_timeout(None)?;
    Ok(request)
}

fn opening_request_sent(
    local_addr: std::net::SocketAddr,
) -> originweave_network::WebDriverBiDiWebSocketOpeningRequestSent {
    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let connection = connect(&endpoint);
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY);
    assert!(key.is_ok(), "{key:?}");
    let Ok(key) = key else {
        unreachable!("asserted canonical sample key")
    };
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connection, key);
    assert!(plan.is_ok(), "{plan:?}");
    let Ok(plan) = plan else {
        unreachable!("asserted plain WebSocket handshake plan")
    };
    let sent = plan.write_opening_request(Duration::from_millis(500));
    assert!(sent.is_ok(), "{sent:?}");
    let Ok(sent) = sent else {
        unreachable!("asserted bounded opening write")
    };
    sent
}

#[test]
fn bounded_opening_response_read_preserves_exact_transport_and_request_evidence() {
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
    let server = thread::spawn(move || -> io::Result<Vec<u8>> {
        let accepted = listener.accept()?;
        let mut stream = accepted.0;
        let request = read_opening_request(&stream)?;
        stream.write_all(SERVER_OPENING_RESPONSE)?;
        Ok(request)
    });

    let sent = opening_request_sent(local_addr);
    let request_byte_count = sent.request_byte_count();
    let write_timeout = sent.write_timeout();
    let read_timeout = Duration::from_millis(500);
    let response = sent.read_opening_response(read_timeout);
    assert!(response.is_ok(), "{response:?}");
    let Ok(response) = response else {
        return;
    };

    assert_eq!(response.header_bytes(), SERVER_OPENING_RESPONSE);
    assert_eq!(response.read_timeout(), read_timeout);
    assert_eq!(response.request_byte_count(), request_byte_count);
    assert_eq!(response.write_timeout(), write_timeout);
    assert_eq!(response.client_key().as_str(), RFC6455_SAMPLE_KEY);
    assert_eq!(
        response.transport_evidence().verified_peer().socket_addr(),
        local_addr
    );
    assert_eq!(
        response.transport_evidence().verified_peer().session_id(),
        SESSION_ID
    );
    assert_eq!(response.transport_evidence().attempt_number(), 1);
    let debug = format!("{response:?}");
    assert!(debug.contains("WebDriverBiDiWebSocketOpeningResponseRead"));
    assert!(!debug.contains(RFC6455_SAMPLE_KEY));

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    if let Ok(received) = server_result {
        assert!(received.is_ok(), "{received:?}");
        if let Ok(received) = received {
            assert!(received.starts_with(b"GET /session/"));
            assert!(received.ends_with(b"\r\n\r\n"));
        }
    }
}

#[test]
fn opening_response_read_rejects_zero_and_excessive_deadlines_before_reading() {
    for timeout in [
        Duration::ZERO,
        MAX_WEBSOCKET_OPENING_READ_TIMEOUT + Duration::from_nanos(1),
    ] {
        let listener = TcpListener::bind(("127.0.0.1", 0));
        assert!(listener.is_ok(), "{listener:?}");
        let Ok(listener) = listener else {
            continue;
        };
        let local_addr = listener.local_addr();
        assert!(local_addr.is_ok(), "{local_addr:?}");
        let Ok(local_addr) = local_addr else {
            continue;
        };
        let server = thread::spawn(move || -> io::Result<Vec<u8>> {
            let accepted = listener.accept()?;
            read_opening_request(&accepted.0)
        });

        let sent = opening_request_sent(local_addr);
        let result = sent.read_opening_response(timeout);
        assert!(matches!(
            result,
            Err(WebDriverBiDiWebSocketOpeningReadError::InvalidReadTimeout {
                read_timeout,
                maximum_timeout: MAX_WEBSOCKET_OPENING_READ_TIMEOUT,
            }) if read_timeout == timeout
        ));

        let server_result = server.join();
        assert!(server_result.is_ok(), "{server_result:?}");
        if let Ok(received) = server_result {
            assert!(received.is_ok(), "{received:?}");
        }
    }
}
