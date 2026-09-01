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
    WebDriverBiDiTextValuePostconditionError, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketTextMessage,
    verify_webdriver_bidi_text_value_postcondition,
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
    let text = match assembler.push_frame(frame)? {
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
        .map_err(|_| io::Error::other("postcondition fixture server panicked"))??;
    Ok(text)
}

#[test]
fn exact_match_is_the_only_successful_text_postcondition() -> Result<(), Box<dyn Error>> {
    let response = receive_server_text(
        br#"{"type":"success","id":70,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"expected"}}}"#,
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(70)?;

    let verified = verify_webdriver_bidi_text_value_postcondition(
        &response,
        "expected",
        &mut correlation,
    )?;

    assert_eq!(verified.command_id(), 70);
    assert_eq!(verified.observed_text_bytes(), "expected".len());
    assert_eq!(correlation.outstanding_count(), 0);
    assert!(!format!("{verified:?}").contains("expected"));
    Ok(())
}

#[test]
fn mismatch_is_typed_failure_after_consuming_its_exact_response() -> Result<(), Box<dyn Error>> {
    let response = receive_server_text(
        br#"{"type":"success","id":71,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"unexpected"}}}"#,
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(71)?;

    let error = verify_webdriver_bidi_text_value_postcondition(
        &response,
        "expected",
        &mut correlation,
    )
    .expect_err("a mismatched page value must not be returned as successful postcondition evidence");

    assert!(matches!(
        error,
        WebDriverBiDiTextValuePostconditionError::PostconditionMismatch {
            command_id: 71,
            observed_text_bytes: 10,
        }
    ));
    assert_eq!(correlation.outstanding_count(), 0);
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-value postcondition did not match the authorized expected text"
    );
    let debug = format!("{error:?}");
    assert!(!debug.contains("expected"));
    assert!(!debug.contains("unexpected"));
    Ok(())
}

#[test]
fn malformed_observation_stays_a_typed_source_error_without_consuming_state()
-> Result<(), Box<dyn Error>> {
    let response = receive_server_text(b"not-json")?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(72)?;

    let error = verify_webdriver_bidi_text_value_postcondition(
        &response,
        "expected",
        &mut correlation,
    )
    .expect_err("malformed observation must fail closed");

    assert!(matches!(
        &error,
        WebDriverBiDiTextValuePostconditionError::Observation { .. }
    ));
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}
