use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError,
    WebDriverBiDiJsonEnvelopeKind, WebDriverBiDiSessionStatusResponseError,
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

fn read_text_over_loopback(
    document: &'static [u8],
) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    if document.len() > 125 {
        return Err(io::Error::other("unit JSON document exceeded one-byte frame length").into());
    }

    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        stream.write_all(&[0x81, document.len() as u8])?;
        stream.write_all(document)
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
    correlation.register_command(7)?;

    let parsed = WebDriverBiDiSessionStatusResult::parse_and_correlate(&text, &mut correlation);
    assert!(matches!(
        parsed,
        Err(WebDriverBiDiSessionStatusResponseError::MissingReady)
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}
