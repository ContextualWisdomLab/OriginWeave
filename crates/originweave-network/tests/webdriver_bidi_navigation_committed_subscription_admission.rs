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
    WebDriverBiDiWebSocketMessageAssembly, advance_webdriver_bidi_navigation_document_epoch,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CONTEXT_ID: &str = "context-a";
const EXPECTED_URL: &str = "https://example.test/after";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const SUBSCRIBE_RESPONSE: &[u8] =
    br#"{"type":"success","id":7,"result":{"subscription":"subscription-a"}}"#;
const NAVIGATION_EVENT: &[u8] = br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"context":"context-a","navigation":"nav-8","timestamp":1234,"url":"https://example.test/after"}}"#;

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
    assembler: &mut WebDriverBiDiWebSocketMessageAssembler,
) -> Result<
    (
        originweave_network::WebDriverBiDiWebSocketEstablished,
        originweave_network::WebDriverBiDiWebSocketTextMessage,
    ),
    Box<dyn Error>,
> {
    let (established, frame) = established.read_frame(Duration::from_millis(500))?;
    match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => Ok((established, text)),
        other => Err(io::Error::other(format!(
            "expected a complete WebDriver BiDi text message, got {other:?}"
        ))
        .into()),
    }
}

fn receive_subscription_result(
    registry: &BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    command_id: u64,
) -> Result<
    (
        WebDriverBiDiNavigationCommittedSubscriptionResult,
        WebDriverBiDiNavigationCommittedSubscriptionBinding,
    ),
    Box<dyn Error>,
> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let expected_command = format!(
        "{{\"id\":{command_id},\"method\":\"session.subscribe\",\"params\":{{\"events\":[\"browsingContext.navigationCommitted\"],\"contexts\":[\"{CONTEXT_ID}\"]}}}}"
    )
    .into_bytes();
    let response = format!(
        "{{\"type\":\"success\",\"id\":{command_id},\"result\":{{\"subscription\":\"subscription-{command_id}\"}}}}"
    )
    .into_bytes();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut stream)?;
        if command != expected_command {
            return Err(io::Error::other(
                "unexpected session.subscribe failure-contract command",
            ));
        }
        write_text_frame(&mut stream, &response)
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

    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        command_id,
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
        WebDriverBiDiWebSocketMaskKey::new([9, 8, 7, 6]),
        Duration::from_millis(500),
    )?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let (_established, response) = next_text(established, &mut assembler)?;
    let result = WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
        &response,
        &mut correlation,
    )?;

    server
        .join()
        .map_err(|_| io::Error::other("subscription result test server panicked"))??;
    Ok((result, binding))
}

#[test]
fn committed_navigation_requires_the_exact_active_subscription_before_document_mutation()
-> Result<(), Box<dyn Error>> {
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
        write_text_frame(&mut stream, SUBSCRIBE_RESPONSE)?;
        write_text_frame(&mut stream, NAVIGATION_EVENT)
    });

    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let pre_navigation_epoch = registry.current_context_epoch(session, context)?;

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

    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7, &registry, session, context, CONTEXT_ID,
    )?;
    let binding = command.admission_binding();
    assert_eq!(binding.command_id(), 7);
    assert_eq!(binding.browser_session(), session);
    assert_eq!(binding.browsing_context(), context);
    let binding_debug = format!("{binding:?}");
    assert!(binding_debug.contains("command_id: 7"));
    assert!(!binding_debug.contains(CONTEXT_ID));

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let established = command.send(
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;

    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let (established, response) = next_text(established, &mut assembler)?;
    let subscription = WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
        &response,
        &mut correlation,
    )?;
    let mut admission = WebDriverBiDiNavigationCommittedSubscriptionAdmission::new(
        subscription,
        binding,
        &registry,
    )?;
    assert_eq!(admission.browser_session(), session);
    assert_eq!(admission.browsing_context(), context);
    let admission_debug = format!("{admission:?}");
    assert!(admission_debug.contains("command_id: 7"));
    assert!(!admission_debug.contains("subscription-a"));

    let (_established, event) = next_text(established, &mut assembler)?;
    let observation = admission.admit(&event, &registry, EXPECTED_URL)?;
    assert_eq!(observation.browser_session(), session);
    assert_eq!(observation.browsing_context(), context);
    assert_eq!(observation.navigation_id(), Some("nav-8"));
    assert_eq!(observation.timestamp(), 1234);
    assert!(
        format!("{observation:?}")
            .contains("WebDriverBiDiNavigationCommittedSubscribedObservation")
    );

    let advanced = advance_webdriver_bidi_navigation_document_epoch(
        observation,
        &mut registry,
        pre_navigation_epoch,
    )?;
    assert_eq!(advanced.browser_session(), session);
    assert_eq!(advanced.browsing_context(), context);

    let replay_error = admission
        .admit(&event, &registry, EXPECTED_URL)
        .err()
        .ok_or_else(|| io::Error::other("replayed navigation event unexpectedly readmitted"))?;
    assert_eq!(
        replay_error.to_string(),
        "WebDriver BiDi navigation-committed event was already admitted by this active subscription"
    );
    assert!(replay_error.source().is_none());
    assert_eq!(
        registry.current_context_epoch(session, context)?,
        advanced.current_epoch()
    );

    registry.remove_context(context)?;
    let stale_error = admission
        .admit(&event, &registry, EXPECTED_URL)
        .err()
        .ok_or_else(|| {
            io::Error::other("retired context unexpectedly admitted navigation event")
        })?;
    assert!(stale_error.source().is_some());

    let unsubscribe = admission.into_unsubscribe(8)?;
    assert_eq!(unsubscribe.command_id(), 8);

    server
        .join()
        .map_err(|_| io::Error::other("subscription admission test server panicked"))??;
    Ok(())
}

#[test]
fn subscription_admission_rejects_mismatched_command_and_retired_context()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;

    let (subscription, _) = receive_subscription_result(&registry, session, context, 8)?;
    let wrong_binding = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        9, &registry, session, context, CONTEXT_ID,
    )?
    .admission_binding();
    let mismatch = WebDriverBiDiNavigationCommittedSubscriptionAdmission::new(
        subscription,
        wrong_binding,
        &registry,
    )
    .err()
    .ok_or_else(|| io::Error::other("mismatched subscription command unexpectedly admitted"))?;
    assert_eq!(
        mismatch.to_string(),
        "WebDriver BiDi navigation subscription response does not match its command binding"
    );
    assert!(mismatch.source().is_none());

    let (subscription, binding) = receive_subscription_result(&registry, session, context, 10)?;
    registry.remove_context(context)?;
    let retired = WebDriverBiDiNavigationCommittedSubscriptionAdmission::new(
        subscription,
        binding,
        &registry,
    )
    .err()
    .ok_or_else(|| io::Error::other("retired subscription context unexpectedly admitted"))?;
    assert_eq!(
        retired.to_string(),
        "WebDriver BiDi navigation subscription context is no longer registered authority"
    );
    assert!(retired.source().is_some());
    Ok(())
}
