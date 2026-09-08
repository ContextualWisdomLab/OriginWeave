use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiConnectionMessageRead,
    WebDriverBiDiReceivedTextMessage, WebDriverBiDiSessionStatusCommand,
    WebDriverBiDiSessionStatusResponseError, WebDriverBiDiSessionStatusResult,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageReader,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const STATUS_RESPONSE: &[u8] =
    br#"{"type":"success","id":7,"result":{"ready":true,"message":"capacity available"}}"#;
const STATUS_RESPONSE_MISSING_READY: &[u8] =
    br#"{"type":"success","id":7,"result":{"message":"capacity available"}}"#;
const STATUS_RESPONSE_EMPTY_RESULT: &[u8] = br#"{"type":"success","id":7,"result":{}}"#;

#[test]
fn teardown_stack_rejects_replacement_status_reply_and_preserves_original_request()
-> Result<(), Box<dyn Error>> {
    let (original, mut pending) = send_status_and_read_response(STATUS_RESPONSE)?;
    let (replacement, _) = send_status_and_read_response(STATUS_RESPONSE)?;
    assert!(
        WebDriverBiDiSessionStatusResult::parse_and_correlate(&replacement, &mut pending).is_err(),
        "a replacement status reply must not complete the original request"
    );
    assert_eq!(pending.outstanding_count(), 1);
    let result = WebDriverBiDiSessionStatusResult::parse_and_correlate(&original, &mut pending)?;
    assert_eq!(result.command_id(), 7);
    assert_eq!(pending.outstanding_count(), 0);
    Ok(())
}

#[test]
fn unbound_command_cannot_consume_a_connection_bound_reply() -> Result<(), Box<dyn Error>> {
    use originweave_network::{WebDriverBiDiCommandCorrelationError, WebDriverBiDiCommandKind};

    let (message, mut original) = send_status_and_read_response(STATUS_RESPONSE)?;
    let mut unbound = WebDriverBiDiCommandCorrelation::new();
    unbound.register_command_for(7, WebDriverBiDiCommandKind::SessionStatus)?;
    assert!(matches!(
        WebDriverBiDiSessionStatusResult::parse_and_correlate(&message, &mut unbound),
        Err(WebDriverBiDiSessionStatusResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandConnectionProvenanceMissing {
                command_id: 7,
            },
        })
    ));
    assert_eq!(unbound.outstanding_count(), 1);
    assert_eq!(original.outstanding_count(), 1);
    let result = WebDriverBiDiSessionStatusResult::parse_and_correlate(&message, &mut original)?;
    assert_eq!(result.command_id(), 7);
    assert_eq!(original.outstanding_count(), 0);
    Ok(())
}

#[test]
fn event_and_null_id_error_preserve_the_sent_status_command() -> Result<(), Box<dyn Error>> {
    use originweave_network::WebDriverBiDiCommandCorrelationError;

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
        let (message, mut correlation) = send_status_and_read_response(document)?;
        let result =
            WebDriverBiDiSessionStatusResult::parse_and_correlate(&message, &mut correlation);
        assert!(matches!(
            result,
            Err(WebDriverBiDiSessionStatusResponseError::Correlation { source }) if source == expected
        ));
        assert_eq!(correlation.outstanding_count(), 1);
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

fn send_status_and_read_response(
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
        if command != br#"{"id":7,"method":"session.status","params":{}}"# {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected session.status command",
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
    let established = WebDriverBiDiSessionStatusCommand::new(7)?.send(
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;

    let message = match WebDriverBiDiWebSocketMessageReader::new(established)
        .read_next(Duration::from_millis(500))?
    {
        WebDriverBiDiConnectionMessageRead::Text { message, .. } => message,
        other => {
            return Err(io::Error::other(format!(
                "session.status response produced unexpected message state: {other:?}"
            ))
            .into());
        }
    };

    server
        .join()
        .map_err(|_| io::Error::other("session.status response test server panicked"))??;
    Ok((message, correlation))
}

#[test]
fn session_status_success_result_is_typed_correlated_and_message_redacted_in_debug()
-> Result<(), Box<dyn Error>> {
    let (message, mut correlation) = send_status_and_read_response(STATUS_RESPONSE)?;
    let result = WebDriverBiDiSessionStatusResult::parse_and_correlate(&message, &mut correlation)?;

    assert_eq!(result.command_id(), 7);
    assert!(result.ready());
    assert_eq!(result.message(), "capacity available");
    assert_eq!(correlation.outstanding_count(), 0);

    let debug = format!("{result:?}");
    assert!(debug.contains("message_len"));
    assert!(!debug.contains("capacity available"));
    Ok(())
}

#[test]
fn malformed_status_result_does_not_consume_the_outstanding_command() -> Result<(), Box<dyn Error>>
{
    let (message, mut correlation) = send_status_and_read_response(STATUS_RESPONSE_MISSING_READY)?;
    let parsed = WebDriverBiDiSessionStatusResult::parse_and_correlate(&message, &mut correlation);

    assert!(matches!(
        parsed,
        Err(WebDriverBiDiSessionStatusResponseError::MissingReady)
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn empty_status_result_fails_before_consuming_the_outstanding_command() -> Result<(), Box<dyn Error>>
{
    let (message, mut correlation) = send_status_and_read_response(STATUS_RESPONSE_EMPTY_RESULT)?;
    let parsed = WebDriverBiDiSessionStatusResult::parse_and_correlate(&message, &mut correlation);

    assert!(matches!(
        parsed,
        Err(WebDriverBiDiSessionStatusResponseError::MissingReady)
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn replacement_status_reply_preserves_original_pending_request_and_recovery()
-> Result<(), Box<dyn Error>> {
    let (original, mut pending) = send_status_and_read_response(STATUS_RESPONSE)?;
    let (replacement, _replacement_pending) = send_status_and_read_response(STATUS_RESPONSE)?;

    assert!(matches!(
        WebDriverBiDiSessionStatusResult::parse_and_correlate(&replacement, &mut pending),
        Err(WebDriverBiDiSessionStatusResponseError::Correlation {
            source: originweave_network::WebDriverBiDiCommandCorrelationError::ResponseConnectionMismatch {
                command_id: 7,
            },
        })
    ));
    assert_eq!(pending.outstanding_count(), 1);
    let completed = WebDriverBiDiSessionStatusResult::parse_and_correlate(&original, &mut pending)?;
    assert_eq!(completed.command_id(), 7);
    assert_eq!(pending.outstanding_count(), 0);
    Ok(())
}
