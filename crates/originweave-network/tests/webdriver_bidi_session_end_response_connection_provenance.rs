use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiConnectionMessageRead,
    WebDriverBiDiReceivedTextMessage, WebDriverBiDiSessionEndCommand,
    WebDriverBiDiSessionEndResponseError, WebDriverBiDiSessionEndResult,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageReader,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const END_SUCCESS_RESPONSE: &[u8] = br#"{"type":"success","id":7,"result":{}}"#;

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
            "session.end command unexpectedly required extended framing",
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

fn establish(
    local_addr: SocketAddr,
) -> Result<originweave_network::WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(SESSION_ID)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    Ok(WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?
        .write_opening_request(Duration::from_millis(500))?
        .read_opening_response(Duration::from_millis(500))?)
}

fn read_one_connection_bound_text(
    established: originweave_network::WebDriverBiDiWebSocketEstablished,
) -> Result<WebDriverBiDiReceivedTextMessage, Box<dyn Error>> {
    let mut reader = WebDriverBiDiWebSocketMessageReader::new(established);
    loop {
        match reader.read_next(Duration::from_millis(500))? {
            WebDriverBiDiConnectionMessageRead::Pending(next) => reader = next,
            WebDriverBiDiConnectionMessageRead::Text { message, .. } => return Ok(message),
            WebDriverBiDiConnectionMessageRead::Control { message, .. } => {
                return Err(io::Error::other(format!(
                    "foreign response produced unexpected control message: {message:?}"
                ))
                .into());
            }
        }
    }
}

#[test]
fn reconnected_response_cannot_consume_prior_connection_command() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut first, _) = listener.accept()?;
        read_opening_request(&mut first)?;
        first.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut first)?;
        if command != br#"{"id":7,"method":"session.end","params":{}}"# {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected session.end command on first connection",
            ));
        }

        let (mut second, _) = listener.accept()?;
        read_opening_request(&mut second)?;
        second.write_all(OPENING_RESPONSE)?;
        second.write_all(&[0x81, END_SUCCESS_RESPONSE.len() as u8])?;
        second.write_all(END_SUCCESS_RESPONSE)?;
        Ok(())
    });

    let first = establish(local_addr)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let _first = WebDriverBiDiSessionEndCommand::new(7)?.send(
        first,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;
    assert_eq!(correlation.outstanding_count(), 1);

    let second = establish(local_addr)?;
    let foreign_response = read_one_connection_bound_text(second)?;
    let parsed =
        WebDriverBiDiSessionEndResult::parse_and_correlate(&foreign_response, &mut correlation);
    let error = parsed
        .err()
        .ok_or_else(|| io::Error::other("foreign response acknowledged prior connection"))?;

    assert!(matches!(
        error,
        WebDriverBiDiSessionEndResponseError::TransportConnectionMismatch { command_id: 7 }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.end response arrived on a different connection"
    );
    assert_eq!(
        correlation.outstanding_count(),
        1,
        "foreign response must not consume connection A correlation"
    );

    server
        .join()
        .map_err(|_| io::Error::other("connection-provenance test server panicked"))??;
    Ok(())
}
