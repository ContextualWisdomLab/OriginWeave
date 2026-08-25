use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserSessionId, BrowsingContextId, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesAdmissionError, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiLocateNodesResponseDocumentError, WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiLocateNodesExchangeError, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const RESPONSE_DOCUMENT: &str =
    r#"{"type":"success","id":7,"result":{"nodes":[{"type":"node","sharedId":"shared-1"}]}}"#;
const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

type ServerHandle = thread::JoinHandle<io::Result<Vec<u8>>>;
type EstablishedFixture =
    Result<(SocketAddr, WebDriverBiDiWebSocketEstablished, ServerHandle), Box<dyn Error>>;

fn connect(
    endpoint: &str,
) -> Result<originweave_network::WebDriverBiDiTcpConnection, Box<dyn Error>> {
    let admitted = WebDriverBiDiWebSocketEndpoint::new(endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?;
    Ok(plan.connect()?)
}

fn read_opening_request(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    let mut buffer = [0_u8; 512];
    while !request.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..count]);
    }
    Ok(request)
}

fn read_client_text_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x81 || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected one masked final client text frame",
        ));
    }
    let payload_length = match header[1] & 0x7f {
        value @ 0..=125 => usize::from(value),
        126 => {
            let mut extended = [0_u8; 2];
            stream.read_exact(&mut extended)?;
            usize::from(u16::from_be_bytes(extended))
        }
        127 => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "test fixture rejects 64-bit client frame lengths",
            ));
        }
        _ => unreachable!("7-bit WebSocket payload marker"),
    };
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
}

fn establish() -> EstablishedFixture {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<Vec<u8>> {
        let (mut stream, _) = listener.accept()?;
        let request = read_opening_request(&mut stream)?;
        if !request.ends_with(b"\r\n\r\n") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "client opening request was incomplete",
            ));
        }
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        let command = read_client_text_frame(&mut stream)?;
        let response = RESPONSE_DOCUMENT.as_bytes();
        let response_length = u8::try_from(response.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "test response exceeds short frame",
            )
        })?;
        if response_length > 125 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "test response exceeds short frame",
            ));
        }
        stream.write_all(&[0x81, response_length])?;
        stream.write_all(response)?;
        Ok(command)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    Ok((local_addr, established, server))
}

fn locate_nodes_command() -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Checkout"), 2)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        7,
        "top-level-context",
        &query,
    )?)
}

fn controlled_origin() -> Result<Origin, Box<dyn Error>> {
    Origin::parse("https://app.example").map_err(|_error| "valid controlled fixture origin".into())
}

fn current_target<'a>(
    registry: &mut BrowserAuthorityRegistry,
    origin: &'a Origin,
) -> Result<
    (
        BrowserContextOriginEpochDispatchTarget<'a>,
        BrowserSessionId,
        BrowsingContextId,
    ),
    Box<dyn Error>,
> {
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "top-level-context")?;
    let epoch = registry.bind_context_origin(session, context, origin)?;
    Ok((
        BrowserContextOriginEpochDispatchTarget::new(
            BrowserContextOriginDispatchTarget::new(
                BrowserContextDispatchTarget::new(session, context),
                origin,
            ),
            epoch,
        ),
        session,
        context,
    ))
}

fn semantic_observation_proof() -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::SemanticObservation],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::SemanticObservation,
    )?)
}

#[test]
fn live_websocket_locate_nodes_exchange_binds_wire_nodes_to_current_authority()
-> Result<(), Box<dyn Error>> {
    let (local_addr, established, server) = establish()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let (target, _session, _context) = current_target(&mut registry, &origin)?;
    let expected_epoch = target.expected_epoch();

    let (established, handles) = established.exchange_locate_nodes_and_bind_current_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        Duration::from_millis(500),
        (semantic_observation_proof()?, target),
        &mut registry,
    )?;

    assert_eq!(handles.len(), 1);
    assert_eq!(handles[0].origin(), &origin);
    assert_eq!(handles[0].document_epoch(), expected_epoch);
    assert_eq!(
        established
            .transport_evidence()
            .verified_peer()
            .socket_addr(),
        local_addr
    );
    let command = server.join().map_err(|_| "test server panicked")??;
    assert_eq!(command, locate_nodes_command()?.as_json().as_bytes());
    Ok(())
}

#[test]
fn live_exchange_fails_closed_when_document_epoch_changed_before_wire_node_binding()
-> Result<(), Box<dyn Error>> {
    let (_local_addr, established, server) = establish()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let (target, session, context) = current_target(&mut registry, &origin)?;
    let stale_epoch = target.expected_epoch();
    let current_epoch = registry.advance_document(context)?;
    registry.bind_context_origin(session, context, &origin)?;

    let error = established.exchange_locate_nodes_and_bind_current_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        Duration::from_millis(500),
        (semantic_observation_proof()?, target),
        &mut registry,
    );

    assert!(matches!(
        error,
        Err(WebDriverBiDiLocateNodesExchangeError::LocateNodesResponse(
            WebDriverBiDiLocateNodesResponseDocumentError::NodeBinding(
                WebDriverBiDiLocateNodesAdmissionError::DocumentEpochMismatch {
                    expected,
                    current,
                },
            ),
        )) if expected == stale_epoch && current == current_epoch
    ));
    let command = server.join().map_err(|_| "test server panicked")??;
    assert_eq!(command, locate_nodes_command()?.as_json().as_bytes());
    Ok(())
}
