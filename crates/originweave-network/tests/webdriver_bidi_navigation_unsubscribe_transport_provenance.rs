use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandKind,
    WebDriverBiDiConnectionMessageRead, WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiNavigationCommittedSubscriptionResult,
    WebDriverBiDiNavigationCommittedUnsubscribeCommand,
    WebDriverBiDiNavigationCommittedUnsubscribeResponseError,
    WebDriverBiDiNavigationCommittedUnsubscribeResult, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketMessageReader, WebDriverBiDiWebSocketTextMessage,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CONTEXT_ID: &str = "context-a";
const FRAME_TIMEOUT: Duration = Duration::from_millis(500);
const SUBSCRIBE_MASK: [u8; 4] = [1, 2, 3, 4];
const UNSUBSCRIBE_MASK: [u8; 4] = [5, 6, 7, 8];
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const SUBSCRIBE_COMMAND: &[u8] = br#"{"id":7,"method":"session.subscribe","params":{"events":["browsingContext.navigationCommitted"],"contexts":["context-a"]}}"#;
const SUBSCRIBE_RESPONSE: &[u8] =
    br#"{"type":"success","id":7,"result":{"subscription":"subscription-a"}}"#;
const UNSUBSCRIBE_COMMAND: &[u8] =
    br#"{"id":8,"method":"session.unsubscribe","params":{"subscriptions":["subscription-a"]}}"#;
const UNSUBSCRIBE_SUCCESS: &[u8] = br#"{"type":"success","id":8,"result":{}}"#;
const UNSUBSCRIBE_ERROR: &[u8] =
    br#"{"type":"error","id":8,"error":"invalid argument","message":"foreign unsubscribe error"}"#;

type TestResult<T> = Result<T, Box<dyn Error>>;

fn accept_websocket(listener: TcpListener) -> io::Result<TcpStream> {
    let (mut stream, _) = listener.accept()?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    while !request.ends_with(b"\r\n\r\n") {
        let mut byte = [0_u8];
        stream.read_exact(&mut byte)?;
        request.push(byte[0]);
    }
    stream.write_all(OPENING_RESPONSE)?;
    Ok(stream)
}

fn receive_exact_command(
    stream: &mut TcpStream,
    payload: &[u8],
    masking_key: [u8; 4],
) -> io::Result<()> {
    let mut expected = vec![0x81];
    if payload.len() <= 125 {
        expected.push(0x80 | payload.len() as u8);
    } else {
        expected.push(0xfe);
        expected.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    }
    expected.extend_from_slice(&masking_key);
    expected.extend(
        payload
            .iter()
            .enumerate()
            .map(|(index, byte)| byte ^ masking_key[index % masking_key.len()]),
    );
    let mut received = vec![0_u8; expected.len()];
    stream.read_exact(&mut received)?;
    if received != expected {
        return Err(io::Error::other(format!(
            "client command differs from literal expected wire bytes: {received:?}"
        )));
    }
    Ok(())
}

fn write_text_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    if payload.len() > 125 {
        return Err(io::Error::other("fixture reply exceeded short-frame encoding"));
    }
    stream.write_all(&[0x81, payload.len() as u8])?;
    stream.write_all(payload)
}

fn read_until_closed(mut stream: TcpStream) -> io::Result<Vec<u8>> {
    let mut received = Vec::new();
    stream.read_to_end(&mut received)?;
    Ok(received)
}

fn join_server(server: thread::JoinHandle<io::Result<Vec<u8>>>) -> TestResult<Vec<u8>> {
    Ok(server
        .join()
        .map_err(|_| io::Error::other("unsubscribe transport fixture server panicked"))??)
}

fn establish(local_addr: SocketAddr) -> TestResult<WebDriverBiDiWebSocketEstablished> {
    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(SESSION_ID)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    Ok(WebDriverBiDiWebSocketHandshakePlan::new(
        connection,
        WebDriverBiDiWebSocketClientKey::new("dGhlIHNhbXBsZSBub25jZQ==")?,
    )?
    .write_opening_request(FRAME_TIMEOUT)?
    .read_opening_response(FRAME_TIMEOUT)?)
}

