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
    WebDriverBiDiPointerClickAuthorityError, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiPointerClickSendError,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    send_webdriver_bidi_pointer_click,
};

const REGISTRY_SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const FOREIGN_TRANSPORT_SESSION_ID: &str = "fedcba98-7654-3210-fedc-ba9876543210";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn protocol_proof(
    capability: BrowserProtocolCapability,
) -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[capability],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        capability,
    )?)
}

fn current_node_fixture() -> Result<
    (
        BrowserAuthorityRegistry,
        AdmittedNodeHandle,
        WebDriverBiDiRemoteNodeReference,
    ),
    Box<dyn Error>,
> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session = registry.register_session(REGISTRY_SESSION_ID)?;
    let browsing_context = registry.register_context(browser_session, "context-a")?;
    let origin = Origin::parse("https://app.example")
        .map_err(|error| io::Error::other(format!("fixture origin rejected: {error:?}")))?;
    let epoch = registry.bind_context_origin(browser_session, browsing_context, &origin)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(browser_session, browsing_context),
            &origin,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 1)?;
    let locate = WebDriverBiDiLocateNodesCommand::new(41, "context-a", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":41,"result":{"nodes":[{"type":"node","sharedId":"shared-node-42"}]}}"#,
    )?;
    let handle = locate
        .bind_response_document_nodes(
            document,
            protocol_proof(BrowserProtocolCapability::SemanticObservation)?,
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
                "opening request ended before the header terminator",
            ));
        }
        request.extend_from_slice(&buffer[..count]);
    }
    Ok(())
}

#[test]
fn current_node_pointer_click_is_rejected_before_writing_to_a_foreign_session_transport()
-> Result<(), Box<dyn Error>> {
    let (registry, handle, remote) = current_node_fixture()?;
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<bool> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        stream.set_read_timeout(Some(Duration::from_millis(500)))?;
        let mut first_command_byte = [0_u8; 1];
        match stream.read(&mut first_command_byte) {
            Ok(0) => Ok(false),
            Ok(_) => Ok(true),
            Err(source)
                if matches!(
                    source.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                Ok(false)
            }
            Err(source) => Err(source),
        }
    });

    let endpoint = format!("ws://{local_addr}/session/{FOREIGN_TRANSPORT_SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(FOREIGN_TRANSPORT_SESSION_ID)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let established = WebDriverBiDiWebSocketHandshakePlan::new(
        connection,
        WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?,
    )?
    .write_opening_request(Duration::from_millis(500))?
    .read_opening_response(Duration::from_millis(500))?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();

    let send_result = send_webdriver_bidi_pointer_click(
        protocol_proof(BrowserProtocolCapability::TypedInput)?,
        42,
        "context-a",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    );
    let command_byte_seen = server
        .join()
        .map_err(|_| io::Error::other("foreign-session pointer server panicked"))??;

    let error = send_result
        .err()
        .ok_or_else(|| io::Error::other("registry session A unexpectedly dispatched pointer input on transport session B"))?;
    assert!(matches!(
        error,
        WebDriverBiDiPointerClickSendError::Authority {
            source: WebDriverBiDiPointerClickAuthorityError::BrowserAuthority(_)
        }
    ));
    assert_eq!(
        correlation.outstanding_count(),
        0,
        "foreign-session rejection must happen before correlation registration"
    );
    assert!(
        !command_byte_seen,
        "foreign-session rejection must happen before any pointer command-frame byte"
    );
    Ok(())
}
