use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError, WebDriverBiDiJsonEnvelopeKind,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly,
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

fn parse_over_real_transport(
    payload: &[u8],
) -> Result<Result<WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError>, Box<dyn Error>> {
    if payload.len() > 125 {
        return Err(io::Error::other("test payload exceeded one-byte frame length").into());
    }

    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let payload = payload.to_vec();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let payload_len = u8::try_from(payload.len())
            .map_err(|_| io::Error::other("test payload length does not fit u8"))?;
        stream.write_all(&[0x81, payload_len])?;
        stream.write_all(&payload)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let admitted = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let established = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?
        .write_opening_request(Duration::from_millis(500))?
        .read_opening_response(Duration::from_millis(500))?;
    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;

    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "validated text frame produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    let parsed = WebDriverBiDiJsonEnvelope::parse(&text);

    server
        .join()
        .map_err(|_| io::Error::other("JSON-envelope server panicked"))??;
    Ok(parsed)
}

#[test]
fn real_transport_accepts_spec_defined_client_window_error() -> Result<(), Box<dyn Error>> {
    let envelope = parse_over_real_transport(
        br#"{"type":"error","id":7,"error":"no such client window","message":"unknown client window"}"#,
    )?;
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::kind),
        Ok(WebDriverBiDiJsonEnvelopeKind::Error)
    );
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::command_id),
        Ok(Some(7))
    );
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::error_code),
        Ok(Some("no such client window"))
    );
    Ok(())
}
