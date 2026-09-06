use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCommandKind, WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiNavigationCommittedSubscriptionResponseError,
    WebDriverBiDiNavigationCommittedSubscriptionResult, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTextMessage,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const SUBSCRIBE_SUCCESS_RESPONSE: &[u8] =
    br#"{"type":"success","id":42,"result":{"subscription":"subscription-a"}}"#;
const SUBSCRIBE_ERROR_RESPONSE: &[u8] =
    br#"{"type":"error","id":42,"error":"invalid argument","message":"blocked","stacktrace":"remote"}"#;

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
    let marker = header[1] & 0x7f;
    let length = match marker {
        0..=125 => usize::from(marker),
        126 => {
            let mut extended = [0_u8; 2];
            stream.read_exact(&mut extended)?;
            usize::from(u16::from_be_bytes(extended))
        }
        127 => {
            let mut extended = [0_u8; 8];
            stream.read_exact(&mut extended)?;
            let length = u64::from_be_bytes(extended);
            usize::try_from(length).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "subscription frame length exceeds usize",
                )
            })?
        }
        _ => unreachable!(),
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

fn establish(local_addr: SocketAddr) -> Result<WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(SESSION_ID)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    Ok(WebDriverBiDiWebSocketHandshakePlan::new(
        connection,
        WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?,
    )?
    .write_opening_request(Duration::from_millis(500))?
    .read_opening_response(Duration::from_millis(500))?)
}

fn read_response(
    established: WebDriverBiDiWebSocketEstablished,
) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let (_, frame) = established.read_frame(Duration::from_millis(500))?;
    let text = match WebDriverBiDiWebSocketMessageAssembler::new().push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(message) => message,
        other => {
            return Err(io::Error::other(format!(
                "replacement subscription connection produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    Ok(text)
}

fn assert_replacement_rejected(foreign_response: &'static [u8]) -> Result<(), Box<dyn Error>> {
    let original_listener = TcpListener::bind(("127.0.0.1", 0))?;
    let original_addr = original_listener.local_addr()?;
    let expected_json = br#"{"id":42,"method":"session.subscribe","params":{"events":["browsingContext.navigationCommitted"],"contexts":["context-a"]}}"#.to_vec();
    let original_server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = original_listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut stream)?;
        if command != expected_json {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected subscription command on original connection",
            ));
        }
        let (mut replacement, _) = original_listener.accept()?;
        read_opening_request(&mut replacement)?;
        replacement.write_all(OPENING_RESPONSE)?;
        replacement.write_all(&[0x81, foreign_response.len() as u8])?;
        replacement.write_all(foreign_response)?;
        stream.write_all(&[0x81, SUBSCRIBE_SUCCESS_RESPONSE.len() as u8])?;
        stream.write_all(SUBSCRIBE_SUCCESS_RESPONSE)
    });

    let original = establish(original_addr)?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "context-a")?;
    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        42,
        &registry,
        session,
        context,
        "context-a",
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(43, WebDriverBiDiCommandKind::SessionStatus)?;
    let original = command.send(
        &registry,
        original,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;
    assert_eq!(correlation.outstanding_count(), 2);

    let replacement_response = read_response(establish(original_addr)?)?;
    let parsed = WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
        &replacement_response,
        &mut correlation,
    );

    let original_response = read_response(original)?;
    original_server
        .join()
        .map_err(|_| io::Error::other("original subscription server panicked"))??;
    assert!(
        matches!(
            parsed,
            Err(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                    source: WebDriverBiDiCommandCorrelationError::ResponseConnectionMismatch {
                        command_id: 42
                    }
                }
            )
        ),
        "replacement response must fail for exact connection mismatch: {parsed:?}"
    );
    assert_eq!(correlation.outstanding_count(), 2);
    let accepted = WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
        &original_response,
        &mut correlation,
    )?;
    assert_eq!(accepted.command_id(), 42);
    assert_eq!(accepted.subscription_id(), "subscription-a");
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn replacement_success_cannot_consume_original_subscription_command() -> Result<(), Box<dyn Error>>
{
    assert_replacement_rejected(SUBSCRIBE_SUCCESS_RESPONSE)
}

#[test]
fn replacement_error_cannot_consume_original_subscription_command() -> Result<(), Box<dyn Error>> {
    assert_replacement_rejected(SUBSCRIBE_ERROR_RESPONSE)
}
