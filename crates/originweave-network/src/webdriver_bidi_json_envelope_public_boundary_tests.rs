use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, WebDriverBiDiWebSocketEndpoint};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandKind, WebDriverBiDiJsonEnvelope,
    WebDriverBiDiJsonEnvelopeError,
    WebDriverBiDiJsonEnvelopeKind, WebDriverBiDiNavigationCommittedObservation,
    WebDriverBiDiNavigationCommittedObservationError, WebDriverBiDiSessionStatusResponseError,
    WebDriverBiDiSessionStatusResult, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTextMessage,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const SUCCESS_MESSAGE: &[u8] =
    br#"{"type":"success","id":7,"result":{"ready":true,"slash":"\/","upper":"\uABCD"}}"#;
const EMPTY_STATUS_RESULT: &[u8] = br#"{"type":"success","id":7,"result":{}}"#;
const NAVIGATION_COMMITTED_EVENT: &[u8] = br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"context":"a","navigation":null,"timestamp":0,"url":"x"}}"#;
const NAVIGATION_COMMITTED_UTF8_EVENT: &str = r#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"context":"a","navigation":null,"timestamp":1,"url":"https://example.test/café"}}"#;
const NAVIGATION_COMMITTED_MISSING_CONTEXT: &[u8] = br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"navigation":null,"timestamp":0,"url":"x"}}"#;
const MALFORMED_NAVIGATION_COMMITTED_EVENT: &[u8] =
    br#"{"type":"event","method":"browsingContext.navigationCommitted","params":"#;
const OTHER_EVENT: &[u8] = br#"{"type":"event","method":"x","params":{}}"#;

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

fn write_unmasked_text_frame(stream: &mut TcpStream, document: &[u8]) -> io::Result<()> {
    if document.len() <= 125 {
        let length = u8::try_from(document.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "short frame length exceeds u8")
        })?;
        stream.write_all(&[0x81, length])?;
    } else {
        let length = u16::try_from(document.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "unit JSON document exceeds two-byte frame length",
            )
        })?;
        stream.write_all(&[0x81, 126])?;
        stream.write_all(&length.to_be_bytes())?;
    }
    stream.write_all(document)
}

fn read_text_over_loopback(
    document: &'static [u8],
) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_unmasked_text_frame(&mut stream, document)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let admitted = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let established = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?
        .write_opening_request(Duration::from_millis(500))?
        .read_opening_response(Duration::from_millis(500))?;
    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;

    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "validated text frame produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };

    server
        .join()
        .map_err(|_| io::Error::other("JSON-envelope unit server panicked"))??;
    Ok(text)
}

fn parse_over_loopback(
    document: &'static [u8],
) -> Result<Result<WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError>, Box<dyn Error>> {
    let text = read_text_over_loopback(document)?;
    Ok(WebDriverBiDiJsonEnvelope::parse(&text))
}

#[test]
fn public_json_envelope_boundary_is_exercised_from_unit_build() -> Result<(), Box<dyn Error>> {
    let parsed = parse_over_loopback(SUCCESS_MESSAGE)?;
    assert_eq!(
        parsed.as_ref().map(WebDriverBiDiJsonEnvelope::kind),
        Ok(WebDriverBiDiJsonEnvelopeKind::Success)
    );
    assert_eq!(
        parsed.as_ref().map(WebDriverBiDiJsonEnvelope::command_id),
        Ok(Some(7))
    );
    Ok(())
}

#[test]
fn public_json_envelope_unit_build_covers_fail_closed_json_edges() -> Result<(), Box<dyn Error>> {
    let malformed_documents: [&'static [u8]; 8] = [
        br#"{"unterminated"#,
        br#"{"type" "success"}"#,
        br#"{"type":"success" "id":1}"#,
        br#"{"type":"success","id":1,"result":{"a" 1}}"#,
        br#"{"type":"success","id":1,"result":[1 2]}"#,
        br##"{"type":"success","id":1,"result":{"bad":"\"##,
        br#"{"type":"success","id":1,"result":{"bad":"\ud800\0041"}}"#,
        br##"{"type":"success","id":1,"result":{"bad":"\ud800\u"##,
    ];

    for document in malformed_documents {
        assert_eq!(
            parse_over_loopback(document)?,
            Err(WebDriverBiDiJsonEnvelopeError::InvalidJson)
        );
    }
    Ok(())
}

