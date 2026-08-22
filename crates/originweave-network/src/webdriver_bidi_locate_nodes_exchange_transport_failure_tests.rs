use std::{
    error::Error,
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiWebSocketEndpoint,
};

use crate::{
    WebDriverBiDiLocateNodesExchangeError, WebDriverBiDiTcpConnection,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";

fn connect(endpoint: &str) -> Result<WebDriverBiDiTcpConnection, Box<dyn Error>> {
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

fn read_masked_client_text_frame(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x81 || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client command was not one final masked text frame",
        ));
    }

    let payload_length = match header[1] & 0x7f {
        value @ 0..=125 => u64::from(value),
        126 => {
            let mut extended = [0_u8; 2];
            stream.read_exact(&mut extended)?;
            u64::from(u16::from_be_bytes(extended))
        }
        _ => {
            let mut extended = [0_u8; 8];
            stream.read_exact(&mut extended)?;
            u64::from_be_bytes(extended)
        }
    };

    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut remaining = usize::try_from(payload_length).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "client command length cannot fit this test process",
        )
    })?;
    let mut buffer = [0_u8; 512];
    while remaining != 0 {
        let chunk = remaining.min(buffer.len());
        stream.read_exact(&mut buffer[..chunk])?;
        remaining -= chunk;
    }
    Ok(())
}

fn locate_nodes_command() -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Checkout"), 2)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        7,
        "top-level-context",
        &query,
    )?)
}

#[test]
fn locate_nodes_exchange_preserves_pong_write_failure_after_ping() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        read_masked_client_text_frame(&mut stream)?;
        stream.write_all(&[0x89, 0])?;
        thread::sleep(Duration::from_millis(250));
        Ok(())
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;
    let shutdown_stream = established.try_clone_stream_for_test()?;
    let pong_key = WebDriverBiDiWebSocketMaskKey::new([0x51, 0x52, 0x53, 0x54]);
    let exchanged = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || {
            let shutdown = shutdown_stream.shutdown(Shutdown::Write);
            assert!(shutdown.is_ok(), "{shutdown:?}");
            Some(pong_key)
        },
        Duration::from_millis(500),
    );

    let server_result = server
        .join()
        .map_err(|_| io::Error::other("Pong write failure test server panicked"))?;
    assert!(server_result.is_ok(), "{server_result:?}");

    let error = exchanged.err().ok_or_else(|| {
        io::Error::other("locateNodes exchange unexpectedly survived a closed client write half")
    })?;
    assert!(
        matches!(
            &error,
            WebDriverBiDiLocateNodesExchangeError::Frame(
                WebDriverBiDiWebSocketFrameError::FrameWriteFailed { .. }
            )
        ),
        "{error:?}"
    );
    assert!(error.source().is_some());
    Ok(())
}
