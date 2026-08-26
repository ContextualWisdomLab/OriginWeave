use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const REUSED_MASK_REASON: &str =
    "client masking key was reused for consecutive frames on this established WebSocket";
const MAX_PONG_PAYLOAD_BYTES: usize = 125;

fn connect(
    endpoint: &str,
) -> Result<originweave_network::WebDriverBiDiTcpConnection, Box<dyn Error>> {
    let admitted = WebDriverBiDiWebSocketEndpoint::new(endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?;
    Ok(plan.connect()?)
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

fn write_opening_response(stream: &mut TcpStream) -> io::Result<()> {
    stream.write_all(
        b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
    )
}

fn read_masked_pong(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x8a || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client did not send one final masked Pong frame",
        ));
    }
    let payload_length = usize::from(header[1] & 0x7f);
    if payload_length > MAX_PONG_PAYLOAD_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Pong payload exceeded the RFC 6455 control-frame bound",
        ));
    }
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
}

fn require_peer_closed_without_another_frame(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut byte = [0_u8; 1];
    match stream.read(&mut byte) {
        Ok(0) => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client emitted frame bytes after a fail-closed Pong rejection",
        )),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("client did not close after a fail-closed Pong rejection: {error}"),
        )),
    }
}

#[test]
fn established_stream_writes_masked_pong_with_exact_ping_payload() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<Vec<u8>> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        write_opening_response(&mut stream)?;
        read_masked_pong(&mut stream)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    let pong_payload = b"peer-keepalive";
    let established = established.write_pong_frame(
        pong_payload,
        WebDriverBiDiWebSocketMaskKey::new([0x51, 0x52, 0x53, 0x54]),
        Duration::from_millis(500),
    )?;
    assert_eq!(
        established
            .transport_evidence()
            .verified_peer()
            .socket_addr(),
        local_addr
    );
    drop(established);

    let received = server
        .join()
        .map_err(|_| io::Error::other("WebSocket Pong test server panicked"))??;
    assert_eq!(received, pong_payload);
    Ok(())
}

#[test]
fn established_stream_rejects_reused_pong_mask_before_second_wire_write()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<Vec<u8>> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        write_opening_response(&mut stream)?;
        let first_payload = read_masked_pong(&mut stream)?;
        require_peer_closed_without_another_frame(&mut stream)?;
        Ok(first_payload)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    let reused_mask = WebDriverBiDiWebSocketMaskKey::new([0x61, 0x62, 0x63, 0x64]);
    let established =
        established.write_pong_frame(b"first-pong", reused_mask, Duration::from_millis(500))?;
    let error =
        match established.write_pong_frame(b"second-pong", reused_mask, Duration::from_millis(500))
        {
            Ok(_) => {
                return Err(io::Error::other(
                    "RFC 6455 Pong masking-key reuse unexpectedly succeeded",
                )
                .into());
            }
            Err(error) => error,
        };
    assert!(matches!(
        error,
        WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: REUSED_MASK_REASON
        }
    ));

    let received = server
        .join()
        .map_err(|_| io::Error::other("WebSocket Pong mask-reuse test server panicked"))??;
    assert_eq!(received, b"first-pong");
    Ok(())
}

#[test]
fn established_stream_rejects_oversized_pong_before_wire_write() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        write_opening_response(&mut stream)?;
        require_peer_closed_without_another_frame(&mut stream)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    let oversized = vec![0x41_u8; MAX_PONG_PAYLOAD_BYTES + 1];
    let error = match established.write_pong_frame(
        &oversized,
        WebDriverBiDiWebSocketMaskKey::new([0x71, 0x72, 0x73, 0x74]),
        Duration::from_millis(500),
    ) {
        Ok(_) => return Err(io::Error::other("oversized Pong unexpectedly succeeded").into()),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WebDriverBiDiWebSocketFrameError::FrameTooLarge {
            payload_bytes,
            maximum_bytes: MAX_PONG_PAYLOAD_BYTES,
        } if payload_bytes == oversized.len()
    ));

    server
        .join()
        .map_err(|_| io::Error::other("WebSocket oversized-Pong test server panicked"))??;
    Ok(())
}
