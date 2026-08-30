use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    BrowserAuthorityRegistry, WebDriverBiDiPointerClickCommand, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiNavigationCommittedObservation,
    WebDriverBiDiNavigationCommittedObservationError, WebDriverBiDiPointerClickResult,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTextMessage, send_webdriver_bidi_pointer_click,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const EXPECTED_URL: &str = "https://example.test/after";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const CLICK_SUCCESS_RESPONSE: &[u8] = br#"{"type":"success","id":42,"result":{}}"#;
const NAVIGATION_COMMITTED_EVENT: &[u8] = br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"context":"context-a","navigation":"nav-42","timestamp":1234,"url":"https://example.test/after","vendorExtension":{"ignored":true}}}"#;

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
            usize::try_from(u64::from_be_bytes(extended)).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "client frame length exceeds usize",
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

fn assemble_text(
    assembler: &mut WebDriverBiDiWebSocketMessageAssembler,
    frame: originweave_network::WebDriverBiDiWebSocketFrame,
) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => Ok(text),
        other => Err(io::Error::other(format!(
            "navigation post-condition produced unexpected assembly state: {other:?}"
        ))
        .into()),
    }
}

fn click_then_observe_navigation() -> Result<
    (
        WebDriverBiDiWebSocketTextMessage,
        BrowserAuthorityRegistry,
        originweave_core::BrowserSessionId,
        originweave_core::BrowsingContextId,
    ),
    Box<dyn Error>,
> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let expected = WebDriverBiDiPointerClickCommand::new(
        42,
        "context-a",
        &WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?,
    )?;
    let expected_json = expected.as_json().as_bytes().to_vec();

    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut stream)?;
        if command != expected_json {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected input.performActions pointer-click command",
            ));
        }
        write_unmasked_text_frame(&mut stream, CLICK_SUCCESS_RESPONSE)?;
        write_unmasked_text_frame(&mut stream, NAVIGATION_COMMITTED_EVENT)
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

    let command = WebDriverBiDiPointerClickCommand::new(
        42,
        "context-a",
        &WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?,
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let established = send_webdriver_bidi_pointer_click(
        &command,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;

    let (established, response_frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let response_text = assemble_text(&mut assembler, response_frame)?;
    let response =
        WebDriverBiDiPointerClickResult::parse_and_correlate(&response_text, &mut correlation)?;
    if response.command_id() != 42 || correlation.outstanding_count() != 0 {
        return Err(
            io::Error::other("pointer-click acknowledgment was not correlated exactly").into(),
        );
    }

    let (_established, event_frame) = established.read_frame(Duration::from_millis(500))?;
    let event_text = assemble_text(&mut assembler, event_frame)?;
    server
        .join()
        .map_err(|_| io::Error::other("navigation post-condition test server panicked"))??;

    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "context-a")?;
    Ok((event_text, registry, session, context))
}

#[test]
fn navigation_committed_event_proves_exact_context_and_declared_url_post_condition()
-> Result<(), Box<dyn Error>> {
    let (event, registry, session, context) = click_then_observe_navigation()?;
    let observation = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &event,
        &registry,
        session,
        context,
        EXPECTED_URL,
    )?;

    assert_eq!(observation.browser_session(), session);
    assert_eq!(observation.browsing_context(), context);
    assert_eq!(observation.navigation_id(), Some("nav-42"));
    assert_eq!(observation.timestamp(), 1234);
    assert_eq!(observation.url(), EXPECTED_URL);
    Ok(())
}

#[test]
fn navigation_observation_fails_closed_for_wrong_url_or_registered_context()
-> Result<(), Box<dyn Error>> {
    let (event, mut registry, session, context) = click_then_observe_navigation()?;

    let wrong_url = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &event,
        &registry,
        session,
        context,
        "https://example.test/not-the-post-condition",
    );
    assert!(matches!(
        wrong_url,
        Err(WebDriverBiDiNavigationCommittedObservationError::UnexpectedUrl)
    ));

    let other_context = registry.register_context(session, "context-b")?;
    let wrong_context = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &event,
        &registry,
        session,
        other_context,
        EXPECTED_URL,
    );
    assert!(matches!(
        wrong_context,
        Err(WebDriverBiDiNavigationCommittedObservationError::ContextBinding { .. })
    ));
    assert_eq!(registry.current_context_epoch(session, context)?.value(), 1);
    assert_eq!(
        registry
            .current_context_epoch(session, other_context)?
            .value(),
        1
    );
    Ok(())
}
