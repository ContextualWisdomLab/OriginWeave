use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserSessionId, BrowsingContextId, WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiNavigationCommittedSubscriptionAdmission,
    WebDriverBiDiNavigationCommittedSubscriptionBinding,
    WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiNavigationCommittedSubscriptionResult, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CONTEXT_ID: &str = "context-a";
const EXPECTED_URL: &str = "https://example.test/after";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const MISSING_NAVIGATION_EVENT: &[u8] = br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"context":"context-a","navigation":null,"timestamp":1234,"url":"https://example.test/after"}}"#;

fn read_opening_request(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    let mut buffer = [0_u8; 512];
    while !request.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "opening request ended before the header terminator",
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
                "test payload unexpectedly required 64-bit framing",
            ));
        }
    }
    stream.write_all(payload)
}

fn next_text(
    established: originweave_network::WebDriverBiDiWebSocketEstablished,
) -> Result<originweave_network::WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => Ok(text),
        other => Err(io::Error::other(format!(
            "expected a complete WebDriver BiDi text message, got {other:?}"
        ))
        .into()),
    }
}

fn establish(
    local_addr: std::net::SocketAddr,
) -> Result<originweave_network::WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
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

fn receive_subscription_result(
    registry: &BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
) -> Result<
    (
        WebDriverBiDiNavigationCommittedSubscriptionResult,
        WebDriverBiDiNavigationCommittedSubscriptionBinding,
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
        if command
            != br#"{"id":7,"method":"session.subscribe","params":{"events":["browsingContext.navigationCommitted"],"contexts":["context-a"]}}"#
        {
            return Err(io::Error::other("unexpected session.subscribe command"));
        }
        write_text_frame(
            &mut stream,
            br#"{"type":"success","id":7,"result":{"subscription":"subscription-a"}}"#,
        )
    });

    let established = establish(local_addr)?;
    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7,
        registry,
        browser_session,
        browsing_context,
        CONTEXT_ID,
    )?;
    let binding = command.admission_binding();
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let established = command.send(
        registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;
    let response = next_text(established)?;
    let result = WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
        &response,
        &mut correlation,
    )?;

    server
        .join()
        .map_err(|_| io::Error::other("subscription failure-contract server panicked"))??;
    Ok((result, binding))
}

fn receive_event(
    payload: &'static [u8],
) -> Result<originweave_network::WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_text_frame(&mut stream, payload)
    });

    let event = next_text(establish(local_addr)?)?;
    server
        .join()
        .map_err(|_| io::Error::other("event failure-contract server panicked"))??;
    Ok(event)
}

#[test]
fn subscription_event_failures_keep_specific_public_diagnostics() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let (subscription, binding) = receive_subscription_result(&registry, session, context)?;
    let mut admission = WebDriverBiDiNavigationCommittedSubscriptionAdmission::new(
        subscription,
        binding,
        &registry,
    )?;
    let missing_navigation_event = receive_event(MISSING_NAVIGATION_EVENT)?;

    let missing_navigation = admission
        .admit(&missing_navigation_event, &registry, EXPECTED_URL)
        .err()
        .ok_or_else(|| io::Error::other("null navigation identity unexpectedly admitted"))?;
    assert_eq!(
        missing_navigation.to_string(),
        "WebDriver BiDi navigation-committed event has no reusable-safe navigation identity"
    );
    assert!(missing_navigation.source().is_none());

    registry.remove_context(context)?;
    let stale_context = admission
        .admit(&missing_navigation_event, &registry, EXPECTED_URL)
        .err()
        .ok_or_else(|| io::Error::other("retired context unexpectedly admitted an event"))?;
    assert_eq!(
        stale_context.to_string(),
        "WebDriver BiDi navigation subscription context is no longer registered authority"
    );
    assert!(stale_context.source().is_some());

    Ok(())
}
