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
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";

fn connect(
    endpoint: &str,
) -> Result<originweave_network::WebDriverBiDiTcpConnection, Box<dyn Error>> {
    let admitted = WebDriverBiDiWebSocketEndpoint::new(endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?;
    Ok(plan.connect()?)
}

fn establish(
    endpoint: &str,
) -> Result<originweave_network::WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    Ok(written.read_opening_response(Duration::from_millis(500))?)
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

fn read_masked_frame(stream: &mut TcpStream, expected_opcode: u8) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x80 | expected_opcode || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client did not send the expected final masked frame",
        ));
    }
    let payload_length = usize::from(header[1] & 0x7f);
    if payload_length > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "test payload unexpectedly used an extended length",
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

fn read_masked_text(stream: &mut TcpStream) -> io::Result<String> {
    let payload = read_masked_frame(stream, 0x1)?;
    String::from_utf8(payload).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn require_peer_closed_before_second_frame(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut byte = [0_u8; 1];
    match stream.read(&mut byte) {
        Ok(0) => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client emitted a second frame after reusing its masking key",
        )),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("client did not close after refusing a reused masking key: {error}"),
        )),
    }
}

fn require_peer_closed_without_frame(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut byte = [0_u8; 1];
    match stream.read(&mut byte) {
        Ok(0) => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client emitted a frame after rejecting the operation",
        )),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("client did not close after rejecting the operation: {error}"),
        )),
    }
}

#[test]
fn established_stream_rejects_client_mask_reuse_across_sequential_frames()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<String> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let first = read_masked_text(&mut stream)?;
        require_peer_closed_before_second_frame(&mut stream)?;
        Ok(first)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let established = establish(&endpoint)?;
    let reused_mask = WebDriverBiDiWebSocketMaskKey::new([0x21, 0x22, 0x23, 0x24]);
    let established =
        established.write_text_frame("first-frame", reused_mask, Duration::from_millis(500))?;
    let error =
        match established.write_text_frame("second-frame", reused_mask, Duration::from_millis(500))
        {
            Ok(_) => {
                return Err(
                    io::Error::other("RFC 6455 masking-key reuse unexpectedly succeeded").into(),
                );
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
        .map_err(|_| io::Error::other("WebSocket mask-reuse test server panicked"))??;
    assert_eq!(received, "first-frame");
    Ok(())
}

#[test]
fn established_stream_rejects_mask_reuse_across_text_and_pong() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<String> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let first = read_masked_text(&mut stream)?;
        require_peer_closed_before_second_frame(&mut stream)?;
        Ok(first)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let established = establish(&endpoint)?;
    let reused_mask = WebDriverBiDiWebSocketMaskKey::new([0x25, 0x26, 0x27, 0x28]);
    let established =
        established.write_text_frame("first-frame", reused_mask, Duration::from_millis(500))?;
    let error = match established.write_pong_frame(
        b"second-frame",
        reused_mask,
        Duration::from_millis(500),
    ) {
        Ok(_) => {
            return Err(
                io::Error::other("cross-type masking-key reuse unexpectedly succeeded").into(),
            );
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
        .map_err(|_| io::Error::other("cross-type mask-reuse test server panicked"))??;
    assert_eq!(received, "first-frame");
    Ok(())
}

#[test]
fn established_stream_round_trips_pong_and_unmasked_server_text() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<Vec<u8>> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let pong_payload = read_masked_frame(&mut stream, 0x0a)?;
        stream.write_all(&[0x81, 0x05, b'r', b'e', b'p', b'l', b'y'])?;
        Ok(pong_payload)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let established = establish(&endpoint)?;
    let established = established.write_pong_frame(
        b"probe",
        WebDriverBiDiWebSocketMaskKey::new([0x31, 0x32, 0x33, 0x34]),
        Duration::from_millis(500),
    )?;
    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    assert!(frame.fin());
    assert_eq!(frame.opcode(), 0x1);
    assert_eq!(frame.payload(), b"reply");

    let pong_payload = server
        .join()
        .map_err(|_| io::Error::other("WebSocket frame round-trip test server panicked"))??;
    assert_eq!(pong_payload, b"probe");
    Ok(())
}

#[test]
fn established_stream_rejects_payloads_above_reviewed_bounds() -> Result<(), Box<dyn Error>> {
    for (text_case, payload_bytes, maximum_bytes) in [
        (true, 1_048_577_usize, 1_048_576_usize),
        (false, 126_usize, 125_usize),
    ] {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let local_addr = listener.local_addr()?;
        let server = thread::spawn(move || -> io::Result<()> {
            let (mut stream, _) = listener.accept()?;
            read_opening_request(&mut stream)?;
            stream.write_all(OPENING_RESPONSE)?;
            require_peer_closed_without_frame(&mut stream)
        });

        let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
        let established = establish(&endpoint)?;
        let masking_key = WebDriverBiDiWebSocketMaskKey::new([0x41, 0x42, 0x43, 0x44]);
        let error = if text_case {
            let oversized_text = "x".repeat(payload_bytes);
            match established.write_text_frame(
                &oversized_text,
                masking_key,
                Duration::from_millis(500),
            ) {
                Ok(_) => return Err(io::Error::other("oversized text frame succeeded").into()),
                Err(error) => error,
            }
        } else {
            let oversized_pong = vec![0_u8; payload_bytes];
            match established.write_pong_frame(
                &oversized_pong,
                masking_key,
                Duration::from_millis(500),
            ) {
                Ok(_) => return Err(io::Error::other("oversized Pong frame succeeded").into()),
                Err(error) => error,
            }
        };
        match error {
            WebDriverBiDiWebSocketFrameError::FrameTooLarge {
                payload_bytes: actual_payload_bytes,
                maximum_bytes: actual_maximum_bytes,
            } => {
                assert_eq!(actual_payload_bytes, payload_bytes);
                assert_eq!(actual_maximum_bytes, maximum_bytes);
            }
            other => {
                return Err(io::Error::other(format!(
                    "oversized payload failed with the wrong error: {other}"
                ))
                .into());
            }
        }
        server
            .join()
            .map_err(|_| io::Error::other("oversized-frame test server panicked"))??;
    }
    Ok(())
}

#[test]
fn established_stream_rejects_invalid_frame_timeouts_before_io() -> Result<(), Box<dyn Error>> {
    for operation in 0_u8..3 {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let local_addr = listener.local_addr()?;
        let server = thread::spawn(move || -> io::Result<()> {
            let (mut stream, _) = listener.accept()?;
            read_opening_request(&mut stream)?;
            stream.write_all(OPENING_RESPONSE)?;
            require_peer_closed_without_frame(&mut stream)
        });

        let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
        let established = establish(&endpoint)?;
        let masking_key = WebDriverBiDiWebSocketMaskKey::new([0x51, 0x52, 0x53, 0x54]);
        let error = match operation {
            0 => established
                .write_text_frame("probe", masking_key, Duration::ZERO)
                .err()
                .ok_or_else(|| io::Error::other("zero-timeout text frame succeeded"))?,
            1 => established
                .write_pong_frame(b"probe", masking_key, Duration::ZERO)
                .err()
                .ok_or_else(|| io::Error::other("zero-timeout Pong frame succeeded"))?,
            _ => established
                .read_frame(Duration::ZERO)
                .err()
                .ok_or_else(|| io::Error::other("zero-timeout frame read succeeded"))?,
        };
        assert!(matches!(
            error,
            WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
                frame_timeout: Duration::ZERO,
                ..
            }
        ));
        server
            .join()
            .map_err(|_| io::Error::other("invalid-timeout test server panicked"))??;
    }
    Ok(())
}
