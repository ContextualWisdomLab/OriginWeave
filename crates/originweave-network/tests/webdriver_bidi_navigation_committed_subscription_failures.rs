use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";

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

fn require_no_client_command(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut byte = [0_u8; 1];
    match stream.read(&mut byte) {
        Ok(0) => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "subscription command was written despite local rejection",
        )),
        Err(source)
            if matches!(
                source.kind(),
                io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted
            ) =>
        {
            Ok(())
        }
        Err(source) => Err(source),
    }
}

fn spawn_no_command_server(listener: TcpListener) -> thread::JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        require_no_client_command(&mut stream)
    })
}

fn establish_websocket(
    local_addr: SocketAddr,
) -> Result<WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
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

#[test]
fn retired_context_is_rejected_before_correlation_or_command_write() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = spawn_no_command_server(listener);

    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "context-a")?;
    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7,
        &registry,
        session,
        context,
        "context-a",
    )?;
    let established = establish_websocket(local_addr)?;
    registry.remove_context(context)?;

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let result = command.send(
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    );
    let error = match result {
        Ok(_) => return Err(io::Error::other("retired context unexpectedly sent subscription").into()),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi navigation subscription context does not match registered authority"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 0);

    server
        .join()
        .map_err(|_| io::Error::other("retired-context test server panicked"))??;
    Ok(())
}

#[test]
fn duplicate_command_id_is_rejected_before_command_write() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = spawn_no_command_server(listener);

    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "context-a")?;
    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7,
        &registry,
        session,
        context,
        "context-a",
    )?;
    let established = establish_websocket(local_addr)?;

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(7)?;
    let result = command.send(
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    );
    let error = match result {
        Ok(_) => return Err(io::Error::other("duplicate command id unexpectedly sent subscription").into()),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi navigation subscription command correlation was rejected"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("duplicate-command test server panicked"))??;
    Ok(())
}

#[test]
fn invalid_frame_timeout_consumes_transport_and_retains_correlation() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = spawn_no_command_server(listener);

    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, "context-a")?;
    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7,
        &registry,
        session,
        context,
        "context-a",
    )?;
    let established = establish_websocket(local_addr)?;

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let result = command.send(
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::ZERO,
    );
    let error = match result {
        Ok(_) => return Err(io::Error::other("zero frame timeout unexpectedly sent subscription").into()),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi navigation subscription command frame write failed"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("frame-write test server panicked"))??;
    Ok(())
}
