use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiLocateNodesExchangeError, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";

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
            "expected one final masked client text frame",
        ));
    }

    let payload_length = match header[1] & 0x7f {
        value @ 0..=125 => usize::from(value),
        126 => {
            let mut extended = [0_u8; 2];
            stream.read_exact(&mut extended)?;
            usize::from(u16::from_be_bytes(extended))
        }
        127 => {
            let mut extended = [0_u8; 8];
            stream.read_exact(&mut extended)?;
            usize::try_from(u64::from_be_bytes(extended)).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "client frame payload length did not fit usize",
                )
            })?
        }
        marker => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid client frame payload-length marker {marker}"),
            ));
        }
    };

    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_length];
    stream.read_exact(&mut payload)?;
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
fn second_text_message_during_fragmentation_fails_closed() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
        )?;
        read_masked_client_text_frame(&mut stream)?;
        stream.write_all(&[0x01, 0x01, b'{'])?;
        stream.write_all(&[0x81, 0x01, b'x'])
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let admitted = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let established = written.read_opening_response(Duration::from_millis(500))?;

    let exchange = established.exchange_locate_nodes(
        locate_nodes_command()?,
        WebDriverBiDiWebSocketMaskKey::new([0x31, 0x32, 0x33, 0x34]),
        &mut || None,
        Duration::from_millis(500),
    );

    server
        .join()
        .map_err(|_| io::Error::other("fragmented second-text server panicked"))??;
    assert_eq!(
        format!("{exchange:?}"),
        "Err(UnexpectedResponseFrame { fin: true, opcode: 1 })"
    );
    assert!(matches!(
        exchange,
        Err(
            WebDriverBiDiLocateNodesExchangeError::UnexpectedResponseFrame {
                fin: true,
                opcode: 0x1
            }
        )
    ));
    Ok(())
}
