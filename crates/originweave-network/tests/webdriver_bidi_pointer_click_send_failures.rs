use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    WebDriverBiDiPointerClickCommand, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCommandKind, WebDriverBiDiPointerClickSendError, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    send_webdriver_bidi_pointer_click,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
type HandshakeOnlyServer = (
    WebDriverBiDiWebSocketEstablished,
    thread::JoinHandle<io::Result<()>>,
);

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

fn pointer_click(command_id: u64) -> Result<WebDriverBiDiPointerClickCommand, Box<dyn Error>> {
    let node = WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?;
    Ok(WebDriverBiDiPointerClickCommand::new(
        command_id,
        "context-a",
        &node,
    )?)
}

#[test]
fn pointer_click_rejects_duplicate_correlation_before_frame_write() -> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(7, WebDriverBiDiCommandKind::PointerClick)?;
    let command = pointer_click(7)?;

    let error = send_webdriver_bidi_pointer_click(
        &command,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("duplicate correlation unexpectedly sent a pointer click"))?;
    assert!(matches!(
        error,
        WebDriverBiDiPointerClickSendError::Correlation { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi pointer-click command correlation was rejected"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("duplicate-correlation pointer server panicked"))??;
    Ok(())
}

#[test]
fn pointer_click_preserves_typed_registration_when_frame_timeout_is_invalid()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let command = pointer_click(11)?;

    let error = send_webdriver_bidi_pointer_click(
        &command,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
        Duration::ZERO,
    )
    .err()
    .ok_or_else(|| io::Error::other("zero frame timeout unexpectedly sent a pointer click"))?;
    assert!(matches!(
        error,
        WebDriverBiDiPointerClickSendError::FrameWrite { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi pointer-click command frame write failed"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    let kind_error = correlation
        .retire_command_for(11, WebDriverBiDiCommandKind::SessionEnd)
        .expect_err("pointer-click correlation must not retire as session.end");
    assert_eq!(
        kind_error,
        WebDriverBiDiCommandCorrelationError::CommandKindMismatch {
            expected: WebDriverBiDiCommandKind::SessionEnd,
            actual: WebDriverBiDiCommandKind::PointerClick,
        }
    );
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("invalid-timeout pointer server panicked"))??;
    Ok(())
}
