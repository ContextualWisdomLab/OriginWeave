use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiNavigationCommittedSubscriptionResult,
    WebDriverBiDiNavigationCommittedUnsubscribeCommand,
    WebDriverBiDiNavigationCommittedUnsubscribeResult, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CONTEXT_ID: &str = "context-a";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const SUBSCRIBE_RESPONSE: &[u8] = br#"{"type":"success","id":7,"result":{"subscription":"sub-\"\\\n\u0001-구독"}}"#;
const UNSUBSCRIBE_RESPONSE: &[u8] = br#"{"type":"success","id":8,"result":{}}"#;

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

fn write_server_text_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    if payload.len() > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "test server payload unexpectedly exceeded short-frame encoding",
        ));
    }
    stream.write_all(&[0x81, payload.len() as u8])?;
    stream.write_all(payload)
}

#[test]
fn validated_subscription_can_be_unsubscribed_without_losing_opaque_text()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;

        let subscribe = read_masked_text_frame(&mut stream)?;
        if subscribe
            != br#"{"id":7,"method":"session.subscribe","params":{"events":["browsingContext.navigationCommitted"],"contexts":["context-a"]}}"#
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unexpected session.subscribe command: {}",
                    String::from_utf8_lossy(&subscribe)
                ),
            ));
        }
        write_server_text_frame(&mut stream, SUBSCRIBE_RESPONSE)?;

        let unsubscribe = read_masked_text_frame(&mut stream)?;
        if unsubscribe
            != br#"{"id":8,"method":"session.unsubscribe","params":{"subscriptions":["sub-\"\\\n\u0001-구독"]}}"#
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unexpected session.unsubscribe command: {}",
                    String::from_utf8_lossy(&unsubscribe)
                ),
            ));
        }
        write_server_text_frame(&mut stream, UNSUBSCRIBE_RESPONSE)
    });

    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;

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
    let subscribe = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7, &registry, session, context, CONTEXT_ID,
    )?;
    let established = subscribe.send(
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;

    let (established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "session.subscribe response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    let subscription = WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
        &text,
        &mut correlation,
    )?;
    assert_eq!(subscription.subscription_id(), "sub-\"\\\n\u{0001}-구독");

    let unsubscribe = WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(8, &subscription)?;
    assert_eq!(unsubscribe.command_id(), 8);
    let established = unsubscribe.send(
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
        Duration::from_millis(500),
    )?;
    assert_eq!(correlation.outstanding_count(), 1);

    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "session.unsubscribe response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    let result = WebDriverBiDiNavigationCommittedUnsubscribeResult::parse_and_correlate(
        &text,
        &mut correlation,
    )?;
    assert_eq!(result.command_id(), 8);
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("session.unsubscribe command test server panicked"))??;
    Ok(())
}
