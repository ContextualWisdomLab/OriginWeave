use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    MAX_WEBDRIVER_BIDI_JS_UINT, MAX_WEBSOCKET_FRAME_TIMEOUT, WebDriverBiDiCommandCorrelation,
    WebDriverBiDiCommandKind, WebDriverBiDiSessionStatusCommand,
    WebDriverBiDiSessionStatusCommandError, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
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

fn read_masked_text_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x81 || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected one final masked client text frame",
        ));
    }
    let length = usize::from(header[1] & 0x7f);
    if length > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "test frame unexpectedly required extended framing",
        ));
    }
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
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
fn session_status_rejects_ids_above_the_webdriver_bidi_js_uint_range() {
    let rejected = WebDriverBiDiSessionStatusCommand::new(MAX_WEBDRIVER_BIDI_JS_UINT + 1);
    assert_eq!(
        rejected.err().map(|error| error.to_string()).as_deref(),
        Some("WebDriver BiDi session.status command id is outside the js-uint range")
    );
}

#[test]
fn session_status_rejects_duplicate_correlation_before_any_frame_write()
-> Result<(), Box<dyn Error>> {
    let (established, server) = establish_with_handshake_only_server()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(7, WebDriverBiDiCommandKind::SessionStatus)?;
    let command = WebDriverBiDiSessionStatusCommand::new(7)?;

    let error = command
        .send(
            established,
            &mut correlation,
            WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
            Duration::from_millis(500),
        )
        .err()
        .ok_or_else(|| io::Error::other("duplicate correlation unexpectedly sent a command"))?;
    assert!(matches!(
        error,
        WebDriverBiDiSessionStatusCommandError::Correlation { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("duplicate-correlation test server panicked"))??;
    Ok(())
}

#[test]
fn session_status_rejects_invalid_frame_timeout_before_correlation_registration()
-> Result<(), Box<dyn Error>> {
    for (command_id, frame_timeout) in [
        (11, Duration::ZERO),
        (12, MAX_WEBSOCKET_FRAME_TIMEOUT + Duration::from_millis(1)),
    ] {
        let (established, server) = establish_with_handshake_only_server()?;
        let mut correlation = WebDriverBiDiCommandCorrelation::new();
        let command = WebDriverBiDiSessionStatusCommand::new(command_id)?;

        let error = command
            .send(
                established,
                &mut correlation,
                WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
                frame_timeout,
            )
            .err()
            .ok_or_else(|| io::Error::other("invalid frame timeout unexpectedly sent a command"))?;
        assert!(matches!(
            error,
            WebDriverBiDiSessionStatusCommandError::FrameWrite { .. }
        ));
        assert_eq!(correlation.outstanding_count(), 0);

        server
            .join()
            .map_err(|_| io::Error::other("invalid-timeout test server panicked"))??;
    }
    Ok(())
}

#[test]
fn session_status_rejects_reused_mask_key_before_correlation_registration()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let seed = read_masked_text_frame(&mut stream)?;
        if seed != b"{}" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected seed frame before reused-key regression",
            ));
        }
        Ok(())
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
    let repeated_key = WebDriverBiDiWebSocketMaskKey::new([9, 10, 11, 12]);
    let established = established.write_text_frame("{}", repeated_key, Duration::from_millis(500))?;

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let command = WebDriverBiDiSessionStatusCommand::new(13)?;
    let error = command
        .send(
            established,
            &mut correlation,
            repeated_key,
            Duration::from_millis(500),
        )
        .err()
        .ok_or_else(|| io::Error::other("reused masking key unexpectedly sent session.status"))?;
    assert!(matches!(
        error,
        WebDriverBiDiSessionStatusCommandError::FrameWrite { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("reused-mask-key test server panicked"))??;
    Ok(())
}
