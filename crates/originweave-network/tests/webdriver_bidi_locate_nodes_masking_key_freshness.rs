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
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const RESPONSE_DOCUMENT: &str =
    r#"{"type":"success","id":7,"result":{"nodes":[{"type":"node","sharedId":"shared-1"}]}}"#;
const PING_PAYLOAD: &[u8] = b"fresh-mask";
const COMMAND_MASK: WebDriverBiDiWebSocketMaskKey =
    WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]);
const PONG_MASK: WebDriverBiDiWebSocketMaskKey =
    WebDriverBiDiWebSocketMaskKey::new([0x51, 0x52, 0x53, 0x54]);
const REUSED_MASK_ERROR: &str =
    "WebDriver BiDi locateNodes exchange refused a Pong masking key already used by this exchange";

type EstablishedServer = (
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

fn read_masked_client_frame(stream: &mut TcpStream, expected_first_byte: u8) -> io::Result<()> {
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
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
    Ok(())
}

fn write_ping(stream: &mut TcpStream) -> io::Result<()> {
    let payload_length = u8::try_from(PING_PAYLOAD.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "test Ping payload exceeded one-byte length",
        )
    })?;
    stream.write_all(&[0x89, payload_length])?;
    stream.write_all(PING_PAYLOAD)
}

fn write_response(stream: &mut TcpStream) -> io::Result<()> {
    let payload_length = u8::try_from(RESPONSE_DOCUMENT.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "test response exceeded one-byte length",
        )
    })?;
    stream.write_all(&[0x81, payload_length])?;
    stream.write_all(RESPONSE_DOCUMENT.as_bytes())
}

fn locate_nodes_command() -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Checkout"), 2)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        7,
        "top-level-context",
        &query,
    )?)
}

fn establish_with_ping_sequence(
    read_first_pong: bool,
) -> Result<EstablishedServer, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        read_masked_client_frame(&mut stream, 0x81)?;
        write_ping(&mut stream)?;
        if read_first_pong {
            read_masked_client_frame(&mut stream, 0x8a)?;
            write_ping(&mut stream)?;
        }
        thread::sleep(Duration::from_millis(150));
        Ok(())
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let client_key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, client_key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    Ok((established, server))
}

fn join_server(server: thread::JoinHandle<io::Result<()>>) -> Result<(), Box<dyn Error>> {
    let result = server
        .join()
        .map_err(|_| io::Error::other("masking-key freshness test server panicked"))?;
    Ok(result?)
}

#[test]
fn locate_nodes_exchange_rejects_pong_mask_reused_from_command_frame() -> Result<(), Box<dyn Error>>
{
    let (established, server) = establish_with_ping_sequence(false)?;
    let exchanged = established.exchange_locate_nodes(
        locate_nodes_command()?,
        COMMAND_MASK,
        &mut || Some(COMMAND_MASK),
        Duration::from_millis(500),
    );

    let error = exchanged.err().ok_or_else(|| {
        io::Error::other("reusing the command masking key for Pong unexpectedly succeeded")
    })?;
    assert_eq!(error.to_string(), REUSED_MASK_ERROR);
    join_server(server)
}

#[test]
fn locate_nodes_exchange_rejects_pong_mask_reused_from_prior_pong() -> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_ping_sequence(true)?;
    let mut keys = [PONG_MASK, PONG_MASK].into_iter();
    let exchanged = established.exchange_locate_nodes(
        locate_nodes_command()?,
        COMMAND_MASK,
        &mut || keys.next(),
        Duration::from_millis(500),
    );

    let error = exchanged.err().ok_or_else(|| {
        io::Error::other("reusing a prior Pong masking key unexpectedly succeeded")
    })?;
    assert_eq!(error.to_string(), REUSED_MASK_ERROR);
    join_server(server)
}

#[test]
fn locate_nodes_exchange_allows_non_adjacent_random_mask_collision() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        read_masked_client_frame(&mut stream, 0x81)?;
        write_ping(&mut stream)?;
        read_masked_client_frame(&mut stream, 0x8a)?;
        write_ping(&mut stream)?;
        read_masked_client_frame(&mut stream, 0x8a)?;
        write_response(&mut stream)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let client_key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, client_key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    let mut keys = [PONG_MASK, COMMAND_MASK].into_iter();
    let exchanged = established.exchange_locate_nodes(
        locate_nodes_command()?,
        COMMAND_MASK,
        &mut || keys.next(),
        Duration::from_millis(500),
    );

    let server_result = server
        .join()
        .map_err(|_| io::Error::other("non-adjacent masking-key collision server panicked"))?;
    assert!(server_result.is_ok(), "{server_result:?}");
    let (_, result) = exchanged?;
    assert_eq!(result.command_id(), 7);
    assert_eq!(result.nodes().len(), 1);
    assert_eq!(result.nodes()[0].shared_id(), "shared-1");
    Ok(())
}
