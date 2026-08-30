use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    WebDriverBiDiNavigationCommittedObservation, WebDriverBiDiNavigationCommittedObservationError,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketTextMessage,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const EXPECTED_URL: &str = "https://example.test/after";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";

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

fn write_unmasked_text_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    stream.write_all(&[0x81])?;
    match payload.len() {
        0..=125 => stream.write_all(&[payload.len() as u8])?,
        126..=65_535 => {
            stream.write_all(&[126])?;
            stream.write_all(&(payload.len() as u16).to_be_bytes())?;
        }
        _ => {
            stream.write_all(&[127])?;
            stream.write_all(&(payload.len() as u64).to_be_bytes())?;
        }
    }
    stream.write_all(payload)
}

fn receive_navigation_event(payload: &[u8]) -> Result<WebDriverBiDiWebSocketTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let payload = payload.to_vec();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_unmasked_text_frame(&mut stream, &payload)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(SESSION_ID)?
        .into_explicit_connect_target()?;
    let connection = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
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
                "navigation event produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    server
        .join()
        .map_err(|_| io::Error::other("navigation unicode test server panicked"))??;
    Ok(text)
}

#[test]
fn navigation_committed_unicode_scalar_failures_remain_fail_closed_after_real_transport()
-> Result<(), Box<dyn Error>> {
    let cases: &[&[u8]] = &[
        br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"context":"\uD800","navigation":null,"timestamp":1,"url":"https://example.test/after"}}"#,
        br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"context":"\uDC00","navigation":null,"timestamp":1,"url":"https://example.test/after"}}"#,
        br#"{"type":"event","method":"browsingContext.navigationCommitted","params":{"context":"\uD800\u0041","navigation":null,"timestamp":1,"url":"https://example.test/after"}}"#,
    ];

    for payload in cases {
        let event = receive_navigation_event(payload)?;
        let mut registry = BrowserAuthorityRegistry::new();
        let session = registry.register_session(SESSION_ID)?;
        let context = registry.register_context(session, "context-a")?;
        let result = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
            &event,
            &registry,
            session,
            context,
            EXPECTED_URL,
        );
        assert!(matches!(
            result,
            Err(WebDriverBiDiNavigationCommittedObservationError::Envelope { .. })
                | Err(WebDriverBiDiNavigationCommittedObservationError::Projection { .. })
        ));
    }
    Ok(())
}