#[test]
fn public_session_status_empty_result_fails_closed_from_unit_build() -> Result<(), Box<dyn Error>> {
    let text = read_text_over_loopback(EMPTY_STATUS_RESULT)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(7, WebDriverBiDiCommandKind::SessionStatus)?;

    let parsed = WebDriverBiDiSessionStatusResult::parse_and_correlate(&text, &mut correlation);
    assert!(matches!(
        parsed,
        Err(WebDriverBiDiSessionStatusResponseError::MissingReady)
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn public_navigation_committed_boundary_is_exercised_from_unit_build() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "a")?;

    let event = read_text_over_loopback(NAVIGATION_COMMITTED_EVENT)?;
    let observation = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &event, &registry, session, context, "x",
    )?;
    assert_eq!(observation.browser_session(), session);
    assert_eq!(observation.browsing_context(), context);
    assert_eq!(observation.navigation_id(), None);
    assert_eq!(observation.timestamp(), 0);
    assert_eq!(observation.url(), "x");
    let debug = format!("{observation:?}");
    assert!(debug.contains("WebDriverBiDiNavigationCommittedObservation"));
    assert!(debug.contains("has_navigation_id: false"));
    assert!(!debug.contains("url: \"x\""));

    let utf8_event = read_text_over_loopback(NAVIGATION_COMMITTED_UTF8_EVENT.as_bytes())?;
    let utf8_observation = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &utf8_event,
        &registry,
        session,
        context,
        "https://example.test/café",
    )?;
    assert_eq!(utf8_observation.timestamp(), 1);
    assert_eq!(utf8_observation.url(), "https://example.test/café");

    let wrong_url = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &event, &registry, session, context, "y",
    );
    let Err(wrong_url @ WebDriverBiDiNavigationCommittedObservationError::UnexpectedUrl) =
        wrong_url
    else {
        return Err(io::Error::other("wrong URL did not fail closed").into());
    };
    assert!(!wrong_url.to_string().is_empty());
    assert!(wrong_url.source().is_none());

    let other_context = registry.register_context(session, "b")?;
    let wrong_context = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &event,
        &registry,
        session,
        other_context,
        "x",
    );
    let Err(
        wrong_context @ WebDriverBiDiNavigationCommittedObservationError::ContextBinding { .. },
    ) = wrong_context
    else {
        return Err(io::Error::other("wrong registered context did not fail closed").into());
    };
    assert!(!wrong_context.to_string().is_empty());
    assert!(wrong_context.source().is_some());

    let missing_context = read_text_over_loopback(NAVIGATION_COMMITTED_MISSING_CONTEXT)?;
    let projection = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &missing_context,
        &registry,
        session,
        context,
        "x",
    );
    let Err(projection @ WebDriverBiDiNavigationCommittedObservationError::Projection { .. }) =
        projection
    else {
        return Err(io::Error::other("missing context did not fail at projection").into());
    };
    assert!(!projection.to_string().is_empty());
    assert!(projection.source().is_some());

    let malformed = read_text_over_loopback(MALFORMED_NAVIGATION_COMMITTED_EVENT)?;
    let envelope = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &malformed, &registry, session, context, "x",
    );
    let Err(envelope @ WebDriverBiDiNavigationCommittedObservationError::Envelope { .. }) =
        envelope
    else {
        return Err(io::Error::other("malformed event did not fail at envelope validation").into());
    };
    assert!(!envelope.to_string().is_empty());
    assert!(envelope.source().is_some());

    let other_event = read_text_over_loopback(OTHER_EVENT)?;
    let unexpected = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
        &other_event,
        &registry,
        session,
        context,
        "x",
    );
    let Err(unexpected @ WebDriverBiDiNavigationCommittedObservationError::UnexpectedEvent) =
        unexpected
    else {
        return Err(io::Error::other("different event method was not rejected").into());
    };
    assert!(!unexpected.to_string().is_empty());
    assert!(unexpected.source().is_none());

    let non_event = read_text_over_loopback(SUCCESS_MESSAGE)?;
    assert!(matches!(
        WebDriverBiDiNavigationCommittedObservation::parse_and_match(
            &non_event, &registry, session, context, "x",
        ),
        Err(WebDriverBiDiNavigationCommittedObservationError::UnexpectedEvent)
    ));
    Ok(())
}
