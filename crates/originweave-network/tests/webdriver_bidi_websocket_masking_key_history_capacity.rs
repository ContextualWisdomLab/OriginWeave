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
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const FRAME_COUNT: u32 = 65_537;

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
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
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

fn read_one_masked_single_byte_text_frame(stream: &mut TcpStream) -> io::Result<()> {
    let mut frame = [0_u8; 7];
    stream.read_exact(&mut frame)?;
    if frame[0] != 0x81 || frame[1] != 0x81 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client did not send one final masked single-byte text frame",
        ));
    }
    if frame[6] ^ frame[2] != b'x' {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "masked text payload did not decode to the expected byte",
        ));
    }
    Ok(())
}

#[test]
fn established_stream_does_not_gain_a_lifetime_frame_cap_from_reuse_detection()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<u32> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        for _ in 0..FRAME_COUNT {
            read_one_masked_single_byte_text_frame(&mut stream)?;
        }
        Ok(FRAME_COUNT)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let mut established = written.read_opening_response(Duration::from_millis(500))?;

    for ordinal in 0..FRAME_COUNT {
        let key_ordinal = (ordinal % (FRAME_COUNT - 1)) + 1;
        let masking_key = WebDriverBiDiWebSocketMaskKey::new(key_ordinal.to_be_bytes());
        established = established.write_text_frame("x", masking_key, Duration::from_millis(500))?;
    }
    drop(established);

    let received = server
        .join()
        .map_err(|_| io::Error::other("WebSocket history-cap test server panicked"))??;
    assert_eq!(received, FRAME_COUNT);
    Ok(())
}
