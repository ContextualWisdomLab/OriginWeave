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
    WebDriverBiDiJsonEnvelopeError, WebDriverBiDiNavigationCommittedSubscriptionResponseError,
    WebDriverBiDiNavigationCommittedSubscriptionResult, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTextMessage,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const MALFORMED_SUCCESS_RESPONSE: &[u8] = br#"{"type":"success","id":7,"result":"#;
const MISSING_SUBSCRIPTION_RESPONSE: &[u8] = br#"{"type":"success","id":7,"result":{"extra":1}}"#;
const UNKNOWN_SUCCESS_RESPONSE: &[u8] =
    br#"{"type":"success","id":8,"result":{"subscription":"subscription-b"}}"#;
const MATCHED_ERROR_RESPONSE: &[u8] =
    br#"{"type":"error","id":7,"error":"invalid argument","message":"denied"}"#;
const UNKNOWN_ERROR_RESPONSE: &[u8] =
    br#"{"type":"error","id":8,"error":"invalid argument","message":"denied"}"#;
const NAVIGATION_EVENT: &[u8] =
    br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{}}"#;

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
                "subscription response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };

    server
        .join()
        .map_err(|_| io::Error::other("subscription response test server panicked"))??;
    Ok(text)
}

#[test]
fn malformed_and_invalid_success_responses_preserve_outstanding_correlation()
-> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(7)?;

    let malformed = read_text_over_loopback(MALFORMED_SUCCESS_RESPONSE)?;
    assert_eq!(
        WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
            &malformed,
            &mut correlation,
        ),
        Err(
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::Envelope {
                source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
            }
        )
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let missing = read_text_over_loopback(MISSING_SUBSCRIPTION_RESPONSE)?;
    assert_eq!(
        WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
            &missing,
            &mut correlation,
        ),
        Err(WebDriverBiDiNavigationCommittedSubscriptionResponseError::MissingSubscription)
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let unknown = read_text_over_loopback(UNKNOWN_SUCCESS_RESPONSE)?;
    assert_eq!(
        WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
            &unknown,
            &mut correlation,
        ),
        Err(
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
            }
        )
    );
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn protocol_error_consumes_only_its_exact_outstanding_command() -> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(7)?;

    let unknown = read_text_over_loopback(UNKNOWN_ERROR_RESPONSE)?;
    assert_eq!(
        WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
            &unknown,
            &mut correlation,
        ),
        Err(
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
            }
        )
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let matched = read_text_over_loopback(MATCHED_ERROR_RESPONSE)?;
    assert_eq!(
        WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
            &matched,
            &mut correlation,
        ),
        Err(
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::RemoteProtocolError {
                command_id: 7,
                error_code: "invalid argument".to_owned(),
            }
        )
    );
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn event_response_is_rejected_without_consuming_outstanding_command() -> Result<(), Box<dyn Error>>
{
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(7)?;
    let event = read_text_over_loopback(NAVIGATION_EVENT)?;

    assert_eq!(
        WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
            &event,
            &mut correlation,
        ),
        Err(
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                source: WebDriverBiDiCommandCorrelationError::EventIsNotResponse,
            }
        )
    );
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}
