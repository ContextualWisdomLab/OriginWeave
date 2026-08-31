use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, Origin, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    WebDriverBiDiNavigationCommittedObservation, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTextMessage, advance_and_bind_webdriver_bidi_navigation_document_origin,
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

fn write_unmasked_text_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    stream.write_all(&[0x81])?;
    match payload.len() {
        0..=125 => stream.write_all(&[payload.len() as u8])?,
        126..=65_535 => {
            stream.write_all(&[126])?;
            stream.write_all(&(payload.len() as u16).to_be_bytes())?;
        }
        _ => {
            stream.write_all(&[127])?;
            stream.write_all(&(payload.len() as u64).to_be_bytes())?;
        }
    }
    stream.write_all(payload)
}

fn receive_navigation_event(url: &str) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let payload = format!(
        "{{\"type\":\"event\",\"method\":\"browsingContext.navigationCommitted\",\"params\":{{\"context\":\"context-a\",\"navigation\":\"navigation-a\",\"timestamp\":17,\"url\":\"{url}\"}}}}"
    )
    .into_bytes();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_unmasked_text_frame(&mut stream, &payload)
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
                "navigation origin-binding event produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    server
        .join()
        .map_err(|_| io::Error::other("navigation origin-binding server panicked"))??;
    Ok(text)
}

fn fixture_origin(value: &str) -> Result<Origin, Box<dyn Error>> {
    Origin::parse(value)
        .map_err(|error| io::Error::other(format!("fixture origin parse failed: {error:?}" )).into())
}

#[test]
fn committed_navigation_rotates_document_and_binds_canonical_observed_origin()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "context-a")?;
    let before = registry.current_context_epoch(session, context)?;
    let previous_origin = fixture_origin("https://before.example")?;
    registry.bind_context_origin(session, context, &previous_origin)?;

    let observed_url = "https://EXAMPLE.TEST:443/after?from=originweave#done";
    let event = receive_navigation_event(observed_url)?;
    let observation = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &event,
        &registry,
        session,
        context,
        observed_url,
    )?;
    let binding = advance_and_bind_webdriver_bidi_navigation_document_origin(
        observation,
        &mut registry,
        before,
    )?;

    let expected_origin = fixture_origin("https://example.test")?;
    assert_eq!(binding.browser_session(), session);
    assert_eq!(binding.browsing_context(), context);
    assert_eq!(binding.previous_epoch(), before);
    assert_ne!(binding.current_epoch(), before);
    assert_eq!(binding.origin(), &expected_origin);
    assert_eq!(
        registry.require_context_origin(session, context, &expected_origin)?,
        binding.current_epoch()
    );
    Ok(())
}

#[test]
fn invalid_observed_origin_fails_before_document_authority_is_rotated()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "context-a")?;
    let before = registry.current_context_epoch(session, context)?;
    let previous_origin = fixture_origin("https://before.example")?;
    registry.bind_context_origin(session, context, &previous_origin)?;

    let observed_url = "https://user@example.test/after";
    let event = receive_navigation_event(observed_url)?;
    let observation = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &event,
        &registry,
        session,
        context,
        observed_url,
    )?;
    let error = advance_and_bind_webdriver_bidi_navigation_document_origin(
        observation,
        &mut registry,
        before,
    )
    .err()
    .ok_or_else(|| io::Error::other("credential-bearing observed URL unexpectedly bound origin"))?;

    assert_eq!(
        error.to_string(),
        "WebDriver BiDi committed navigation URL cannot enter canonical origin authority"
    );
    assert!(error.source().is_none());
    assert_eq!(registry.current_context_epoch(session, context)?, before);
    assert_eq!(
        registry.require_context_origin(session, context, &previous_origin)?,
        before
    );
    Ok(())
}