fn subscribe(
    established: WebDriverBiDiWebSocketEstablished,
    correlation: &mut WebDriverBiDiCommandCorrelation,
) -> TestResult<(
    WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiNavigationCommittedSubscriptionResult,
)> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7, &registry, session, context, CONTEXT_ID,
    )?;
    let established = command.send(
        &registry,
        established,
        correlation,
        WebDriverBiDiWebSocketMaskKey::new(SUBSCRIBE_MASK),
        FRAME_TIMEOUT,
    )?;
    match WebDriverBiDiWebSocketMessageReader::new(established).read_next(FRAME_TIMEOUT)? {
        WebDriverBiDiConnectionMessageRead::Text {
            established,
            message,
        } => Ok((
            established,
            WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
                &message,
                correlation,
            )?,
        )),
        other => Err(io::Error::other(format!(
            "expected actual connection-bound subscription receipt, got {other:?}"
        ))
        .into()),
    }
}

fn read_unsubscribe_text(
    established: WebDriverBiDiWebSocketEstablished,
) -> TestResult<(
    WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketTextMessage,
)> {
    let (established, frame) = established.read_frame(FRAME_TIMEOUT)?;
    match WebDriverBiDiWebSocketMessageAssembler::new().push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(message) => Ok((established, message)),
        other => Err(io::Error::other(format!(
            "expected actual unsubscribe response text frame, got {other:?}"
        ))
        .into()),
    }
}

#[test]
fn subscription_receipt_cannot_dispatch_unsubscribe_on_another_connection() -> TestResult<()> {
    let sent_listener = TcpListener::bind(("127.0.0.1", 0))?;
    let sent_addr = sent_listener.local_addr()?;
    let sent_server = thread::spawn(move || {
        let mut stream = accept_websocket(sent_listener)?;
        receive_exact_command(&mut stream, SUBSCRIBE_COMMAND, SUBSCRIBE_MASK)?;
        write_text_frame(&mut stream, SUBSCRIBE_RESPONSE)?;
        read_until_closed(stream)
    });
    let foreign_listener = TcpListener::bind(("127.0.0.1", 0))?;
    let foreign_addr = foreign_listener.local_addr()?;
    let foreign_server = thread::spawn(move || read_until_closed(accept_websocket(foreign_listener)?));

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(99, WebDriverBiDiCommandKind::SessionStatus)?;
    let (sent_established, subscription) = subscribe(establish(sent_addr)?, &mut correlation)?;
    let before_send = correlation.outstanding_count();
    let result = WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(8, &subscription)?.send(
        establish(foreign_addr)?,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new(UNSUBSCRIBE_MASK),
        FRAME_TIMEOUT,
    );
    let rejected = result.is_err();
    let after_send = correlation.outstanding_count();
    drop(result);
    drop(sent_established);
    let sent_extra = join_server(sent_server)?;
    let foreign_bytes = join_server(foreign_server)?;
    let unrelated_retained = correlation
        .retire_command_for(99, WebDriverBiDiCommandKind::SessionStatus)
        .is_ok();
    let unsubscribe_registered = correlation
        .retire_command_for(8, WebDriverBiDiCommandKind::NavigationCommittedUnsubscribe)
        .is_ok();

    assert!(sent_extra.is_empty(), "unexpected A bytes: {sent_extra:?}");
    assert!(
        foreign_bytes.is_empty(),
        "foreign connection emitted unsubscribe wire bytes: {foreign_bytes:?}"
    );
    assert!(rejected, "a receipt from A must not authorize dispatch on B");
    assert_eq!((before_send, after_send), (1, 1));
    assert!(unrelated_retained);
    assert!(!unsubscribe_registered);
    Ok(())
}

