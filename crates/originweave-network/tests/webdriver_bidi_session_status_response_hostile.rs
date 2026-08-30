use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiSessionStatusCommand,
    WebDriverBiDiSessionStatusResponseError, WebDriverBiDiSessionStatusResult,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTextMessage, MAX_WEBDRIVER_BIDI_SESSION_STATUS_MESSAGE_SIZE,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";

type StatusRead = (
    WebDriverBiDiWebSocketTextMessage,
    WebDriverBiDiCommandCorrelation,
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

fn read_masked_text_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x81 || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected one final masked client text frame",
        ));
    }
    let length = usize::from(header[1] & 0x7f);
    if length > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "test command unexpectedly required extended framing",
        ));
    }
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
}

fn write_unmasked_text_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    match payload.len() {
        0..=125 => {
            let length = u8::try_from(payload.len()).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "short frame length overflow")
            })?;
            stream.write_all(&[0x81, length])?;
        }
        126..=65_535 => {
            let length = u16::try_from(payload.len()).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "extended frame length overflow")
            })?;
            stream.write_all(&[0x81, 126])?;
            stream.write_all(&length.to_be_bytes())?;
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "hostile response exceeds the reviewed test-frame budget",
            ));
        }
    }
    stream.write_all(payload)
}

fn send_status_and_read_response(response: Vec<u8>) -> Result<StatusRead, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut stream)?;
        if command != br#"{"id":7,"method":"session.status","params":{}}"# {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected session.status command",
            ));
        }
        write_unmasked_text_frame(&mut stream, &response)
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

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let established = WebDriverBiDiSessionStatusCommand::new(7)?.send(
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;

    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "session.status response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };

    server
        .join()
        .map_err(|_| io::Error::other("session.status response test server panicked"))??;
    Ok((text, correlation))
}

fn parse_response(
    response: Vec<u8>,
) -> Result<
    (
        Result<WebDriverBiDiSessionStatusResult, WebDriverBiDiSessionStatusResponseError>,
        WebDriverBiDiCommandCorrelation,
    ),
    Box<dyn Error>,
> {
    let (text, mut correlation) = send_status_and_read_response(response)?;
    let parsed = WebDriverBiDiSessionStatusResult::parse_and_correlate(&text, &mut correlation);
    Ok((parsed, correlation))
}

#[test]
fn status_projection_accepts_extensible_json_and_unicode_through_real_transport()
-> Result<(), Box<dyn Error>> {
    let response = br#"{"type":"success","id":7,"meta":[null,true,false,1,-2.5e+3,{"nested":"value"}],"re\u0073ult":{"message":"re\u0061dy \ud83d\ude80","extra":{},"ready":false}}"#
        .to_vec();
    let (parsed, correlation) = parse_response(response)?;
    let result = parsed?;

    assert_eq!(result.command_id(), 7);
    assert!(!result.ready());
    assert_eq!(result.message(), "ready 🚀");
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn malformed_success_bodies_fail_closed_without_consuming_correlation() -> Result<(), Box<dyn Error>>
{
    let oversized = format!(
        "{{\"type\":\"success\",\"id\":7,\"result\":{{\"ready\":true,\"message\":\"{}\"}}}}",
        "x".repeat(MAX_WEBDRIVER_BIDI_SESSION_STATUS_MESSAGE_SIZE + 1)
    )
    .into_bytes();
    let cases = [
        br#"{"type":"success","id":7,"result":{"ready":0,"message":"x"}}"#.to_vec(),
        br#"{"type":"success","id":7,"result":{"ready":true}}"#.to_vec(),
        br#"{"type":"success","id":7,"result":{"ready":true,"message":false}}"#.to_vec(),
        br#"{"type":"success","id":7,"result":{"ready":true,"ready":false,"message":"x"}}"#
            .to_vec(),
        br#"{"type":"success","id":7,"result":{"ready":true,"message":"x","message":"y"}}"#
            .to_vec(),
        oversized,
    ];

    for response in cases {
        let (parsed, correlation) = parse_response(response)?;
        assert!(parsed.is_err());
        assert_eq!(correlation.outstanding_count(), 1);
    }
    Ok(())
}

#[test]
fn envelope_correlation_and_remote_error_failures_preserve_exact_command_semantics()
-> Result<(), Box<dyn Error>> {
    let (invalid_envelope, correlation) = parse_response(
        br#"{"type":"success","id":7,"result":{"ready":true,"message":"x"}"#.to_vec(),
    )?;
    assert!(matches!(
        invalid_envelope,
        Err(WebDriverBiDiSessionStatusResponseError::Envelope { .. })
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let (unknown_id, correlation) = parse_response(
        br#"{"type":"success","id":8,"result":{"ready":true,"message":"x"}}"#.to_vec(),
    )?;
    assert!(matches!(
        unknown_id,
        Err(WebDriverBiDiSessionStatusResponseError::Correlation { .. })
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let (event, correlation) = parse_response(
        br#"{"type":"event","method":"log.entryAdded","params":{}}"#.to_vec(),
    )?;
    assert!(matches!(
        event,
        Err(WebDriverBiDiSessionStatusResponseError::Correlation { .. })
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let (remote_error, correlation) = parse_response(
        br#"{"type":"error","id":7,"error":"unknown error","message":"remote refused status","stacktrace":""}"#
            .to_vec(),
    )?;
    assert!(matches!(
        remote_error,
        Err(WebDriverBiDiSessionStatusResponseError::RemoteProtocolError { command_id: 7 })
    ));
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}
