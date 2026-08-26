use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiResponseDocumentAdmissionError,
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

type EstablishedFrameServer = (
    WebDriverBiDiWebSocketEstablished,
    thread::JoinHandle<io::Result<()>>,
);

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

fn read_masked_client_text_frame(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x81 || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected one final masked client text frame",
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
            usize::try_from(u64::from_be_bytes(extended)).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "client frame payload length did not fit usize",
                )
            })?
        }
        marker => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid client frame payload-length marker {marker}"),
            ));
        }
    };

    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
    Ok(())
}

fn server_frame(fin: bool, opcode: u8, payload: &[u8]) -> io::Result<Vec<u8>> {
    let first_byte = if fin { 0x80 | opcode } else { opcode };
    let mut frame = Vec::with_capacity(payload.len() + 10);
    frame.push(first_byte);

    if payload.len() <= 125 {
        frame.push(u8::try_from(payload.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "short server frame length did not fit u8",
            )
        })?);
    } else if payload.len() <= usize::from(u16::MAX) {
        frame.push(126);
        frame.extend_from_slice(
            &u16::try_from(payload.len())
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "medium server frame length did not fit u16",
                    )
                })?
                .to_be_bytes(),
        );
    } else {
        frame.push(127);
        frame.extend_from_slice(
            &u64::try_from(payload.len())
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "large server frame length did not fit u64",
                    )
                })?
                .to_be_bytes(),
        );
    }

    frame.extend_from_slice(payload);
    Ok(frame)
}

fn establish_with_frames(frames: Vec<Vec<u8>>) -> Result<EstablishedFrameServer, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        read_masked_client_text_frame(&mut stream)?;
        for frame in frames {
            stream.write_all(&frame)?;
        }
        Ok(())
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let admitted = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    Ok((established, server))
}

fn locate_nodes_command() -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Checkout"), 2)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        7,
        "top-level-context",
        &query,
    )?)
}

fn join_server(server: thread::JoinHandle<io::Result<()>>) -> Result<(), Box<dyn Error>> {
    server
        .join()
        .map_err(|_| io::Error::other("fragmentation boundary server panicked"))??;
    Ok(())
}

#[test]
fn three_fragment_response_reassembles() -> Result<(), Box<dyn Error>> {
    let response = RESPONSE_DOCUMENT.as_bytes();
    let first_end = response.len() / 3;
    let second_end = first_end * 2;
    let frames = vec![
        server_frame(false, 0x1, &response[..first_end])?,
        server_frame(false, 0x0, &response[first_end..second_end])?,
        server_frame(true, 0x0, &response[second_end..])?,
    ];
    let (established, server) = establish_with_frames(frames)?;
    let mut no_pong_keys = || None;
    let exchange = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut no_pong_keys,
        Duration::from_millis(500),
    );

    join_server(server)?;
    let (_, result) = exchange?;
    assert_eq!(result.command_id(), 7);
    assert_eq!(result.nodes().len(), 1);
    Ok(())
}

#[test]
fn oversized_initial_fragment_fails_closed() -> Result<(), Box<dyn Error>> {
    let payload = vec![b'{'; MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES + 1];
    let frames = vec![server_frame(false, 0x1, &payload)?];
    let (established, server) = establish_with_frames(frames)?;
    let mut no_pong_keys = || None;
    let exchange = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut no_pong_keys,
        Duration::from_millis(500),
    );

    join_server(server)?;
    assert!(matches!(
        exchange,
        Err(WebDriverBiDiLocateNodesExchangeError::ResponseDocument(
            WebDriverBiDiResponseDocumentAdmissionError::DocumentTooLarge
        ))
    ));
    Ok(())
}

#[test]
fn fragmented_response_over_budget_fails_closed() -> Result<(), Box<dyn Error>> {
    let first_payload = vec![b'{'; MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES];
    let frames = vec![
        server_frame(false, 0x1, &first_payload)?,
        server_frame(true, 0x0, b"}")?,
    ];
    let (established, server) = establish_with_frames(frames)?;
    let mut no_pong_keys = || None;
    let exchange = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut no_pong_keys,
        Duration::from_millis(500),
    );

    join_server(server)?;
    assert!(matches!(
        exchange,
        Err(WebDriverBiDiLocateNodesExchangeError::ResponseDocument(
            WebDriverBiDiResponseDocumentAdmissionError::DocumentTooLarge
        ))
    ));
    Ok(())
}

#[test]
fn fragmented_invalid_utf8_fails_closed() -> Result<(), Box<dyn Error>> {
    let frames = vec![
        server_frame(false, 0x1, b"{")?,
        server_frame(true, 0x0, &[0xff, b'}'])?,
    ];
    let (established, server) = establish_with_frames(frames)?;
    let mut no_pong_keys = || None;
    let exchange = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut no_pong_keys,
        Duration::from_millis(500),
    );

    join_server(server)?;
    assert!(matches!(
        exchange,
        Err(WebDriverBiDiLocateNodesExchangeError::ResponseDocument(
            WebDriverBiDiResponseDocumentAdmissionError::InvalidUtf8
        ))
    ));
    Ok(())
}
