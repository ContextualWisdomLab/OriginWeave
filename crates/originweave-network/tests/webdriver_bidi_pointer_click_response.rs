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
    WebDriverBiDiPointerClickCommand, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiPointerClickResponseError,
    WebDriverBiDiPointerClickResult, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketTextMessage,
    send_webdriver_bidi_pointer_click,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const CLICK_SUCCESS_RESPONSE: &[u8] =
    br#"{"type":"success","id":42,"result":{"vendorExtension":{"observed":false}}}"#;
const CLICK_REMOTE_ERROR_RESPONSE: &[u8] =
    br#"{"type":"error","id":42,"error":"element click intercepted","message":"blocked"}"#;
const CLICK_UNKNOWN_ID_RESPONSE: &[u8] =
    br#"{"type":"success","id":43,"result":{"vendorExtension":true}}"#;
const CLICK_MALFORMED_RESPONSE: &[u8] = br#"{"type":"success","id":42}"#;
const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

type AdmittedPointerClickFixture = (
    BrowserAuthorityRegistry,
    AdmittedNodeHandle,
    WebDriverBiDiRemoteNodeReference,
);

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

fn typed_input_proof() -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::TypedInput],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::TypedInput,
    )?)
}

fn admitted_pointer_click_fixture() -> Result<AdmittedPointerClickFixture, Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session = registry.register_session(SESSION_ID)?;
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
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 1)?;
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
            let length = usize::from(u16::from_be_bytes(extended));
            if length <= 125 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "client text frame used non-minimal 16-bit length encoding",
                ));
            }
            length
        }
        127 => {
            let mut extended = [0_u8; 8];
            stream.read_exact(&mut extended)?;
            let length = u64::from_be_bytes(extended);
            if length <= u64::from(u16::MAX) || length > usize::MAX as u64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "client text frame used invalid 64-bit length encoding",
                ));
            }
            length as usize
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

fn send_click_and_read_response(
    response: &'static [u8],
) -> Result<
    (
        WebDriverBiDiWebSocketTextMessage,
        WebDriverBiDiCommandCorrelation,
    ),
    Box<dyn Error>,
> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let (registry, handle, remote) = admitted_pointer_click_fixture()?;
    let expected_json = WebDriverBiDiPointerClickCommand::new_for_current_node(
        42,
        "context-a",
        &handle,
        &remote,
        &registry,
    )?
    .as_json()
    .as_bytes()
    .to_vec();

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
        stream.write_all(&[0x81, response.len() as u8])?;
        stream.write_all(response)
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

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let established = send_webdriver_bidi_pointer_click(
        typed_input_proof()?,
        42,
        "context-a",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;

    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "pointer-click response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };

    server
        .join()
        .map_err(|_| io::Error::other("pointer-click response test server panicked"))??;
    Ok((text, correlation))
}

#[test]
fn pointer_click_success_accepts_extensible_empty_result_and_consumes_exact_correlation()
-> Result<(), Box<dyn Error>> {
    let (text, mut correlation) = send_click_and_read_response(CLICK_SUCCESS_RESPONSE)?;
    assert_eq!(correlation.outstanding_count(), 1);

    let result = WebDriverBiDiPointerClickResult::parse_and_correlate(&text, &mut correlation)?;
    assert_eq!(result.command_id(), 42);
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn pointer_click_remote_error_consumes_only_the_correlated_command() -> Result<(), Box<dyn Error>> {
    let (text, mut correlation) = send_click_and_read_response(CLICK_REMOTE_ERROR_RESPONSE)?;
    let parsed = WebDriverBiDiPointerClickResult::parse_and_correlate(&text, &mut correlation);
    let error = match parsed {
        Ok(_) => {
            return Err(
                io::Error::other("remote error was accepted as pointer-click success").into(),
            );
        }
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WebDriverBiDiPointerClickResponseError::RemoteProtocolError { command_id: 42 }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi pointer-click returned a protocol error"
    );
    assert!(error.source().is_none());
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn malformed_pointer_click_envelope_fails_before_consuming_correlation()
-> Result<(), Box<dyn Error>> {
    let (text, mut correlation) = send_click_and_read_response(CLICK_MALFORMED_RESPONSE)?;
    let parsed = WebDriverBiDiPointerClickResult::parse_and_correlate(&text, &mut correlation);
    let error = match parsed {
        Ok(_) => {
            return Err(io::Error::other("malformed pointer-click response was accepted").into());
        }
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WebDriverBiDiPointerClickResponseError::Envelope { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi pointer-click envelope is invalid"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn unknown_pointer_click_response_id_does_not_consume_the_outstanding_command()
-> Result<(), Box<dyn Error>> {
    let (text, mut correlation) = send_click_and_read_response(CLICK_UNKNOWN_ID_RESPONSE)?;
    let parsed = WebDriverBiDiPointerClickResult::parse_and_correlate(&text, &mut correlation);
    let error = match parsed {
        Ok(_) => {
            return Err(io::Error::other("unknown pointer-click response id was accepted").into());
        }
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WebDriverBiDiPointerClickResponseError::Correlation { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi pointer-click response correlation failed"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}
