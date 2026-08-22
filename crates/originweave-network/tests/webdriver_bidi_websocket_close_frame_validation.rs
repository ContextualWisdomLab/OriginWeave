use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketHandshakePlan,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";

fn exchange_server_close_frame(frame: &[u8]) -> Result<WebDriverBiDiWebSocketFrameError, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let frame = frame.to_vec();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        let mut opening = Vec::new();
        let mut byte = [0_u8; 1];
        while !opening.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte)?;
            opening.push(byte[0]);
        }
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        stream.write_all(&frame)
    });

    let endpoint = WebDriverBiDiWebSocketEndpoint::new(&format!(
        "ws://{local_addr}/session/{SESSION_ID}"
    ))?;
    let correlated = endpoint.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let connection = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?
        .connect()?;
    let client_key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let handshake = WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key)?;
    let opening = handshake.write_opening_request(Duration::from_millis(500))?;
    let established = opening.read_opening_response(Duration::from_millis(500))?;
    let result = established.read_frame(Duration::from_millis(500));

    let server_result = server
        .join()
        .map_err(|_| io::Error::other("close-frame validation test server panicked"))?;
    server_result?;

    result
        .err()
        .ok_or_else(|| io::Error::other("invalid RFC 6455 Close frame was admitted").into())
}

#[test]
fn close_frame_rejects_one_byte_body_and_invalid_utf8_reason() -> Result<(), Box<dyn Error>> {
    let one_byte_body = exchange_server_close_frame(&[0x88, 0x01, 0x00])?;
    assert!(matches!(
        one_byte_body,
        WebDriverBiDiWebSocketFrameError::MalformedFrame { .. }
    ));

    let invalid_utf8_reason =
        exchange_server_close_frame(&[0x88, 0x04, 0x03, 0xe8, 0xff, 0xff])?;
    assert!(matches!(
        invalid_utf8_reason,
        WebDriverBiDiWebSocketFrameError::MalformedFrame { .. }
    ));
    Ok(())
}
