use std::{
    io::{self, Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE, MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketHandshakeResponseError, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketOpeningWriteError,
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

fn read_client_text_frame(mut stream: std::net::TcpStream) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    assert_eq!(header[0], 0x81);
    assert_ne!(header[1] & 0x80, 0);
    let payload_length = usize::from(header[1] & 0x7f);
    assert!(payload_length < 126);
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
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
    let (release_tx, release_rx) = mpsc::channel();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: invalid\r\n\r\n",
        )?;
        release_rx
            .recv()
            .map_err(|error| io::Error::other(error.to_string()))
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

    let response = written.read_opening_response(Duration::from_millis(500));
    let released = release_tx.send(());
    assert!(released.is_ok(), "{released:?}");
    assert!(matches!(
        response,
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

#[test]
fn established_stream_writes_masked_text_and_reads_unmasked_text_frames() {
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
        let (mut stream, _) = listener.accept()?;
        read_opening_request(stream.try_clone()?)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let client_payload = read_client_text_frame(stream.try_clone()?)?;
        stream.write_all(b"\x89\x00\x81\x08{\"id\":2}")?;
        Ok(client_payload)
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

    let established = established.write_text_frame(
        r#"{"id":1}"#,
        WebDriverBiDiWebSocketMaskKey::new([0x37, 0xfa, 0x21, 0x3d]),
        Duration::from_millis(500),
    );
    assert!(established.is_ok(), "{established:?}");
    let Ok(established) = established else {
        return;
    };
    let ping = established.read_frame(Duration::from_millis(500));
    assert!(ping.is_ok(), "{ping:?}");
    let Ok((established, ping)) = ping else {
        return;
    };
    assert!(ping.fin());
    assert_eq!(ping.opcode(), 0x9);
    assert!(ping.payload().is_empty());

    let received = established.read_frame(Duration::from_millis(500));
    assert!(received.is_ok(), "{received:?}");
    let Ok((_established, frame)) = received else {
        return;
    };
    assert!(frame.fin());
    assert_eq!(frame.opcode(), 0x1);
    assert_eq!(frame.payload(), br#"{"id":2}"#);

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    if let Ok(client_payload) = server_result {
        assert!(client_payload.is_ok(), "{client_payload:?}");
        if let Ok(client_payload) = client_payload {
            assert_eq!(client_payload, br#"{"id":1}"#);
        }
    }
}

#[test]
fn established_stream_rejects_oversized_client_text_frames() {
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
        read_opening_request(stream.try_clone()?)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
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
    let payload = "x".repeat(MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE + 1);
    let result = established.write_text_frame(
        &payload,
        WebDriverBiDiWebSocketMaskKey::new([0x37, 0xfa, 0x21, 0x3d]),
        Duration::from_millis(500),
    );
    assert!(matches!(
        result,
        Err(WebDriverBiDiWebSocketFrameError::FrameTooLarge {
            payload_bytes,
            maximum_bytes,
        }) if payload_bytes == MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE + 1
            && maximum_bytes == MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE
    ));
    assert!(server.join().is_ok());
}

#[test]
fn established_stream_propagates_frame_write_failures() {
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
        for _ in 0..2 {
            let (mut stream, _) = listener.accept()?;
            read_opening_request(stream.try_clone()?)?;
            stream.write_all(
                b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
            )?;
        }
        Ok(())
    });

    for (frame_timeout, invalid_timeout) in
        [(Duration::ZERO, true), (Duration::from_nanos(1), false)]
    {
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
        let result = established.write_text_frame(
            "x",
            WebDriverBiDiWebSocketMaskKey::new([0x37, 0xfa, 0x21, 0x3d]),
            frame_timeout,
        );
        if invalid_timeout {
            assert!(matches!(
                result,
                Err(WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout { .. })
            ));
        } else {
            assert!(matches!(
                result,
                Err(WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut { .. })
            ));
        }
    }
    assert!(server.join().is_ok());
}

#[test]
fn established_stream_propagates_frame_read_failures() {
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
        read_opening_request(stream.try_clone()?)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        stream.write_all(b"\x81\x01")?;
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
    assert!(established.read_frame(Duration::from_millis(500)).is_err());
    assert!(server.join().is_ok());
}

#[test]
fn established_stream_rejects_invalid_read_frame_deadline() {
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
        read_opening_request(stream.try_clone()?)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
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
    assert!(matches!(
        established.read_frame(Duration::ZERO),
        Err(WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout { .. })
    ));
    assert!(server.join().is_ok());
}
