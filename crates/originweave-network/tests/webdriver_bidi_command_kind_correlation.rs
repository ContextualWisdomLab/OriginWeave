use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCommandKind, WebDriverBiDiCorrelatedResponseOutcome, WebDriverBiDiJsonEnvelope,
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

fn parse_success_over_loopback() -> Result<WebDriverBiDiJsonEnvelope, Box<dyn Error>> {
    const DOCUMENT: &[u8] = br#"{"type":"success","id":42,"result":{}}"#;
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        stream.write_all(&[0x81, DOCUMENT.len() as u8])?;
        stream.write_all(DOCUMENT)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(SESSION_ID)?
        .into_explicit_connect_target()?;
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
    let envelope = WebDriverBiDiJsonEnvelope::parse(&text)?;
    server
        .join()
        .map_err(|_| io::Error::other("command-kind correlation test server panicked"))??;
    Ok(envelope)
}

#[test]
fn typed_response_provenance_cannot_cross_the_legacy_correlation_boundary()
-> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(42, WebDriverBiDiCommandKind::TextValueObservation)?;
    let response = parse_success_over_loopback()?;

    assert_eq!(
        correlation.correlate_response(&response),
        Err(WebDriverBiDiCommandCorrelationError::CommandKindMismatch)
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let completed = correlation.correlate_response_for(
        &response,
        WebDriverBiDiCommandKind::TextValueObservation,
    )?;
    assert_eq!(completed.command_id(), 42);
    assert_eq!(
        completed.outcome(),
        WebDriverBiDiCorrelatedResponseOutcome::Success
    );
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}
