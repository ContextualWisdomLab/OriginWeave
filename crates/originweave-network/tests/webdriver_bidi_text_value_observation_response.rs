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
    WebDriverBiDiRemoteNodeReference, WebDriverBiDiTextValueObservationCommand,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandKind, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiTextValueObservationResponseError, WebDriverBiDiTextValueObservationResult,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketTextMessage,
    send_webdriver_bidi_text_value_observation,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const OBSERVATION_SUCCESS_RESPONSE: &[u8] = br#"{"type":"success","id":43,"result":{"type":"success","result":{"type":"string","value":"Quarterly review"},"realm":"realm-1"}}"#;
const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

type AdmittedTextFieldFixture = (
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

fn admitted_text_field_fixture() -> Result<AdmittedTextFieldFixture, Box<dyn Error>> {
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
                io::Error::new(io::ErrorKind::InvalidData, "frame length exceeds usize")
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
                "fixture response unexpectedly required 64-bit framing",
            ));
        }
    }
    stream.write_all(payload)
}

fn receive_server_text(
    payload: &[u8],
) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let response = payload.to_vec();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
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
    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let message = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "fixture produced unexpected message assembly state: {other:?}"
            ))
            .into());
        }
    };
    server
        .join()
        .map_err(|_| io::Error::other("response fixture server panicked"))??;
    Ok(message)
}

#[test]
fn observed_text_postcondition_consumes_exact_command_without_exposing_page_text()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let (registry, handle, remote) = admitted_text_field_fixture()?;
    let expected_json = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
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
                "unexpected script.callFunction text-value observation command",
            ));
        }
        write_text_frame(&mut stream, OBSERVATION_SUCCESS_RESPONSE)
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

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let established = send_webdriver_bidi_text_value_observation(
        semantic_observation_proof()?,
        43,
        "context-a",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;
    assert_eq!(correlation.outstanding_count(), 1);

    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "text-value observation response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    let result = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        &text,
        "Quarterly review",
        &mut correlation,
    )?;
    assert_eq!(result.command_id(), 43);
    assert!(result.matches_expected_text());
    assert_eq!(result.observed_text_bytes(), "Quarterly review".len());
    assert_eq!(correlation.outstanding_count(), 0);
    let debug = format!("{result:?}");
    assert!(!debug.contains("Quarterly review"));

    server
        .join()
        .map_err(|_| io::Error::other("text-value observation response server panicked"))??;
    Ok(())
}

#[test]
fn response_admission_failures_preserve_or_consume_exact_correlation_state()
-> Result<(), Box<dyn Error>> {
    let invalid = receive_server_text(b"not-json")?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(70, WebDriverBiDiCommandKind::TextValueObservation)?;
    let Err(envelope_error) = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        &invalid,
        "expected",
        &mut correlation,
    ) else {
        return Err(io::Error::other("invalid envelope unexpectedly admitted").into());
    };
    assert!(matches!(
        &envelope_error,
        WebDriverBiDiTextValueObservationResponseError::Envelope { .. }
    ));
    assert_eq!(
        envelope_error.to_string(),
        "WebDriver BiDi text-value observation envelope is invalid"
    );
    assert!(envelope_error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    let event = receive_server_text(br#"{"type":"event","method":"log.entryAdded","params":{}}"#)?;
    assert!(matches!(
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &event,
            "expected",
            &mut correlation,
        ),
        Err(WebDriverBiDiTextValueObservationResponseError::UnexpectedEvent)
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    correlation.register_command_for(71, WebDriverBiDiCommandKind::TextValueObservation)?;
    let protocol_error = receive_server_text(
        br#"{"type":"error","id":71,"error":"unknown error","message":"remote failure"}"#,
    )?;
    assert!(matches!(
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &protocol_error,
            "expected",
            &mut correlation,
        ),
        Err(WebDriverBiDiTextValueObservationResponseError::RemoteProtocolError { command_id: 71 })
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let unknown_success = receive_server_text(
        br#"{"type":"success","id":72,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"expected"}}}"#,
    )?;
    let Err(correlation_error) =
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &unknown_success,
            "expected",
            &mut correlation,
        )
    else {
        return Err(io::Error::other("unknown response id unexpectedly correlated").into());
    };
    assert!(matches!(
        &correlation_error,
        WebDriverBiDiTextValueObservationResponseError::Correlation { .. }
    ));
    assert_eq!(
        correlation_error.to_string(),
        "WebDriver BiDi text-value observation response correlation failed"
    );
    assert!(correlation_error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    correlation.register_command_for(73, WebDriverBiDiCommandKind::TextValueObservation)?;
    let malformed_projection = receive_server_text(
        br#"{"type":"success","id":73,"result":{"type":"success","result":{"type":"string","value":"expected"}}}"#,
    )?;
    let Err(projection_error) =
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &malformed_projection,
            "expected",
            &mut correlation,
        )
    else {
        return Err(io::Error::other("malformed script result unexpectedly admitted").into());
    };
    assert!(matches!(
        &projection_error,
        WebDriverBiDiTextValueObservationResponseError::Projection { .. }
    ));
    assert_eq!(
        projection_error.to_string(),
        "WebDriver BiDi text-value observation result is invalid"
    );
    assert!(projection_error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 2);

    let script_exception = receive_server_text(
        br#"{"type":"success","id":73,"result":{"type":"exception","realm":"realm-1","exceptionDetails":{}}}"#,
    )?;
    assert!(matches!(
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &script_exception,
            "expected",
            &mut correlation,
        ),
        Err(WebDriverBiDiTextValueObservationResponseError::ScriptException { command_id: 73 })
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let valid_success = receive_server_text(
        br#"{"type":"success","id":70,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"expected"}}}"#,
    )?;
    assert!(matches!(
        WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
            &valid_success,
            "bad\ttext",
            &mut correlation,
        ),
        Err(WebDriverBiDiTextValueObservationResponseError::InvalidExpectedText)
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let result = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        &valid_success,
        "expected",
        &mut correlation,
    )?;
    assert!(result.matches_expected_text());
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}
