use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandKind, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiTextValueObservationResponseError, WebDriverBiDiTextValueObservationResult,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTextMessage,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const ERROR_UNKNOWN_COMMAND: &[u8] =
    br#"{"type":"error","id":71,"error":"unknown error","message":"remote failure"}"#;
const MALFORMED_PROJECTION: &[u8] = br#"{"type":"success","id":70,"result":{"type":"success","result":{"type":"string","value":"expected"}}}"#;
const SUCCESS_UNKNOWN_COMMAND: &[u8] = br#"{"type":"success","id":72,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"expected"}}}"#;
const FINAL_EXPECTED_TEXT: &str = "Quarterly review";
const VALID_SUCCESS: &[u8] = br#"{"type":"success","id":70,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"Quarterly review"}}}"#;

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

fn write_unmasked_text_frame(stream: &mut TcpStream, document: &[u8]) -> io::Result<()> {
    if document.len() <= 125 {
        let length = u8::try_from(document.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "short frame length exceeds u8")
        })?;
        stream.write_all(&[0x81, length])?;
    } else {
        let length = u16::try_from(document.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "unit JSON document exceeds two-byte frame length",
            )
        })?;
        stream.write_all(&[0x81, 126])?;
        stream.write_all(&length.to_be_bytes())?;
    }
    stream.write_all(document)
}

fn read_text_over_loopback(
    document: &'static [u8],
) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_unmasked_text_frame(&mut stream, document)
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
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "validated text frame produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };

    server
        .join()
        .map_err(|_| io::Error::other("text-value unit server panicked"))??;
    Ok(text)
}

#[test]
fn public_text_value_boundary_covers_error_adapters_and_credential_safe_result()
-> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(70, WebDriverBiDiCommandKind::TextValueObservation)?;

    let invalid = read_text_over_loopback(b"not-json")?;
    let envelope_result = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        &invalid,
        "expected",
        &mut correlation,
    );
    assert!(matches!(
        &envelope_result,
        Err(WebDriverBiDiTextValueObservationResponseError::Envelope { .. })
    ));
    assert_eq!(
        envelope_result
            .as_ref()
            .err()
            .map(ToString::to_string)
            .as_deref(),
        Some("WebDriver BiDi text-value observation envelope is invalid")
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let error_unknown = read_text_over_loopback(ERROR_UNKNOWN_COMMAND)?;
    assert!(matches!(
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &error_unknown,
            "expected",
            &mut correlation,
        ),
        Err(WebDriverBiDiTextValueObservationResponseError::Correlation { .. })
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let malformed_projection = read_text_over_loopback(MALFORMED_PROJECTION)?;
    let projection_result = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        &malformed_projection,
        "expected",
        &mut correlation,
    );
    assert!(matches!(
        &projection_result,
        Err(WebDriverBiDiTextValueObservationResponseError::Projection { .. })
    ));
    assert_eq!(
        projection_result
            .as_ref()
            .err()
            .map(ToString::to_string)
            .as_deref(),
        Some("WebDriver BiDi text-value observation result is invalid")
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let success_unknown = read_text_over_loopback(SUCCESS_UNKNOWN_COMMAND)?;
    let correlation_result = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        &success_unknown,
        "expected",
        &mut correlation,
    );
    assert!(matches!(
        &correlation_result,
        Err(WebDriverBiDiTextValueObservationResponseError::Correlation { .. })
    ));
    assert_eq!(
        correlation_result
            .as_ref()
            .err()
            .map(ToString::to_string)
            .as_deref(),
        Some("WebDriver BiDi text-value observation response correlation failed")
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let valid_success = read_text_over_loopback(VALID_SUCCESS)?;
    let result = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        &valid_success,
        FINAL_EXPECTED_TEXT,
        &mut correlation,
    )?;
    assert_eq!(result.command_id(), 70);
    assert_eq!(result.observed_text_bytes(), FINAL_EXPECTED_TEXT.len());
    assert!(result.matches_expected_text());
    assert_eq!(correlation.outstanding_count(), 0);

    let debug = format!("{result:?}");
    assert!(debug.contains("WebDriverBiDiTextValueObservationResult"));
    assert!(!debug.contains(FINAL_EXPECTED_TEXT));
    Ok(())
}
