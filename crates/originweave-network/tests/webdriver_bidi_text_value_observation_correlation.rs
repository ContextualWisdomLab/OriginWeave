use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandKind, WebDriverBiDiTcpConnectionPlan,
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

fn write_text_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    stream.write_all(&[0x81])?;
    match payload.len() {
        0..=125 => stream.write_all(&[payload.len() as u8])?,
        126..=65_535 => {
            stream.write_all(&[126])?;
            stream.write_all(&(payload.len() as u16).to_be_bytes())?;
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "fixture response unexpectedly required 64-bit framing",
            ));
        }
    }
    stream.write_all(payload)
}

fn receive_server_text(
    payload: &[u8],
) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let response = payload.to_vec();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_text_frame(&mut stream, &response)
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

fn require_observation_error(
    message: &WebDriverBiDiWebSocketTextMessage,
    correlation: &mut WebDriverBiDiCommandCorrelation,
) -> Result<WebDriverBiDiTextValueObservationResponseError, Box<dyn Error>> {
    match WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        message,
        "expected",
        correlation,
    ) {
        Err(error) => Ok(error),
        Ok(_) => Err(io::Error::other("fixture unexpectedly admitted an invalid response").into()),
    }
}

fn require_expected_text_error(
    message: &WebDriverBiDiWebSocketTextMessage,
    expected_text: &str,
    correlation: &mut WebDriverBiDiCommandCorrelation,
) -> Result<WebDriverBiDiTextValueObservationResponseError, Box<dyn Error>> {
    match WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        message,
        expected_text,
        correlation,
    ) {
        Err(error) => Ok(error),
        Ok(_) => {
            Err(io::Error::other("fixture unexpectedly admitted invalid expected text").into())
        }
    }
}

#[test]
fn protocol_error_correlation_and_diagnostics_fail_closed_without_consuming_other_state()
-> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(70, WebDriverBiDiCommandKind::TextValueObservation)?;

    let unknown_protocol_error = receive_server_text(
        br#"{"type":"error","id":71,"error":"unknown error","message":"remote failure"}"#,
    )?;
    let correlation_error = require_observation_error(&unknown_protocol_error, &mut correlation)?;
    assert!(matches!(
        &correlation_error,
        WebDriverBiDiTextValueObservationResponseError::Correlation { .. }
    ));
    assert_eq!(
        correlation_error.to_string(),
        "WebDriver BiDi text-value observation response correlation failed"
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let invalid_envelope = receive_server_text(b"not-json")?;
    let envelope_error = require_observation_error(&invalid_envelope, &mut correlation)?;
    assert!(matches!(
        &envelope_error,
        WebDriverBiDiTextValueObservationResponseError::Envelope { .. }
    ));
    assert_eq!(
        envelope_error.to_string(),
        "WebDriver BiDi text-value observation envelope is invalid"
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let malformed_projection = receive_server_text(
        br#"{"type":"success","id":70,"result":{"type":"success","result":{"type":"string","value":"expected"}}}"#,
    )?;
    let projection_error = require_observation_error(&malformed_projection, &mut correlation)?;
    assert!(matches!(
        &projection_error,
        WebDriverBiDiTextValueObservationResponseError::Projection { .. }
    ));
    assert_eq!(
        projection_error.to_string(),
        "WebDriver BiDi text-value observation result is invalid"
    );
    assert_eq!(correlation.outstanding_count(), 1);

    Ok(())
}

#[test]
fn invalid_expected_text_fails_before_response_or_correlation_state_is_touched()
-> Result<(), Box<dyn Error>> {
    let invalid_envelope = receive_server_text(b"not-json")?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(70, WebDriverBiDiCommandKind::TextValueObservation)?;

    let empty_error = require_expected_text_error(&invalid_envelope, "", &mut correlation)?;
    assert!(matches!(
        empty_error,
        WebDriverBiDiTextValueObservationResponseError::EmptyExpectedText
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let oversized = "x".repeat(MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES + 1);
    let oversized_error =
        require_expected_text_error(&invalid_envelope, &oversized, &mut correlation)?;
    assert!(matches!(
        oversized_error,
        WebDriverBiDiTextValueObservationResponseError::ExpectedTextTooLong
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let control_error =
        require_expected_text_error(&invalid_envelope, "bad\u{0001}value", &mut correlation)?;
    assert!(matches!(
        control_error,
        WebDriverBiDiTextValueObservationResponseError::InvalidExpectedText
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    Ok(())
}
