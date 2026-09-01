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
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiTextValueObservationAuthorityError, WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiTextValueObservationSendError, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, send_webdriver_bidi_text_value_observation,
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
type ObservationFixture = (
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

fn observation_fixture() -> Result<ObservationFixture, Box<dyn Error>> {
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
fn observation_rejects_non_semantic_proof_before_correlation_or_frame_write()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (registry, handle, remote) = observation_fixture()?;

    let error = send_webdriver_bidi_text_value_observation(
        protocol_proof(
            BrowserProtocolKind::WebDriverBiDi,
            BrowserProtocolCapability::TypedInput,
        )?,
        43,
        "context-a",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("typed-input proof unexpectedly sent observation"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTextValueObservationSendError::UnsupportedCapability(
            BrowserProtocolCapability::TypedInput
        )
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-value observation send requires semantic-observation capability"
    );
    assert!(error.source().is_none());
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("semantic capability rejection server panicked"))??;
    Ok(())
}

#[test]
fn observation_rejects_non_webdriver_bidi_proof_before_correlation_or_frame_write()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (registry, handle, remote) = observation_fixture()?;

    let error = send_webdriver_bidi_text_value_observation(
        protocol_proof(
            BrowserProtocolKind::ChromeDevToolsProtocol,
            BrowserProtocolCapability::SemanticObservation,
        )?,
        44,
        "context-a",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("CDP proof unexpectedly sent observation"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTextValueObservationSendError::UnsupportedProtocolKind(
            BrowserProtocolKind::ChromeDevToolsProtocol
        )
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-value observation send requires a WebDriver BiDi proof"
    );
    assert!(error.source().is_none());
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("WebDriver BiDi proof rejection server panicked"))??;
    Ok(())
}

#[test]
fn observation_rejects_stale_external_context_before_correlation_or_frame_write()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (registry, handle, remote) = observation_fixture()?;

    let error = send_webdriver_bidi_text_value_observation(
        semantic_observation_proof()?,
        45,
        "context-b",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("wrong context unexpectedly reached WebSocket I/O"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTextValueObservationSendError::Authority {
            source: WebDriverBiDiTextValueObservationAuthorityError::BrowserAuthority(
                BrowserRegistryError::ContextExternalIdentifierMismatch
            )
        }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-value observation authority was rejected"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("authority rejection server panicked"))??;
    Ok(())
}

#[test]
fn observation_rejects_duplicate_correlation_before_frame_write() -> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(46)?;
    let (registry, handle, remote) = observation_fixture()?;

    let error = send_webdriver_bidi_text_value_observation(
        semantic_observation_proof()?,
        46,
        "context-a",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("duplicate correlation unexpectedly sent observation"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTextValueObservationSendError::Correlation { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-value observation command correlation was rejected"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("duplicate-correlation observation server panicked"))??;
    Ok(())
}

#[test]
fn observation_preserves_registration_when_frame_timeout_is_invalid() -> Result<(), Box<dyn Error>>
{
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (registry, handle, remote) = observation_fixture()?;

    let error = send_webdriver_bidi_text_value_observation(
        semantic_observation_proof()?,
        47,
        "context-a",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
        Duration::ZERO,
    )
    .err()
    .ok_or_else(|| io::Error::other("zero frame timeout unexpectedly sent observation"))?;
    assert!(matches!(
        error,
        WebDriverBiDiTextValueObservationSendError::FrameWrite { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-value observation command frame write failed"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("invalid-timeout observation server panicked"))??;
    Ok(())
}
