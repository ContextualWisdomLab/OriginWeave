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

fn connect(endpoint: &str) -> TestResult<originweave_network::WebDriverBiDiTcpConnection> {
    let endpoint = WebDriverBiDiWebSocketEndpoint::new(endpoint)?;
    let correlated = endpoint.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    Ok(connection)
}

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
    if !request.ends_with(b"\r\n\r\n") {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "opening request ended before headers",
        ));
    }
    Ok(())
}

fn exercise_response(
    response: Vec<u8>,
) -> TestResult<Result<(), WebDriverBiDiWebSocketHandshakeResponseError>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(&response)?;
        Ok(())
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let plan = WebDriverBiDiWebSocketHandshakePlan::new(connect(&endpoint)?, key)?;
    let written = plan.write_opening_request(Duration::from_millis(500))?;
    let result = written
        .read_opening_response(Duration::from_millis(500))
        .map(|established| {
            drop(established);
        });
    match server.join() {
        Ok(server_result) => server_result?,
        Err(_) => return Err(io::Error::other("loopback fixture thread panicked").into()),
    }
    Ok(result)
}

#[test]
fn repeated_list_valued_upgrade_and_connection_lines_are_combined_semantically() -> TestResult<()> {
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: h2c\r\n\
Upgrade: websocket\r\n\
Connection: keep-alive\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: {RFC6455_SAMPLE_ACCEPT}\r\n\r\n"
    )
    .into_bytes();

    let result = exercise_response(response)?;
    assert!(
        result.is_ok(),
        "RFC 9110 list-valued fields must combine: {result:?}"
    );
    Ok(())
}

#[test]
fn repeated_sec_websocket_accept_remains_fail_closed() -> TestResult<()> {
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: {RFC6455_SAMPLE_ACCEPT}\r\n\
Sec-WebSocket-Accept: {RFC6455_SAMPLE_ACCEPT}\r\n\r\n"
    )
    .into_bytes();

    assert!(matches!(
        exercise_response(response)?,
        Err(WebDriverBiDiWebSocketHandshakeResponseError::MalformedResponse { .. })
    ));
    Ok(())
}
