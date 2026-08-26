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
const PING_PAYLOAD: &[u8] = b"ping-between-fragments";

type EstablishedFragmentServer = (
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

fn read_masked_client_frame(stream: &mut TcpStream, expected_first_byte: u8) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != expected_first_byte || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client frame did not use the expected final masked opcode",
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
            let mut extended = [0_u8; 8];
            stream.read_exact(&mut extended)?;
            let payload_length = u64::from_be_bytes(extended);
            usize::try_from(payload_length).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "client frame length did not fit usize",
                )
            })?
        }
        marker => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected WebSocket length marker {marker}"),
            ));
        }
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

fn read_masked_client_text_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    read_masked_client_frame(stream, 0x81)
}

fn locate_nodes_command() -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Checkout"), 2)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        7,
        "top-level-context",
        &query,
    )?)
}

fn response_fragments() -> io::Result<(Vec<u8>, Vec<u8>)> {
    let response = RESPONSE_DOCUMENT.as_bytes();
    let split = response.len() / 2;
    let first = &response[..split];
    let second = &response[split..];
    let first_length = u8::try_from(first.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "first response fragment exceeded one-byte test length",
        )
    })?;
    let second_length = u8::try_from(second.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "second response fragment exceeded one-byte test length",
        )
    })?;

    let mut first_frame = Vec::with_capacity(first.len() + 2);
    first_frame.extend_from_slice(&[0x01, first_length]);
    first_frame.extend_from_slice(first);
    let mut second_frame = Vec::with_capacity(second.len() + 2);
    second_frame.extend_from_slice(&[0x80, second_length]);
    second_frame.extend_from_slice(second);
    Ok((first_frame, second_frame))
}

fn establish_with_fragmented_response() -> Result<EstablishedFragmentServer, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let _command = read_masked_client_text_frame(&mut stream)?;
        let (first_frame, second_frame) = response_fragments()?;
        stream.write_all(&first_frame)?;
        stream.write_all(&second_frame)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    Ok((established, server))
}

fn establish_with_ping_between_fragments() -> Result<EstablishedFragmentServer, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let _command = read_masked_client_text_frame(&mut stream)?;
        let (first_frame, second_frame) = response_fragments()?;
        stream.write_all(&first_frame)?;
        let ping_length = u8::try_from(PING_PAYLOAD.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "test Ping payload exceeded 125 bytes")
        })?;
        stream.write_all(&[0x89, ping_length])?;
        stream.write_all(PING_PAYLOAD)?;

        let pong_payload = read_masked_client_frame(&mut stream, 0x8a)?;
        if pong_payload != PING_PAYLOAD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "client Pong did not echo the interleaved Ping payload",
            ));
        }
        stream.write_all(&second_frame)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    Ok((established, server))
}

fn assert_successful_exchange(
    exchanged: Result<
        (
            originweave_network::WebDriverBiDiWebSocketEstablished,
            originweave_core::ValidatedWebDriverBiDiLocateNodesResult,
        ),
        originweave_network::WebDriverBiDiLocateNodesExchangeError,
    >,
    server: thread::JoinHandle<io::Result<()>>,
) -> Result<(), Box<dyn Error>> {
    let server_result = server
        .join()
        .map_err(|_| io::Error::other("fragmentation regression server panicked"))?;
    assert!(server_result.is_ok(), "{server_result:?}");
    assert!(exchanged.is_ok(), "{exchanged:?}");

    let (_, result) = exchanged?;
    assert_eq!(result.command_id(), 7);
    assert_eq!(result.nodes().len(), 1);
    assert_eq!(result.nodes()[0].shared_id(), "shared-1");
    Ok(())
}

#[test]
fn locate_nodes_exchange_reassembles_fragmented_text_response() -> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_fragmented_response()?;
    let command = locate_nodes_command()?;
    let mut no_pong_keys = || None;
    let exchanged = established.exchange_locate_nodes(
        command,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut no_pong_keys,
        Duration::from_millis(500),
    );
    assert_successful_exchange(exchanged, server)
}

#[test]
fn locate_nodes_exchange_handles_ping_between_response_fragments() -> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_ping_between_fragments()?;
    let command = locate_nodes_command()?;
    let mut pong_keys = || Some(WebDriverBiDiWebSocketMaskKey::new([0x55, 0x66, 0x77, 0x88]));
    let exchanged = established.exchange_locate_nodes(
        command,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut pong_keys,
        Duration::from_millis(500),
    );
    assert_successful_exchange(exchanged, server)
}
