use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiConnectionMessageRead, WebDriverBiDiConnectionMessageReadError,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMessageReader,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";

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

fn established_for_frames(
    frames: Vec<Vec<u8>>,
) -> Result<originweave_network::WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        for frame in frames {
            stream.write_all(&frame)?;
        }
        Ok(())
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(SESSION_ID)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    Ok(WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?
        .write_opening_request(Duration::from_millis(500))?
        .read_opening_response(Duration::from_millis(500))?)
}

#[test]
fn fragmented_text_and_interleaved_control_remain_on_one_reader() -> Result<(), Box<dyn Error>> {
    let established = established_for_frames(vec![
        vec![0x01, 0x03, b'a', b'b', b'c'],
        vec![0x89, 0x00],
        vec![0x80, 0x03, b'd', b'e', b'f'],
    ])?;
    let reader = WebDriverBiDiWebSocketMessageReader::new(established);
    assert!(format!("{reader:?}").contains("connection_bound"));

    let first = reader.read_next(Duration::from_millis(500))?;
    assert!(format!("{first:?}").starts_with("Pending"));
    let reader = match first {
        WebDriverBiDiConnectionMessageRead::Pending(reader) => reader,
        _ => return Err(io::Error::other("first fragment did not remain pending").into()),
    };

    let control = reader.read_next(Duration::from_millis(500))?;
    assert!(format!("{control:?}").starts_with("Control"));
    let reader = match control {
        WebDriverBiDiConnectionMessageRead::Control { reader, message } => {
            assert_eq!(message.payload(), b"");
            reader
        }
        _ => return Err(io::Error::other("interleaved Ping was not surfaced").into()),
    };

    let completed = reader.read_next(Duration::from_millis(500))?;
    let debug = format!("{completed:?}");
    assert!(debug.starts_with("Text"));
    assert!(debug.contains("payload_bytes"));
    match completed {
        WebDriverBiDiConnectionMessageRead::Text {
            established,
            message: _,
        } => drop(established),
        _ => return Err(io::Error::other("continuation did not complete text message").into()),
    }
    Ok(())
}

#[test]
fn frame_and_message_failures_remain_typed_and_sourced() -> Result<(), Box<dyn Error>> {
    let malformed = established_for_frames(vec![vec![0x81, 0x80]])?;
    let frame_error = WebDriverBiDiWebSocketMessageReader::new(malformed)
        .read_next(Duration::from_millis(500))
        .err()
        .ok_or_else(|| io::Error::other("masked server frame was accepted"))?;
    assert!(matches!(
        frame_error,
        WebDriverBiDiConnectionMessageReadError::Frame { .. }
    ));
    assert_eq!(
        frame_error.to_string(),
        "connection-bound WebDriver BiDi WebSocket frame read failed"
    );
    assert!(frame_error.source().is_some());

    let binary = established_for_frames(vec![vec![0x82, 0x00]])?;
    let message_error = WebDriverBiDiWebSocketMessageReader::new(binary)
        .read_next(Duration::from_millis(500))
        .err()
        .ok_or_else(|| io::Error::other("binary BiDi message was accepted"))?;
    assert!(matches!(
        message_error,
        WebDriverBiDiConnectionMessageReadError::Message { .. }
    ));
    assert_eq!(
        message_error.to_string(),
        "connection-bound WebDriver BiDi WebSocket message assembly failed"
    );
    assert!(message_error.source().is_some());
    Ok(())
}
