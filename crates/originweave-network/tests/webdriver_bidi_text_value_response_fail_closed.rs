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
const EVENT: &[u8] = br#"{"type":"event","method":"browsingContext.load","params":{}}"#;
const PROTOCOL_ERROR: &[u8] =
    br#"{"type":"error","id":73,"error":"unknown error","message":"page-controlled detail"}"#;
const SCRIPT_EXCEPTION: &[u8] = br#"{"type":"success","id":74,"result":{"type":"exception","realm":"realm-1","exceptionDetails":{"text":"page-controlled detail"}}}"#;

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
                "test JSON document exceeds two-byte frame length",
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
        .map_err(|_| io::Error::other("text-value integration server panicked"))??;
    Ok(text)
}

#[test]
fn production_instantiation_fails_closed_for_event_protocol_error_and_script_exception()
-> Result<(), Box<dyn Error>> {
    let event = read_text_over_loopback(EVENT)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    assert!(matches!(
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &event,
            "expected",
            &mut correlation,
        ),
        Err(WebDriverBiDiTextValueObservationResponseError::UnexpectedEvent)
    ));
    assert_eq!(correlation.outstanding_count(), 0);

    correlation.register_command(73)?;
    let protocol_error = read_text_over_loopback(PROTOCOL_ERROR)?;
    assert!(matches!(
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &protocol_error,
            "expected",
            &mut correlation,
        ),
        Err(WebDriverBiDiTextValueObservationResponseError::RemoteProtocolError { command_id: 73 })
    ));
    assert_eq!(correlation.outstanding_count(), 0);

    correlation.register_command(74)?;
    let script_exception = read_text_over_loopback(SCRIPT_EXCEPTION)?;
    assert!(matches!(
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &script_exception,
            "expected",
            &mut correlation,
        ),
        Err(WebDriverBiDiTextValueObservationResponseError::ScriptException { command_id: 74 })
    ));
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}
