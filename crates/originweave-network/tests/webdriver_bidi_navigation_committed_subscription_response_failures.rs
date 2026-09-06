use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCommandKind, WebDriverBiDiConnectionMessageRead, WebDriverBiDiJsonEnvelopeError,
    WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiNavigationCommittedSubscriptionResponseError,
    WebDriverBiDiNavigationCommittedSubscriptionResult, WebDriverBiDiReceivedTextMessage,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageReader,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const MALFORMED_SUCCESS_RESPONSE: &[u8] = br#"{"type":"success","id":7,"result":"#;
const MISSING_SUBSCRIPTION_RESPONSE: &[u8] = br#"{"type":"success","id":7,"result":{"extra":1}}"#;
const MATCHED_SUCCESS_RESPONSE: &[u8] =
    br#"{"type":"success","id":7,"result":{"subscription":"subscription-a"}}"#;
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

fn read_masked_text_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x81 || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected one final masked client text frame",
        ));
    }
    let length = match header[1] & 0x7f {
        length @ 0..=125 => usize::from(length),
        126 => {
            let mut extended = [0_u8; 2];
            stream.read_exact(&mut extended)?;
            usize::from(u16::from_be_bytes(extended))
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "test command unexpectedly required 64-bit framing",
            ));
        }
    };
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
}

fn read_text_over_loopback(
    document: &'static [u8],
) -> Result<WebDriverBiDiReceivedTextMessage, Box<dyn Error>> {
    read_response_over_loopback(document, None)
}

fn read_response_over_loopback(
    document: &'static [u8],
    correlation: Option<&mut WebDriverBiDiCommandCorrelation>,
) -> Result<WebDriverBiDiReceivedTextMessage, Box<dyn Error>> {
    let send_command = correlation.is_some();
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        if send_command {
            assert_eq!(read_masked_text_frame(&mut stream)?, br#"{"id":7,"method":"session.subscribe","params":{"events":["browsingContext.navigationCommitted"],"contexts":["context-a"]}}"#);
        }
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
    let established = if let Some(correlation) = correlation {
        let mut registry = BrowserAuthorityRegistry::new();
        let session = registry.register_session(SESSION_ID)?;
        let context = registry.register_context(session, "context-a")?;
        WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
            7,
            &registry,
            session,
            context,
            "context-a",
        )?
        .send(
            &registry,
            established,
            correlation,
            WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
            Duration::from_millis(500),
        )?
    } else {
        established
    };
    let text = match WebDriverBiDiWebSocketMessageReader::new(established)
        .read_next(Duration::from_millis(500))?
    {
        WebDriverBiDiConnectionMessageRead::Text { message, .. } => message,
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
    correlation
        .register_command_for(7, WebDriverBiDiCommandKind::NavigationCommittedSubscription)?;

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
    correlation.register_command_for(43, WebDriverBiDiCommandKind::SessionStatus)?;

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

    let matched = read_response_over_loopback(MATCHED_ERROR_RESPONSE, Some(&mut correlation))?;
    assert_eq!(correlation.outstanding_count(), 2);
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
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn subscription_response_cannot_consume_another_command_kind() -> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(7, WebDriverBiDiCommandKind::SessionStatus)?;
    let response = read_text_over_loopback(MATCHED_SUCCESS_RESPONSE)?;

    assert_eq!(
        WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
            &response,
            &mut correlation,
        ),
        Err(
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                source: WebDriverBiDiCommandCorrelationError::CommandKindMismatch {
                    expected: WebDriverBiDiCommandKind::NavigationCommittedSubscription,
                    actual: WebDriverBiDiCommandKind::SessionStatus,
                },
            }
        )
    );
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn event_response_is_rejected_without_consuming_outstanding_command() -> Result<(), Box<dyn Error>>
{
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation
        .register_command_for(7, WebDriverBiDiCommandKind::NavigationCommittedSubscription)?;
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

#[test]
fn manually_registered_subscription_cannot_supply_sender_provenance() -> Result<(), Box<dyn Error>>
{
    for document in [MATCHED_SUCCESS_RESPONSE, MATCHED_ERROR_RESPONSE] {
        let mut correlation = WebDriverBiDiCommandCorrelation::new();
        correlation
            .register_command_for(7, WebDriverBiDiCommandKind::NavigationCommittedSubscription)?;
        let message = read_text_over_loopback(document)?;
        assert_eq!(
            WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
                &message,
                &mut correlation
            ),
            Err(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                    source:
                        WebDriverBiDiCommandCorrelationError::CommandConnectionProvenanceMissing {
                            command_id: 7
                        },
                }
            )
        );
        assert_eq!(correlation.outstanding_count(), 1);
    }
    Ok(())
}
