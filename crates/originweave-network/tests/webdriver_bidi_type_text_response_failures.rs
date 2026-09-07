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
    WebDriverBiDiCommandKind, WebDriverBiDiConnectionMessageRead, WebDriverBiDiReceivedTextMessage,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiTypeTextResponseError,
    WebDriverBiDiTypeTextResult, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMessageReader,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const REMOTE_ERROR_RESPONSE: &[u8] =
    br#"{"type":"error","id":42,"error":"invalid argument","message":"rejected"}"#;
const UNKNOWN_ID_RESPONSE: &[u8] =
    br#"{"type":"success","id":43,"result":{"vendorExtension":true}}"#;
const MALFORMED_RESPONSE: &[u8] = br#"{"type":"success","id":42}"#;

#[test]
fn text_response_rejects_other_command_families_without_consuming_them()
-> Result<(), Box<dyn Error>> {
    for payload in [
        br#"{"type":"success","id":42,"result":{}}"#.as_slice(),
        REMOTE_ERROR_RESPONSE,
    ] {
        for kind in [
            WebDriverBiDiCommandKind::SessionStatus,
            WebDriverBiDiCommandKind::SessionEnd,
            WebDriverBiDiCommandKind::PointerClick,
            WebDriverBiDiCommandKind::NavigationCommittedSubscription,
            WebDriverBiDiCommandKind::NavigationCommittedUnsubscribe,
        ] {
            let text = receive_response(payload)?;
            let mut correlation = WebDriverBiDiCommandCorrelation::new();
            correlation.register_command_for(42, kind)?;
            assert!(matches!(
                WebDriverBiDiTypeTextResult::parse_and_correlate(&text, &mut correlation),
                Err(WebDriverBiDiTypeTextResponseError::Correlation {
                    source: WebDriverBiDiCommandCorrelationError::CommandKindMismatch {
                        expected: WebDriverBiDiCommandKind::TypeText,
                        actual,
                    },
                }) if actual == kind
            ));
            assert_eq!(correlation.outstanding_count(), 1);
        }
    }
    Ok(())
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

fn receive_response(
    payload: &'static [u8],
) -> Result<WebDriverBiDiReceivedTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_text_frame(&mut stream, payload)
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
    let text = match WebDriverBiDiWebSocketMessageReader::new(established)
        .read_next(Duration::from_millis(500))?
    {
        WebDriverBiDiConnectionMessageRead::Text { message, .. } => message,
        other => {
            return Err(io::Error::other(format!(
                "text-input response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };

    server
        .join()
        .map_err(|_| io::Error::other("text-input response fixture server panicked"))??;
    Ok(text)
}

#[test]
fn remote_protocol_error_cannot_consume_a_command_without_sender_provenance()
-> Result<(), Box<dyn Error>> {
    let text = receive_response(REMOTE_ERROR_RESPONSE)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(42, WebDriverBiDiCommandKind::TypeText)?;

    let error = WebDriverBiDiTypeTextResult::parse_and_correlate(&text, &mut correlation)
        .err()
        .ok_or_else(|| {
            io::Error::other("remote protocol error was accepted as text-input success")
        })?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandConnectionProvenanceMissing {
                command_id: 42,
            },
        }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-input response correlation failed"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn malformed_text_input_envelope_fails_before_consuming_correlation() -> Result<(), Box<dyn Error>>
{
    let text = receive_response(MALFORMED_RESPONSE)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(42, WebDriverBiDiCommandKind::TypeText)?;

    let error = WebDriverBiDiTypeTextResult::parse_and_correlate(&text, &mut correlation)
        .err()
        .ok_or_else(|| io::Error::other("malformed text-input response was accepted"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextResponseError::Envelope { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn unknown_text_input_response_id_does_not_consume_outstanding_command()
-> Result<(), Box<dyn Error>> {
    let text = receive_response(UNKNOWN_ID_RESPONSE)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(42, WebDriverBiDiCommandKind::TypeText)?;

    let error = WebDriverBiDiTypeTextResult::parse_and_correlate(&text, &mut correlation)
        .err()
        .ok_or_else(|| io::Error::other("unknown text-input response id was accepted"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextResponseError::Correlation { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}
