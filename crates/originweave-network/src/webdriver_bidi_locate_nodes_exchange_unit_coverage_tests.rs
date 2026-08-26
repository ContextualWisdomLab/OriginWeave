use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, Origin, OriginWeaveProtocolVersion,
    ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiWebSocketEndpoint,
};

use crate::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
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

fn read_client_text_frame(stream: &mut TcpStream) -> io::Result<()> {
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
    Ok(())
}

fn write_opening_response(stream: &mut TcpStream) -> io::Result<()> {
    stream.write_all(
        b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
    )
}

fn write_short_server_frame(
    stream: &mut TcpStream,
    first_byte: u8,
    payload: &[u8],
) -> io::Result<()> {
    let payload_length = u8::try_from(payload.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "unit coverage server frame exceeds short-frame limit",
        )
    })?;
    if payload_length > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unit coverage server frame exceeds short-frame limit",
        ));
    }
    stream.write_all(&[first_byte, payload_length])?;
    stream.write_all(payload)
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

fn establish_client(
    local_addr: std::net::SocketAddr,
) -> Result<crate::WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let admitted = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    Ok(written.read_opening_response(Duration::from_millis(500))?)
}

#[test]
fn binding_wrapper_success_path_executes_in_library_unit_crate() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        write_opening_response(&mut stream)?;
        read_client_text_frame(&mut stream)?;
        write_short_server_frame(&mut stream, 0x81, RESPONSE_DOCUMENT.as_bytes())
    });

    let established = establish_client(local_addr)?;

    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "top-level-context")?;
    let epoch = registry.bind_context_origin(session, context, &origin)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &origin,
        ),
        epoch,
    );

    let (_established, handles) = established.exchange_locate_nodes_and_bind_current_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        Duration::from_millis(500),
        (semantic_observation_proof()?, target),
        &mut registry,
    )?;
    assert_eq!(handles.len(), 1);
    assert_eq!(handles[0].origin(), &origin);

    server
        .join()
        .map_err(|_| io::Error::other("unit coverage test server panicked"))??;
    Ok(())
}

#[test]
fn fragmented_response_executes_nonfinal_text_arm_in_library_unit_crate()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        write_opening_response(&mut stream)?;
        read_client_text_frame(&mut stream)?;
        let response = RESPONSE_DOCUMENT.as_bytes();
        write_short_server_frame(&mut stream, 0x01, &response[..1])?;
        write_short_server_frame(&mut stream, 0x80, &response[1..])
    });

    let established = establish_client(local_addr)?;
    let (_established, result) = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x21, 0x22, 0x23, 0x24]),
        &mut || None,
        Duration::from_millis(500),
    )?;
    assert_eq!(result.nodes().len(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("fragmented unit coverage test server panicked"))??;
    Ok(())
}

#[test]
fn second_text_frame_during_fragmentation_executes_guard_denial_in_library_unit_crate()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        write_opening_response(&mut stream)?;
        read_client_text_frame(&mut stream)?;
        write_short_server_frame(&mut stream, 0x01, b"{")?;
        write_short_server_frame(&mut stream, 0x81, b"x")
    });

    let established = establish_client(local_addr)?;
    let exchange = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x31, 0x32, 0x33, 0x34]),
        &mut || None,
        Duration::from_millis(500),
    );
    assert_eq!(
        format!("{exchange:?}"),
        "Err(UnexpectedResponseFrame { fin: true, opcode: 1 })"
    );

    server
        .join()
        .map_err(|_| io::Error::other("second-text unit coverage test server panicked"))??;
    Ok(())
}
