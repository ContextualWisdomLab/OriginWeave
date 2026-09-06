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
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketHandshakeResponseError,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const RFC6455_SAMPLE_ACCEPT: &str = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";

type TestResult<T> = Result<T, Box<dyn Error>>;

fn read_opening_request(stream: &mut std::net::TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    let mut buffer = [0_u8; 256];
    while !request.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..count]);
    }
    if request.ends_with(b"\r\n\r\n") {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "opening request ended before headers",
        ))
    }
}

#[test]
fn response_without_mandatory_space_after_status_code_fails_closed() -> TestResult<()> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        write!(
            stream,
            "HTTP/1.1 101\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {RFC6455_SAMPLE_ACCEPT}\r\n\r\n"
        )?;
        Ok(())
    });

    let endpoint_url = format!("ws://{local_addr}/session/{SESSION_ID}");
    let endpoint = WebDriverBiDiWebSocketEndpoint::new(&endpoint_url)?;
    let correlated = endpoint.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let written = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?
        .write_opening_request(Duration::from_millis(500))?;

    let result = written.read_opening_response(Duration::from_millis(500));
    match server.join() {
        Ok(server_result) => server_result?,
        Err(_) => return Err(io::Error::other("loopback fixture thread panicked").into()),
    }

    assert!(matches!(
        result,
        Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { .. })
    ));
    Ok(())
}
