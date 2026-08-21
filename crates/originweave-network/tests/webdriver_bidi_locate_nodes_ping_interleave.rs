use std::{
    error::Error,
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
    WebDriverBiDiLocateNodesExchangeError, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const RESPONSE_DOCUMENT: &str =
    r#"{"type":"success","id":7,"result":{"nodes":[{"type":"node","sharedId":"shared-1"}]}}"#;
const PING_PAYLOAD: &[u8] = b"keepalive";

type EstablishedPingServer = (
    originweave_network::WebDriverBiDiWebSocketEstablished,
    thread::JoinHandle<io::Result<()>>,
);

fn connect(
    endpoint: &str,
) -> Result<originweave_network::WebDriverBiDiTcpConnection, Box<dyn Error>> {
    let admitted = WebDriverBiDiWebSocketEndpoint::new(endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?;
    Ok(plan.connect()?)
}

fn read_opening_request(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    let mut buffer = [0_u8; 512];
    while !request.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "client opening request ended before the header terminator",
            ));
        }
        request.extend_from_slice(&buffer[..count]);
    }
    Ok(())
}

fn read_masked_client_frame(
    stream: &mut TcpStream,
    expected_first_byte: u8,
) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != expected_first_byte || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client frame did not have the expected final opcode and masking bit",
        ));
    }
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
    if expected_first_byte == 0x8a && payload_length > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Pong payload exceeded the RFC 6455 control-frame bound",
        ));
    }
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
}

fn locate_nodes_command() -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Checkout"), 2)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        7,
        "top-level-context",
        &query,
    )?)
}

fn establish_with_ping(keep_open: Duration) -> Result<EstablishedPingServer, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let _command = read_masked_client_frame(&mut stream, 0x81)?;
        let ping_length = u8::try_from(PING_PAYLOAD.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "test Ping payload exceeded one-byte length",
            )
        })?;
        stream.write_all(&[0x89, ping_length])?;
        stream.write_all(PING_PAYLOAD)?;
        thread::sleep(keep_open);
        Ok(())
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    Ok((established, server))
}

fn join_ping_server(server: thread::JoinHandle<io::Result<()>>) -> Result<(), Box<dyn Error>> {
    let result = server
        .join()
        .map_err(|_| io::Error::other("Ping failure test server panicked"))?;
    Ok(result?)
}

#[test]
fn locate_nodes_exchange_answers_ping_and_ignores_unsolicited_pong_before_response()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<(Vec<u8>, Vec<u8>)> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let command = read_masked_client_frame(&mut stream, 0x81)?;
        let ping_length = u8::try_from(PING_PAYLOAD.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "test Ping payload exceeded one-byte length",
            )
        })?;
        stream.write_all(&[0x89, ping_length])?;
        stream.write_all(PING_PAYLOAD)?;
        let pong = read_masked_client_frame(&mut stream, 0x8a)?;
        stream.write_all(&[0x8a, 0])?;
        let response_length = u8::try_from(RESPONSE_DOCUMENT.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "test response exceeded one-byte length",
            )
        })?;
        stream.write_all(&[0x81, response_length])?;
        stream.write_all(RESPONSE_DOCUMENT.as_bytes())?;
        Ok((command, pong))
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    let command = locate_nodes_command()?;
    let expected_command = command.as_json().as_bytes().to_vec();
    let mut pong_keys = [WebDriverBiDiWebSocketMaskKey::new([0x51, 0x52, 0x53, 0x54])].into_iter();
    let exchanged = established.exchange_locate_nodes(
        command,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || pong_keys.next(),
        Duration::from_millis(500),
    );

    let server_result = server
        .join()
        .map_err(|_| io::Error::other("interleaved control-frame test server panicked"))?;
    assert!(exchanged.is_ok(), "{exchanged:?}");
    assert!(server_result.is_ok(), "{server_result:?}");
    let (received_command, received_pong) = server_result?;
    assert_eq!(received_command, expected_command);
    assert_eq!(received_pong, PING_PAYLOAD);

    let (established, result) = exchanged?;
    assert_eq!(result.command_id(), 7);
    assert_eq!(result.nodes().len(), 1);
    assert_eq!(result.nodes()[0].shared_id(), "shared-1");
    assert_eq!(
        established
            .transport_evidence()
            .verified_peer()
            .socket_addr(),
        local_addr
    );
    Ok(())
}

#[test]
fn locate_nodes_exchange_fails_closed_when_ping_entropy_is_unavailable()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_ping(Duration::from_millis(100))?;
    let exchanged = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        Duration::from_millis(500),
    );

    let error = exchanged
        .err()
        .ok_or_else(|| io::Error::other("Ping without masking entropy unexpectedly succeeded"))?;
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi locateNodes exchange received Ping without a fresh caller-supplied Pong masking key"
    );
    join_ping_server(server)
}

#[test]
fn locate_nodes_exchange_charges_ping_callback_time_to_exchange_deadline()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_ping(Duration::from_millis(650))?;
    let pong_key = WebDriverBiDiWebSocketMaskKey::new([0x51, 0x52, 0x53, 0x54]);
    let exchanged = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || {
            thread::sleep(Duration::from_millis(550));
            Some(pong_key)
        },
        Duration::from_millis(500),
    );

    let error = exchanged.err().ok_or_else(|| {
        io::Error::other("slow Ping callback unexpectedly reset the exchange deadline")
    })?;
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi locateNodes exchange exhausted its 500ms end-to-end deadline before the next operation"
    );
    join_ping_server(server)
}

#[test]
fn locate_nodes_exchange_bounds_valid_interleaved_control_frames() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let _command = read_masked_client_frame(&mut stream, 0x81)?;

        let mut frames = Vec::with_capacity(
            (MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE + 1) * 2 + RESPONSE_DOCUMENT.len() + 2,
        );
        for _ in 0..=MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE {
            frames.extend_from_slice(&[0x8a, 0]);
        }
        let response_length = u8::try_from(RESPONSE_DOCUMENT.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "test response exceeded one-byte length",
            )
        })?;
        frames.extend_from_slice(&[0x81, response_length]);
        frames.extend_from_slice(RESPONSE_DOCUMENT.as_bytes());
        stream.write_all(&frames)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    let exchanged = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        Duration::from_millis(500),
    );

    let server_result = server
        .join()
        .map_err(|_| io::Error::other("control-frame limit test server panicked"))?;
    assert!(server_result.is_ok(), "{server_result:?}");
    let error = exchanged
        .err()
        .ok_or_else(|| io::Error::other("control-frame flood unexpectedly reached response"))?;
    assert!(matches!(
        error,
        WebDriverBiDiLocateNodesExchangeError::ControlFrameLimitExceeded {
            maximum_control_frames,
        } if maximum_control_frames == MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE
    ));
    Ok(())
}
