use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiNavigationCommittedSubscriptionCommandError, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
};

const REGISTRY_SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const FOREIGN_TRANSPORT_SESSION_ID: &str = "fedcba98-7654-3210-fedc-ba9876543210";
const CONTEXT_ID: &str = "context-a";
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
                "opening request ended before the header terminator",
            ));
        }
        request.extend_from_slice(&buffer[..count]);
    }
    Ok(())
}

fn spawn_foreign_transport_server(listener: TcpListener) -> thread::JoinHandle<io::Result<bool>> {
    thread::spawn(move || {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        stream.set_read_timeout(Some(Duration::from_millis(500)))?;

        let mut first_command_byte = [0_u8; 1];
        match stream.read(&mut first_command_byte) {
            Ok(0) => Ok(false),
            Ok(_) => Ok(true),
            Err(source)
                if matches!(
                    source.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                Ok(false)
            }
            Err(source) => Err(source),
        }
    })
}

fn establish(
    local_addr: SocketAddr,
    session_id: &str,
) -> Result<WebDriverBiDiWebSocketEstablished, Box<dyn Error>> {
    let endpoint = format!("ws://{local_addr}/session/{session_id}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(session_id)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    Ok(WebDriverBiDiWebSocketHandshakePlan::new(
        connection,
        WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?,
    )?
    .write_opening_request(Duration::from_millis(500))?
    .read_opening_response(Duration::from_millis(500))?)
}

#[test]
fn registry_bound_subscription_is_rejected_before_writing_to_a_foreign_session_transport()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(REGISTRY_SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7, &registry, session, context, CONTEXT_ID,
    )?;

    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = spawn_foreign_transport_server(listener);
    let established = establish(local_addr, FOREIGN_TRANSPORT_SESSION_ID)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();

    let send_result = command.send(
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    );
    let command_byte_seen = server
        .join()
        .map_err(|_| io::Error::other("foreign-session fixture server panicked"))??;

    assert!(
        send_result.is_err(),
        "registry session A unexpectedly dispatched on transport session B"
    );
    assert!(matches!(
        send_result,
        Err(
            WebDriverBiDiNavigationCommittedSubscriptionCommandError::ContextBinding {
                source: BrowserRegistryError::SessionExternalIdentifierMismatch
            }
        )
    ));
    assert_eq!(
        correlation.outstanding_count(),
        0,
        "foreign-session rejection must happen before correlation registration"
    );
    assert!(
        !command_byte_seen,
        "foreign-session rejection must happen before any command-frame byte"
    );
    Ok(())
}
