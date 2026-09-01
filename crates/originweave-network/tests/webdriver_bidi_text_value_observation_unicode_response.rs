use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiTextValueObservationResponseError, WebDriverBiDiTextValueObservationResult,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTextMessage,
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

fn receive_server_text(
    payload: &[u8],
) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    if payload.len() > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "fixture payload must fit one short server text frame",
        )
        .into());
    }

    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let response = payload.to_vec();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        stream.write_all(&[0x81, response.len() as u8])?;
        stream.write_all(&response)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(SESSION_ID)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let established = WebDriverBiDiWebSocketHandshakePlan::new(
        connection,
        WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?,
    )?
    .write_opening_request(Duration::from_millis(500))?
    .read_opening_response(Duration::from_millis(500))?;
    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let message = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "fixture produced unexpected message assembly state: {other:?}"
            ))
            .into());
        }
    };
    server
        .join()
        .map_err(|_| io::Error::other("response fixture server panicked"))??;
    Ok(message)
}

#[test]
fn escaped_unicode_is_compared_after_bounded_response_projection() -> Result<(), Box<dyn Error>> {
    let response = receive_server_text(
        br#"{"type":"success","id":81,"result":{"type":"success","realm":"r","result":{"type":"string","value":"\u20ac"}}}"#,
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(81)?;

    let result = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        &response,
        "€",
        &mut correlation,
    )?;

    assert_eq!(result.command_id(), 81);
    assert_eq!(result.observed_text_bytes(), "€".len());
    assert!(result.matches_expected_text());
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn projection_error_diagnostic_remains_structural_and_non_sensitive() -> Result<(), Box<dyn Error>> {
    let response = receive_server_text(
        br#"{"type":"success","id":82,"result":{"type":"success","result":{"type":"string","value":"x"}}}"#,
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(82)?;

    let Err(WebDriverBiDiTextValueObservationResponseError::Projection { source }) =
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &response,
            "x",
            &mut correlation,
        )
    else {
        return Err(io::Error::other("malformed projection unexpectedly admitted").into());
    };

    assert_eq!(source.to_string(), "missing member result.realm");
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}
