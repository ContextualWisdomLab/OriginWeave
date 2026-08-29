use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketControlKind, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketMessageError,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";

fn connect(
    endpoint: &str,
) -> Result<originweave_network::WebDriverBiDiTcpConnection, Box<dyn Error>> {
    let admitted = WebDriverBiDiWebSocketEndpoint::new(endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?;
    Ok(plan.connect()?)
}

fn establish(
    endpoint: &str,
) -> Result<originweave_network::WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    Ok(written.read_opening_response(Duration::from_millis(500))?)
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

fn write_server_frame(
    stream: &mut TcpStream,
    fin: bool,
    opcode: u8,
    payload: &[u8],
) -> io::Result<()> {
    let first = if fin { 0x80 | opcode } else { opcode };
    stream.write_all(&[first])?;
    match payload.len() {
        0..=125 => stream.write_all(&[payload.len() as u8])?,
        126..=65_535 => {
            stream.write_all(&[126])?;
            stream.write_all(&(payload.len() as u16).to_be_bytes())?;
        }
        _ => {
            stream.write_all(&[127])?;
            stream.write_all(&(payload.len() as u64).to_be_bytes())?;
        }
    }
    stream.write_all(payload)
}

#[test]
fn message_assembler_reassembles_split_utf8_around_interleaved_ping() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_server_frame(&mut stream, false, 0x1, b"A\xe2")?;
        write_server_frame(&mut stream, true, 0x9, b"probe")?;
        write_server_frame(&mut stream, true, 0x0, b"\x82\xacB")
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let established = establish(&endpoint)?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();

    let (established, first) = established.read_frame(Duration::from_millis(500))?;
    assert!(matches!(
        assembler.push_frame(first)?,
        WebDriverBiDiWebSocketMessageAssembly::Pending
    ));

    let (established, ping) = established.read_frame(Duration::from_millis(500))?;
    let control = match assembler.push_frame(ping)? {
        WebDriverBiDiWebSocketMessageAssembly::Control(control) => control,
        other => {
            return Err(io::Error::other(format!(
                "interleaved Ping produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    assert_eq!(control.kind(), WebDriverBiDiWebSocketControlKind::Ping);
    assert_eq!(control.payload(), b"probe");

    let (_established, continuation) = established.read_frame(Duration::from_millis(500))?;
    let text = match assembler.push_frame(continuation)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "final continuation produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    assert_eq!(text.as_str(), "A€B");

    server
        .join()
        .map_err(|_| io::Error::other("fragmented-message server panicked"))??;
    Ok(())
}

#[test]
fn message_assembler_fails_closed_after_binary_message() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_server_frame(&mut stream, true, 0x2, b"binary")?;
        write_server_frame(&mut stream, true, 0x1, b"later-text")
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let established = establish(&endpoint)?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();

    let (established, binary) = established.read_frame(Duration::from_millis(500))?;
    assert!(matches!(
        assembler.push_frame(binary),
        Err(WebDriverBiDiWebSocketMessageError::UnexpectedBinaryMessage)
    ));

    let (_established, later_text) = established.read_frame(Duration::from_millis(500))?;
    assert!(matches!(
        assembler.push_frame(later_text),
        Err(WebDriverBiDiWebSocketMessageError::AssemblerPoisoned)
    ));

    server
        .join()
        .map_err(|_| io::Error::other("binary-message server panicked"))??;
    Ok(())
}

#[test]
fn message_assembler_rejects_continuation_without_text_start() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_server_frame(&mut stream, true, 0x0, b"orphan")
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let established = establish(&endpoint)?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let (_established, continuation) = established.read_frame(Duration::from_millis(500))?;
    assert!(matches!(
        assembler.push_frame(continuation),
        Err(WebDriverBiDiWebSocketMessageError::UnexpectedContinuation)
    ));

    server
        .join()
        .map_err(|_| io::Error::other("orphan-continuation server panicked"))??;
    Ok(())
}