fn foreign_unsubscribe_reply_preserves_original_command(foreign_reply: &'static [u8]) -> TestResult<()> {
    let sent_listener = TcpListener::bind(("127.0.0.1", 0))?;
    let sent_addr = sent_listener.local_addr()?;
    let (release_sender, release_receiver) = mpsc::sync_channel::<()>(0);
    let sent_server = thread::spawn(move || {
        let mut stream = accept_websocket(sent_listener)?;
        receive_exact_command(&mut stream, SUBSCRIBE_COMMAND, SUBSCRIBE_MASK)?;
        write_text_frame(&mut stream, SUBSCRIBE_RESPONSE)?;
        receive_exact_command(&mut stream, UNSUBSCRIBE_COMMAND, UNSUBSCRIBE_MASK)?;
        release_receiver
            .recv_timeout(Duration::from_secs(2))
            .map_err(|error| io::Error::other(format!("original response barrier failed: {error}")))?;
        write_text_frame(&mut stream, UNSUBSCRIBE_SUCCESS)?;
        read_until_closed(stream)
    });
    let foreign_listener = TcpListener::bind(("127.0.0.1", 0))?;
    let foreign_addr = foreign_listener.local_addr()?;
    let foreign_server = thread::spawn(move || {
        let mut stream = accept_websocket(foreign_listener)?;
        write_text_frame(&mut stream, foreign_reply)?;
        read_until_closed(stream)
    });

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(99, WebDriverBiDiCommandKind::SessionStatus)?;
    let (sent_established, subscription) = subscribe(establish(sent_addr)?, &mut correlation)?;
    let sent_established = WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(8, &subscription)?
        .send(
            sent_established,
            &mut correlation,
            WebDriverBiDiWebSocketMaskKey::new(UNSUBSCRIBE_MASK),
            FRAME_TIMEOUT,
        )?;
    let before_foreign = correlation.outstanding_count();
    let (foreign_established, foreign_message) = read_unsubscribe_text(establish(foreign_addr)?)?;
    let foreign_result = WebDriverBiDiNavigationCommittedUnsubscribeResult::parse_and_correlate(
        &foreign_message,
        &mut correlation,
    );
    let after_foreign = correlation.outstanding_count();

    release_sender.send(())?;
    let (sent_established, original_message) = read_unsubscribe_text(sent_established)?;
    let original_result = WebDriverBiDiNavigationCommittedUnsubscribeResult::parse_and_correlate(
        &original_message,
        &mut correlation,
    );
    let after_original = correlation.outstanding_count();
    drop(sent_established);
    drop(foreign_established);
    let sent_extra = join_server(sent_server)?;
    let foreign_bytes = join_server(foreign_server)?;
    let unrelated_retained = correlation
        .retire_command_for(99, WebDriverBiDiCommandKind::SessionStatus)
        .is_ok();

    assert!(sent_extra.is_empty(), "unexpected A bytes: {sent_extra:?}");
    assert!(foreign_bytes.is_empty(), "unexpected B bytes: {foreign_bytes:?}");
    assert_eq!(
        (before_foreign, after_foreign, after_original),
        (2, 2, 1),
        "foreign response must preserve pending A until its genuine reply; foreign={foreign_result:?}, original={original_result:?}"
    );
    assert!(matches!(
        foreign_result,
        Err(WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Correlation { .. })
    ));
    assert_eq!(original_result?.command_id(), 8);
    assert!(unrelated_retained);
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn foreign_unsubscribe_success_preserves_pending_until_original_reply() -> TestResult<()> {
    foreign_unsubscribe_reply_preserves_original_command(UNSUBSCRIBE_SUCCESS)
}

#[test]
fn foreign_unsubscribe_error_preserves_pending_until_original_reply() -> TestResult<()> {
    foreign_unsubscribe_reply_preserves_original_command(UNSUBSCRIBE_ERROR)
}
