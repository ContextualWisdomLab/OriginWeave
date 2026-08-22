use std::{
    io::{self, Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use originweave_core::{
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const RESPONSE_DOCUMENT: &str =
    r#"{"type":"success","id":7,"result":{"nodes":[{"type":"node","sharedId":"shared-1"}]}}"#;

fn read_client_text_frame(stream: &mut impl Read) -> io::Result<()> {
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    assert_eq!(header[0], 0x81);
    assert_ne!(header[1] & 0x80, 0);

    let payload_marker = header[1] & 0x7f;
    let payload_length = if payload_marker <= 125 {
        usize::from(payload_marker)
    } else if payload_marker == 126 {
        let mut extended = [0_u8; 2];
        stream.read_exact(&mut extended)?;
        usize::from(u16::from_be_bytes(extended))
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "test command unexpectedly used a 64-bit WebSocket payload length",
        ));
    };

    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
    Ok(())
}

#[test]
fn exchange_budget_above_per_frame_ceiling_remains_a_valid_end_to_end_budget() {
    let listener = TcpListener::bind(("127.0.0.1", 0));
    assert!(listener.is_ok(), "{listener:?}");
    let Ok(listener) = listener else {
        return;
    };
    let address = listener.local_addr();
    assert!(address.is_ok(), "{address:?}");
    let Ok(address) = address else {
        return;
    };
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;

        let mut opening = Vec::new();
        let mut byte = [0_u8; 1];
        while !opening.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte)?;
            opening.push(byte[0]);
        }
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;

        read_client_text_frame(&mut stream)?;

        let response = RESPONSE_DOCUMENT.as_bytes();
        let response_length = u8::try_from(response.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "test response unexpectedly exceeded short-frame length",
            )
        })?;
        stream.write_all(&[0x81, response_length])?;
        stream.write_all(response)?;
        Ok(())
    });

    let endpoint =
        WebDriverBiDiWebSocketEndpoint::new(&format!("ws://{address}/session/{SESSION_ID}"));
    assert!(endpoint.is_ok(), "{endpoint:?}");
    let Ok(endpoint) = endpoint else {
        return;
    };
    let correlated = endpoint.correlate_session_id(SESSION_ID);
    assert!(correlated.is_ok(), "{correlated:?}");
    let Ok(correlated) = correlated else {
        return;
    };
    let target = correlated.into_explicit_connect_target();
    assert!(target.is_ok(), "{target:?}");
    let Ok(target) = target else {
        return;
    };
    let connection_plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1);
    assert!(connection_plan.is_ok(), "{connection_plan:?}");
    let Ok(connection_plan) = connection_plan else {
        return;
    };
    let connection = connection_plan.connect();
    assert!(connection.is_ok(), "{connection:?}");
    let Ok(connection) = connection else {
        return;
    };
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY);
    assert!(key.is_ok(), "{key:?}");
    let Ok(key) = key else {
        return;
    };
    let handshake = WebDriverBiDiWebSocketHandshakePlan::new(connection, key);
    assert!(handshake.is_ok(), "{handshake:?}");
    let Ok(handshake) = handshake else {
        return;
    };
    let opening = handshake.write_opening_request(Duration::from_millis(500));
    assert!(opening.is_ok(), "{opening:?}");
    let Ok(opening) = opening else {
        return;
    };
    let established = opening.read_opening_response(Duration::from_millis(500));
    assert!(established.is_ok(), "{established:?}");
    let Ok(established) = established else {
        return;
    };

    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Checkout"), 2);
    assert!(query.is_ok(), "{query:?}");
    let Ok(query) = query else {
        return;
    };
    let command = WebDriverBiDiLocateNodesCommand::new(7, "top-level-context", &query);
    assert!(command.is_ok(), "{command:?}");
    let Ok(command) = command else {
        return;
    };

    let exchanged = established.exchange_locate_nodes(
        command,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        Duration::from_secs(6),
    );
    assert!(exchanged.is_ok(), "{exchanged:?}");

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    if let Ok(server_io) = server_result {
        assert!(server_io.is_ok(), "{server_io:?}");
    }
}
