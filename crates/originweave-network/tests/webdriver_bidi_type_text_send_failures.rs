use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    AdmittedNodeHandle, BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, Origin, OriginWeaveProtocolVersion,
    ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiRemoteNodeReference, WebDriverBiDiTypeTextAuthorityError,
    WebDriverBiDiTypeTextCommandError, WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiTypeTextSendError, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, send_webdriver_bidi_type_text,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

type HandshakeOnlyServer = (
    WebDriverBiDiWebSocketEstablished,
    thread::JoinHandle<io::Result<()>>,
);
type TypeTextFixture = (
    BrowserAuthorityRegistry,
    AdmittedNodeHandle,
    WebDriverBiDiRemoteNodeReference,
);

fn protocol_proof(
    kind: BrowserProtocolKind,
    capability: BrowserProtocolCapability,
) -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        kind,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[capability],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        kind,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        capability,
    )?)
}

fn semantic_observation_proof() -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    protocol_proof(
        BrowserProtocolKind::WebDriverBiDi,
        BrowserProtocolCapability::SemanticObservation,
    )
}

fn typed_input_proof() -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    protocol_proof(
        BrowserProtocolKind::WebDriverBiDi,
        BrowserProtocolCapability::TypedInput,
    )
}

fn type_text_fixture() -> Result<TypeTextFixture, Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session = registry.register_session("webdriver-session")?;
    let browsing_context = registry.register_context(browser_session, "context-a")?;
    let origin = Origin::parse("https://app.example").map_err(|error| {
        io::Error::other(format!("fixture origin rejected unexpectedly: {error:?}"))
    })?;
    let epoch = registry.bind_context_origin(browser_session, browsing_context, &origin)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(browser_session, browsing_context),
            &origin,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), Some("Task title"), 1)?;
    let locate = WebDriverBiDiLocateNodesCommand::new(41, "context-a", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":41,"result":{"nodes":[{"type":"node","sharedId":"shared-node-42"}]}}"#,
    )?;
    let handle = locate
        .bind_response_document_nodes(
            document,
            semantic_observation_proof()?,
            &mut registry,
            target,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("locateNodes fixture did not bind its node"))?;
    let remote = WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?;
    Ok((registry, handle, remote))
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

fn establish_with_handshake_only_server() -> Result<HandshakeOnlyServer, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)
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
    Ok((established, server))
}

#[test]
fn type_text_rejects_non_typed_input_proof_before_correlation_or_frame_write()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (registry, handle, remote) = type_text_fixture()?;

    let error = send_webdriver_bidi_type_text(
        semantic_observation_proof()?,
        5,
        "context-a",
        "Quarterly review",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("semantic-observation proof unexpectedly sent text input"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextSendError::UnsupportedCapability(
            BrowserProtocolCapability::SemanticObservation
        )
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-input send requires typed-input capability"
    );
    assert!(error.source().is_none());
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("typed-input capability rejection server panicked"))??;
    Ok(())
}

#[test]
fn type_text_rejects_non_webdriver_bidi_proof_before_correlation_or_frame_write()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (registry, handle, remote) = type_text_fixture()?;

    let error = send_webdriver_bidi_type_text(
        protocol_proof(
            BrowserProtocolKind::ChromeDevToolsProtocol,
            BrowserProtocolCapability::TypedInput,
        )?,
        6,
        "context-a",
        "Quarterly review",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("CDP typed-input proof unexpectedly sent text input"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextSendError::UnsupportedProtocolKind(
            BrowserProtocolKind::ChromeDevToolsProtocol
        )
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-input send requires a WebDriver BiDi proof"
    );
    assert!(error.source().is_none());
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("WebDriver BiDi proof rejection server panicked"))??;
    Ok(())
}

#[test]
fn type_text_rejects_invalid_text_before_correlation_or_frame_write() -> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (registry, handle, remote) = type_text_fixture()?;

    let error = send_webdriver_bidi_type_text(
        typed_input_proof()?,
        7,
        "context-a",
        "buyer-private\ntext",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("invalid text unexpectedly reached WebSocket I/O"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextSendError::Authority {
            source: WebDriverBiDiTypeTextAuthorityError::Command(
                WebDriverBiDiTypeTextCommandError::InvalidText
            )
        }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-input authority was rejected"
    );
    assert!(error.source().is_some());
    assert!(!format!("{error:?}").contains("buyer-private"));
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("invalid-text rejection server panicked"))??;
    Ok(())
}

#[test]
fn type_text_rejects_duplicate_correlation_before_frame_write() -> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(8)?;
    let (registry, handle, remote) = type_text_fixture()?;

    let error = send_webdriver_bidi_type_text(
        typed_input_proof()?,
        8,
        "context-a",
        "Quarterly review",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("duplicate correlation unexpectedly sent text input"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextSendError::Correlation { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-input command correlation was rejected"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("duplicate-correlation text server panicked"))??;
    Ok(())
}

#[test]
fn type_text_preserves_registration_when_frame_timeout_is_invalid() -> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (registry, handle, remote) = type_text_fixture()?;

    let error = send_webdriver_bidi_type_text(
        typed_input_proof()?,
        11,
        "context-a",
        "Quarterly review",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
        Duration::ZERO,
    )
    .err()
    .ok_or_else(|| io::Error::other("zero frame timeout unexpectedly sent text input"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextSendError::FrameWrite { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-input command frame write failed"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("invalid-timeout text server panicked"))??;
    Ok(())
}
