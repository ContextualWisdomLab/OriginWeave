use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiLocateNodesExchangeError, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const RESPONSE_DOCUMENT: &str =
    r#"{"type":"success","id":7,"result":{"nodes":[{"type":"node","sharedId":"shared-1"}]}}"#;
const MISMATCHED_RESPONSE_DOCUMENT: &str = r#"{"type":"success","id":8,"result":{"nodes":[]}}"#;

type ServerHandle = thread::JoinHandle<io::Result<Vec<u8>>>;
type EstablishedFixture =
    Result<(SocketAddr, WebDriverBiDiWebSocketEstablished, ServerHandle), Box<dyn Error>>;

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

fn server_frame(first_byte: u8, payload: &[u8]) -> io::Result<Vec<u8>> {
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
    let mut frame = vec![first_byte, payload_length];
    frame.extend_from_slice(payload);
    Ok(frame)
}

fn establish_with_server_frame(response_frame: &[u8]) -> EstablishedFixture {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let response_frame = response_frame.to_vec();
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
        stream.write_all(&response_frame)?;
        Ok(command)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint), key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    Ok((local_addr, established, server))
}

fn locate_nodes_command() -> WebDriverBiDiLocateNodesCommand {
    let name = "x".repeat(512);
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some(&name), 2);
    assert!(query.is_ok(), "{query:?}");
    let Ok(query) = query else {
        unreachable!("asserted valid test query")
    };
    let command = WebDriverBiDiLocateNodesCommand::new(7, "top-level-context", &query);
    assert!(command.is_ok(), "{command:?}");
    let Ok(command) = command else {
        unreachable!("asserted valid test command")
    };
    command
}

fn exchange_error(
    response_frame: &[u8],
    frame_timeout: Duration,
    server_must_receive_command: bool,
) -> WebDriverBiDiLocateNodesExchangeError {
    let fixture = establish_with_server_frame(response_frame);
    assert!(fixture.is_ok(), "{fixture:?}");
    let Ok((_, established, server)) = fixture else {
        unreachable!("asserted valid test exchange fixture")
    };
    let error = established.exchange_locate_nodes(
        locate_nodes_command(),
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        frame_timeout,
    );
    assert!(error.is_err(), "{error:?}");
    let Err(error) = error else {
        unreachable!("asserted failing test exchange")
    };
    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    let Ok(server_result) = server_result else {
        unreachable!("asserted joined test server")
    };
    assert_eq!(
        server_result.is_ok(),
        server_must_receive_command,
        "test server command receipt did not match the exchange boundary"
    );
    error
}

#[test]
fn established_stream_exchanges_exact_locate_nodes_command_and_correlates_wire_result() {
    let response_frame = server_frame(0x81, RESPONSE_DOCUMENT.as_bytes());
    assert!(response_frame.is_ok(), "{response_frame:?}");
    let Ok(response_frame) = response_frame else {
        return;
    };
    let fixture = establish_with_server_frame(&response_frame);
    assert!(fixture.is_ok(), "{fixture:?}");
    let Ok((local_addr, established, server)) = fixture else {
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
        &mut || None,
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

#[test]
fn exchange_deadline_is_not_reset_after_the_frame_write() {
    let response_frame = server_frame(0x81, RESPONSE_DOCUMENT.as_bytes());
    assert!(response_frame.is_ok(), "{response_frame:?}");
    let Ok(response_frame) = response_frame else {
        return;
    };
    let fixture = establish_with_server_frame(&response_frame);
    assert!(fixture.is_ok(), "{fixture:?}");
    let Ok((_, established, server)) = fixture else {
        return;
    };

    let error = established.exchange_locate_nodes(
        locate_nodes_command(),
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        Duration::from_micros(20),
    );
    assert!(error.is_err(), "{error:?}");
    let Err(error) = error else {
        unreachable!("asserted exhausted exchange deadline")
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi locateNodes exchange exhausted its 20µs end-to-end deadline before the next operation"
    );

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
}

#[test]
fn exchange_rejects_a_non_final_or_non_text_response_frame() {
    for (first_byte, expected_fin, expected_opcode) in
        [(0x01_u8, false, 0x01_u8), (0x82_u8, true, 0x02_u8)]
    {
        let response_frame = server_frame(first_byte, &[]);
        assert!(response_frame.is_ok(), "{response_frame:?}");
        let Ok(response_frame) = response_frame else {
            return;
        };
        let fixture = establish_with_server_frame(&response_frame);
        assert!(fixture.is_ok(), "{fixture:?}");
        let Ok((_, established, server)) = fixture else {
            return;
        };

        let error = established.exchange_locate_nodes(
            locate_nodes_command(),
            WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
            &mut || None,
            Duration::from_millis(500),
        );
        assert!(error.is_err(), "{error:?}");
        let Err(error) = error else {
            unreachable!("asserted invalid response frame failure")
        };
        assert!(matches!(
            error,
            WebDriverBiDiLocateNodesExchangeError::UnexpectedResponseFrame {
                fin,
                opcode,
            } if fin == expected_fin && opcode == expected_opcode
        ));

        let server_result = server.join();
        assert!(server_result.is_ok(), "{server_result:?}");
        if let Ok(command_result) = server_result {
            assert!(command_result.is_ok(), "{command_result:?}");
        }
    }
}

#[test]
fn exchange_preserves_frame_document_and_response_admission_boundaries() {
    let write_error = exchange_error(&[], Duration::ZERO, false);
    assert!(matches!(
        write_error,
        WebDriverBiDiLocateNodesExchangeError::ExchangeDeadlineExceeded {
            exchange_timeout
        } if exchange_timeout.is_zero()
    ));

    let read_error = exchange_error(&[], Duration::from_millis(500), true);
    assert!(matches!(
        read_error,
        WebDriverBiDiLocateNodesExchangeError::Frame(_)
    ));

    let invalid_utf8_frame = server_frame(0x81, &[0xff]);
    assert!(invalid_utf8_frame.is_ok(), "{invalid_utf8_frame:?}");
    let Ok(invalid_utf8_frame) = invalid_utf8_frame else {
        return;
    };
    let document_error = exchange_error(&invalid_utf8_frame, Duration::from_millis(500), true);
    assert!(matches!(
        document_error,
        WebDriverBiDiLocateNodesExchangeError::ResponseDocument(_)
    ));

    let mismatched_frame = server_frame(0x81, MISMATCHED_RESPONSE_DOCUMENT.as_bytes());
    assert!(mismatched_frame.is_ok(), "{mismatched_frame:?}");
    let Ok(mismatched_frame) = mismatched_frame else {
        return;
    };
    let response_error = exchange_error(&mismatched_frame, Duration::from_millis(500), true);
    assert!(matches!(
        response_error,
        WebDriverBiDiLocateNodesExchangeError::LocateNodesResponse(_)
    ));
}
