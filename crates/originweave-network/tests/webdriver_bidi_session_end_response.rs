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
    WebDriverBiDiConnectionMessageRead, WebDriverBiDiReceivedTextMessage,
    WebDriverBiDiSessionEndCommand, WebDriverBiDiSessionEndResponseError,
    WebDriverBiDiSessionEndResult, WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageReader,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const END_SUCCESS_RESPONSE: &[u8] =
    br#"{"type":"success","id":7,"result":{"vendorExtension":{"clean":true}}}"#;
const END_REMOTE_ERROR_RESPONSE: &[u8] =
    br#"{"type":"error","id":7,"error":"unknown error","message":"remote refused"}"#;
const END_UNKNOWN_ID_RESPONSE: &[u8] =
    br#"{"type":"success","id":8,"result":{"vendorExtension":true}}"#;
const END_MALFORMED_RESPONSE: &[u8] = br#"{"type":"success","id":7}"#;

#[test]
fn unbound_end_command_cannot_consume_a_connection_bound_reply() -> Result<(), Box<dyn Error>> {
    use originweave_network::WebDriverBiDiCommandKind;

    let (message, mut original) = send_end_and_read_response(END_SUCCESS_RESPONSE)?;
    let mut unbound = WebDriverBiDiCommandCorrelation::new();
    unbound.register_command_for(7, WebDriverBiDiCommandKind::SessionEnd)?;
    assert!(matches!(
        WebDriverBiDiSessionEndResult::parse_and_correlate(&message, &mut unbound),
        Err(WebDriverBiDiSessionEndResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandConnectionProvenanceMissing {
                command_id: 7,
            },
        })
    ));
    assert_eq!(unbound.outstanding_count(), 1);
    let result = WebDriverBiDiSessionEndResult::parse_and_correlate(&message, &mut original)?;
    assert_eq!(result.command_id(), 7);
    assert_eq!(original.outstanding_count(), 0);
    Ok(())
}

#[test]
fn event_and_null_id_error_preserve_the_sent_end_command() -> Result<(), Box<dyn Error>> {
    for (document, expected) in [
        (
            br#"{"type":"event","method":"log.entryAdded","params":{}}"#.as_slice(),
            WebDriverBiDiCommandCorrelationError::EventIsNotResponse,
        ),
        (
            br#"{"type":"error","id":null,"error":"unknown error","message":"remote"}"#.as_slice(),
            WebDriverBiDiCommandCorrelationError::UncorrelatableErrorResponse,
        ),
    ] {
        let (message, mut correlation) = send_end_and_read_response(document)?;
        assert!(matches!(
            WebDriverBiDiSessionEndResult::parse_and_correlate(&message, &mut correlation),
            Err(WebDriverBiDiSessionEndResponseError::Correlation { source }) if source == expected
        ));
        assert_eq!(correlation.outstanding_count(), 1);
    }
    Ok(())
}

#[test]
fn replacement_end_replies_preserve_original_pending_request_and_recovery()
-> Result<(), Box<dyn Error>> {
    for response in [END_SUCCESS_RESPONSE, END_REMOTE_ERROR_RESPONSE] {
        let (original, mut pending) = send_end_and_read_response(END_SUCCESS_RESPONSE)?;
        let (replacement, _) = send_end_and_read_response(response)?;
        assert!(matches!(
            WebDriverBiDiSessionEndResult::parse_and_correlate(&replacement, &mut pending),
            Err(WebDriverBiDiSessionEndResponseError::Correlation {
                source: WebDriverBiDiCommandCorrelationError::ResponseConnectionMismatch {
                    command_id: 7
                }
            })
        ));
        assert_eq!(pending.outstanding_count(), 1);
        let result = WebDriverBiDiSessionEndResult::parse_and_correlate(&original, &mut pending)?;
        assert_eq!(result.command_id(), 7);
        assert_eq!(pending.outstanding_count(), 0);
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
            "session.end command unexpectedly required extended framing",
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

fn send_end_and_read_response(
    response: &'static [u8],
) -> Result<
    (
        WebDriverBiDiReceivedTextMessage,
        WebDriverBiDiCommandCorrelation,
    ),
    Box<dyn Error>,
> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut stream)?;
        if command != br#"{"id":7,"method":"session.end","params":{}}"# {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected session.end command",
            ));
        }
        stream.write_all(&[0x81, response.len() as u8])?;
        stream.write_all(response)
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
    let established = WebDriverBiDiSessionEndCommand::new(7)?.send(
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;

    let text = match WebDriverBiDiWebSocketMessageReader::new(established)
        .read_next(Duration::from_millis(500))?
    {
        WebDriverBiDiConnectionMessageRead::Text { message, .. } => message,
        other => {
            return Err(io::Error::other(format!(
                "session.end response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };

    server
        .join()
        .map_err(|_| io::Error::other("session.end response test server panicked"))??;
    Ok((text, correlation))
}

#[test]
fn session_end_success_accepts_extensible_empty_result_and_consumes_exact_correlation()
-> Result<(), Box<dyn Error>> {
    let (text, mut correlation) = send_end_and_read_response(END_SUCCESS_RESPONSE)?;
    assert_eq!(correlation.outstanding_count(), 1);

    let result = WebDriverBiDiSessionEndResult::parse_and_correlate(&text, &mut correlation)?;
    assert_eq!(result.command_id(), 7);
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn session_end_remote_error_consumes_only_the_correlated_command() -> Result<(), Box<dyn Error>> {
    let (text, mut correlation) = send_end_and_read_response(END_REMOTE_ERROR_RESPONSE)?;
    let parsed = WebDriverBiDiSessionEndResult::parse_and_correlate(&text, &mut correlation);
    let error = match parsed {
        Ok(_) => {
            return Err(
                io::Error::other("remote error was accepted as session.end success").into(),
            );
        }
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WebDriverBiDiSessionEndResponseError::RemoteProtocolError { command_id: 7 }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.end returned a protocol error"
    );
    assert!(error.source().is_none());
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn malformed_session_end_envelope_fails_before_consuming_correlation() -> Result<(), Box<dyn Error>>
{
    let (text, mut correlation) = send_end_and_read_response(END_MALFORMED_RESPONSE)?;
    let parsed = WebDriverBiDiSessionEndResult::parse_and_correlate(&text, &mut correlation);
    let error = match parsed {
        Ok(_) => {
            return Err(io::Error::other("malformed session.end response was accepted").into());
        }
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WebDriverBiDiSessionEndResponseError::Envelope { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.end envelope is invalid"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn unknown_session_end_response_id_does_not_consume_the_outstanding_command()
-> Result<(), Box<dyn Error>> {
    let (text, mut correlation) = send_end_and_read_response(END_UNKNOWN_ID_RESPONSE)?;
    let parsed = WebDriverBiDiSessionEndResult::parse_and_correlate(&text, &mut correlation);
    let error = match parsed {
        Ok(_) => {
            return Err(io::Error::other("unknown session.end response id was accepted").into());
        }
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WebDriverBiDiSessionEndResponseError::Correlation { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.end response correlation failed"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}
