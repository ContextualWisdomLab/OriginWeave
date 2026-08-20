use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
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

fn read_opening_request(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
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

fn read_client_text_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    assert_eq!(header[0], 0x81);
    assert_ne!(header[1] & 0x80, 0);

    let payload_length = match header[1] & 0x7f {
        value @ 0..=125 => usize::from(value),
        126 => {
            let mut extended = [0_u8; 2];
            stream.read_exact(&mut extended)?;
            usize::from(u16::from_be_bytes(extended))
        }
        127 => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "test fixture does not admit 64-bit client frame lengths",
            ));
        }
        _ => unreachable!("7-bit WebSocket payload marker"),
    };

    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
}

fn write_server_text_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    let payload_length = u8::try_from(payload.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "test response must fit one short WebSocket text frame",
        )
    })?;
    if payload_length > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "test response must fit one short WebSocket text frame",
        ));
    }
    stream.write_all(&[0x81, payload_length])?;
    stream.write_all(payload)
}

#[test]
fn established_stream_exchanges_exact_locate_nodes_command_and_correlates_wire_result() {
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
        let request = read_opening_request(&mut stream)?;
        if !request.ends_with(b"\r\n\r\n") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "client opening request was incomplete",
            ));
        }
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let command = read_client_text_frame(&mut stream)?;
        write_server_text_frame(&mut stream, RESPONSE_DOCUMENT.as_bytes())?;
        Ok(command)
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
    let expected_command = command.as_json().as_bytes().to_vec();

    let exchanged = established.exchange_locate_nodes(
        command,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        Duration::from_millis(500),
    );
    assert!(exchanged.is_ok(), "{exchanged:?}");
    let Ok((established, result)) = exchanged else {
        return;
    };

    assert_eq!(result.command_id(), 7);
    assert_eq!(result.browsing_context(), "top-level-context");
    assert_eq!(result.max_node_count(), 2);
    assert_eq!(result.nodes().len(), 1);
    assert_eq!(result.nodes()[0].shared_id(), "shared-1");
    assert_eq!(
        established
            .transport_evidence()
            .verified_peer()
            .socket_addr(),
        local_addr
    );
    assert_eq!(
        established
            .transport_evidence()
            .verified_peer()
            .session_id(),
        SESSION_ID
    );

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    if let Ok(command_result) = server_result {
        assert!(command_result.is_ok(), "{command_result:?}");
        if let Ok(actual_command) = command_result {
            assert_eq!(actual_command, expected_command);
        }
    }
}
