use std::{
    io::{self, Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketHandshakeResponseError, WebDriverBiDiWebSocketOpeningWriteError,
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

fn read_opening_request(mut stream: std::net::TcpStream) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    let mut buffer = [0_u8; 512];
    while !request.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..count]);
    }
    Ok(request)
}

#[test]
fn bounded_opening_write_sends_exact_request_and_preserves_transport_evidence() {
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
    let server = thread::spawn(move || {
        let accepted = listener.accept()?;
        read_opening_request(accepted.0)
    });

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
    let write_timeout = Duration::from_millis(500);
    let written = plan.write_opening_request(write_timeout);
    assert!(written.is_ok(), "{written:?}");
    let Ok(written) = written else {
        return;
    };

    assert_eq!(written.request_byte_count(), expected.len());
    assert_eq!(written.write_timeout(), write_timeout);
    assert_eq!(written.client_key().as_str(), RFC6455_SAMPLE_KEY);
    assert_eq!(
        written.transport_evidence().verified_peer().socket_addr(),
        local_addr
    );
    assert_eq!(
        written.transport_evidence().verified_peer().session_id(),
        SESSION_ID
    );
    assert_eq!(written.transport_evidence().attempt_number(), 1);
    let debug = format!("{written:?}");
    assert!(debug.contains("WebDriverBiDiWebSocketOpeningRequestSent"));
    assert!(!debug.contains(RFC6455_SAMPLE_KEY));

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    if let Ok(received) = server_result {
        assert!(received.is_ok(), "{received:?}");
        if let Ok(received) = received {
            assert_eq!(received, expected.as_bytes());
        }
    }
}

#[test]
fn opening_write_rejects_zero_and_excessive_deadlines_before_success_evidence() {
    for timeout in [
        Duration::ZERO,
        MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT + Duration::from_nanos(1),
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
        let server = thread::spawn(move || listener.accept().map(|_| ()));

        let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
        let connection = connect(&endpoint);
        let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY);
        assert!(key.is_ok(), "{key:?}");
        let Ok(key) = key else {
            continue;
        };
        let plan = WebDriverBiDiWebSocketHandshakePlan::new(connection, key);
        assert!(plan.is_ok(), "{plan:?}");
        let Ok(plan) = plan else {
            continue;
        };

        let result = plan.write_opening_request(timeout);
        assert!(matches!(
            result,
            Err(WebDriverBiDiWebSocketOpeningWriteError::InvalidWriteTimeout {
                write_timeout,
                maximum_timeout: MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT,
            }) if write_timeout == timeout
        ));

        let server_result = server.join();
        assert!(server_result.is_ok(), "{server_result:?}");
        if let Ok(accept_result) = server_result {
            assert!(accept_result.is_ok(), "{accept_result:?}");
        }
    }
}

#[test]
fn opening_response_requires_rfc6455_switching_protocols_and_matching_accept() {
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
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let mut close_probe = [0_u8; 1];
        let _ = stream.read(&mut close_probe);
        Ok(())
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY);
    assert!(key.is_ok(), "{key:?}");
    let Ok(key) = key else {
        return;
    };
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint), key);
    assert!(plan.is_ok(), "{plan:?}");
    let Ok(plan) = plan else {
        return;
    };
    let written = plan.write_opening_request(Duration::from_millis(500));
    assert!(written.is_ok(), "{written:?}");
    let Ok(written) = written else {
        return;
    };

    let established = written.read_opening_response(Duration::from_millis(500));
    assert!(established.is_ok(), "{established:?}");
    let Ok(established) = established else {
        return;
    };
    assert_eq!(established.response_status(), 101);
    assert!(established.response_byte_count() > 0);
    assert!(established.request_byte_count() > 0);
    assert_eq!(established.response_timeout(), Duration::from_millis(500));
    assert_eq!(established.write_timeout(), Duration::from_millis(500));
    assert_eq!(established.client_key().as_str(), RFC6455_SAMPLE_KEY);
    assert_eq!(
        established
            .transport_evidence()
            .verified_peer()
            .socket_addr(),
        local_addr
    );
    let debug = format!("{established:?}");
    assert!(debug.contains("WebDriverBiDiWebSocketEstablished"));
    assert!(!debug.contains(RFC6455_SAMPLE_KEY));
    drop(established);
    assert!(server.join().is_ok());
}

#[test]
fn opening_response_rejects_a_mismatched_accept_value() {
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
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: invalid\r\n\r\n",
        )
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY);
    assert!(key.is_ok(), "{key:?}");
    let Ok(key) = key else {
        return;
    };
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint), key);
    assert!(plan.is_ok(), "{plan:?}");
    let Ok(plan) = plan else {
        return;
    };
    let written = plan.write_opening_request(Duration::from_millis(500));
    assert!(written.is_ok(), "{written:?}");
    let Ok(written) = written else {
        return;
    };

    assert!(matches!(
        written.read_opening_response(Duration::from_millis(500)),
        Err(WebDriverBiDiWebSocketHandshakeResponseError::AcceptMismatch)
    ));
    assert!(server.join().is_ok());
}

#[test]
fn opening_response_rejects_zero_and_excessive_deadlines_before_socket_mode_change() {
    for timeout in [
        Duration::ZERO,
        originweave_network::MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT + Duration::from_nanos(1),
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
        let server = thread::spawn(move || listener.accept().map(|_| ()));

        let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
        let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY);
        assert!(key.is_ok(), "{key:?}");
        let Ok(key) = key else {
            continue;
        };
        let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint), key);
        assert!(plan.is_ok(), "{plan:?}");
        let Ok(plan) = plan else {
            continue;
        };
        let written = plan.write_opening_request(Duration::from_millis(500));
        assert!(written.is_ok(), "{written:?}");
        let Ok(written) = written else {
            continue;
        };

        assert!(matches!(
            written.read_opening_response(timeout),
            Err(WebDriverBiDiWebSocketHandshakeResponseError::InvalidResponseTimeout {
                response_timeout,
                maximum_timeout,
            }) if response_timeout == timeout
                && maximum_timeout
                    == originweave_network::MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT
        ));
        assert!(server.join().is_ok());
    }
}
