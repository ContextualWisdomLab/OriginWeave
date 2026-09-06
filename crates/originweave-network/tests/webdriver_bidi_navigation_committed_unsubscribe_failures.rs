use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{BrowserAuthorityRegistry, WebDriverBiDiWebSocketEndpoint};
use originweave_network::{
    MAX_WEBDRIVER_BIDI_JS_UINT, WebDriverBiDiCommandCorrelation,
    WebDriverBiDiCommandCorrelationError, WebDriverBiDiCommandKind,
    WebDriverBiDiConnectionMessageRead, WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiNavigationCommittedSubscriptionResult,
    WebDriverBiDiNavigationCommittedUnsubscribeCommand,
    WebDriverBiDiNavigationCommittedUnsubscribeCommandError,
    WebDriverBiDiNavigationCommittedUnsubscribeResponseError,
    WebDriverBiDiNavigationCommittedUnsubscribeResult, WebDriverBiDiReceivedTextMessage,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageReader,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CONTEXT_ID: &str = "context-a";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const SUBSCRIBE_RESPONSE: &str =
    r#"{"type":"success","id":7,"result":{"subscription":"sub-\"\\\n\u0001-구독"}}"#;
const MALFORMED_UNSUBSCRIBE_RESPONSE: &[u8] = br#"{"type":"success","id":8,"result":"#;
const UNKNOWN_UNSUBSCRIBE_RESPONSE: &[u8] = br#"{"type":"success","id":9,"result":{}}"#;
const MATCHED_UNSUBSCRIBE_SUCCESS: &[u8] = br#"{"type":"success","id":8,"result":{}}"#;
const MATCHED_UNSUBSCRIBE_ERROR: &[u8] =
    br#"{"type":"error","id":8,"error":"invalid argument","message":"denied"}"#;

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
    let length = match header[1] & 0x7f {
        length @ 0..=125 => usize::from(length),
        126 => {
            let mut extended = [0_u8; 2];
            stream.read_exact(&mut extended)?;
            usize::from(u16::from_be_bytes(extended))
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "test command unexpectedly required 64-bit framing",
            ));
        }
    };
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
}

fn read_empty_masked_pong_frame(
    stream: &mut TcpStream,
    expected_masking_key: [u8; 4],
) -> io::Result<()> {
    let mut frame = [0_u8; 6];
    stream.read_exact(&mut frame)?;
    let expected = [
        0x8a,
        0x80,
        expected_masking_key[0],
        expected_masking_key[1],
        expected_masking_key[2],
        expected_masking_key[3],
    ];
    if frame != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected one empty masked client pong frame",
        ));
    }
    Ok(())
}

fn write_unmasked_text_frame(stream: &mut TcpStream, document: &[u8]) -> io::Result<()> {
    if document.len() <= 125 {
        let length = u8::try_from(document.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "short frame length exceeds u8")
        })?;
        stream.write_all(&[0x81, length])?;
    } else {
        let length = u16::try_from(document.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "test document exceeds two-byte frame length",
            )
        })?;
        stream.write_all(&[0x81, 126])?;
        stream.write_all(&length.to_be_bytes())?;
    }
    stream.write_all(document)
}

fn require_no_client_command(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut byte = [0_u8; 1];
    match stream.read(&mut byte) {
        Ok(0) => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsubscribe command was written despite local rejection",
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
        let mut stream = accept_subscription(listener)?;
        require_no_client_command(&mut stream)
    })
}

fn accept_subscription(listener: TcpListener) -> io::Result<TcpStream> {
    let (mut stream, _) = listener.accept()?;
    read_opening_request(&mut stream)?;
    stream.write_all(OPENING_RESPONSE)?;
    let subscribe = read_masked_text_frame(&mut stream)?;
    if subscribe
        != br#"{"id":7,"method":"session.subscribe","params":{"events":["browsingContext.navigationCommitted"],"contexts":["context-a"]}}"#
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected session.subscribe command: {}", String::from_utf8_lossy(&subscribe)),
        ));
    }
    write_unmasked_text_frame(&mut stream, SUBSCRIBE_RESPONSE.as_bytes())?;
    Ok(stream)
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

fn obtain_subscription_receipt(
    established: WebDriverBiDiWebSocketEstablished,
) -> Result<
    (
        WebDriverBiDiWebSocketEstablished,
        WebDriverBiDiNavigationCommittedSubscriptionResult,
    ),
    Box<dyn Error>,
> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let subscribe = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7, &registry, session, context, CONTEXT_ID,
    )?;
    let established = subscribe.send(
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;
    let (established, received) = match WebDriverBiDiWebSocketMessageReader::new(established)
        .read_next(Duration::from_millis(500))?
    {
        WebDriverBiDiConnectionMessageRead::Text {
            established,
            message,
        } => (established, message),
        other => {
            return Err(io::Error::other(format!(
                "session.subscribe response produced unexpected connection-bound state: {other:?}"
            ))
            .into());
        }
    };
    let subscription = WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
        &received,
        &mut correlation,
    )?;

    Ok((established, subscription))
}

fn standalone_subscription_receipt()
-> Result<WebDriverBiDiNavigationCommittedSubscriptionResult, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = spawn_no_command_server(listener);
    let (established, subscription) =
        obtain_subscription_receipt(establish_websocket(local_addr)?)?;
    drop(established);
    server
        .join()
        .map_err(|_| io::Error::other("subscription receipt test server panicked"))??;
    Ok(subscription)
}

fn read_received_text(
    established: WebDriverBiDiWebSocketEstablished,
) -> Result<
    (
        WebDriverBiDiWebSocketEstablished,
        WebDriverBiDiReceivedTextMessage,
    ),
    Box<dyn Error>,
> {
    match WebDriverBiDiWebSocketMessageReader::new(established)
        .read_next(Duration::from_millis(500))?
    {
        WebDriverBiDiConnectionMessageRead::Text {
            established,
            message,
        } => Ok((established, message)),
        other => Err(io::Error::other(format!(
            "session.unsubscribe response produced unexpected connection-bound state: {other:?}"
        ))
        .into()),
    }
}

fn read_text_over_loopback(
    document: &'static [u8],
) -> Result<WebDriverBiDiReceivedTextMessage, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        write_unmasked_text_frame(&mut stream, document)
    });

    let (established, text) = read_received_text(establish_websocket(local_addr)?)?;
    drop(established);

    server
        .join()
        .map_err(|_| io::Error::other("unsubscribe response test server panicked"))??;
    Ok(text)
}

#[test]
fn command_validation_and_debug_are_public_and_subscription_safe() -> Result<(), Box<dyn Error>> {
    let subscription = standalone_subscription_receipt()?;
    let range = match WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(
        MAX_WEBDRIVER_BIDI_JS_UINT + 1,
        subscription,
    ) {
        Ok(_) => {
            return Err(
                io::Error::other("out-of-range unsubscribe command id was accepted").into(),
            );
        }
        Err(error) => error,
    };
    assert_eq!(
        range.to_string(),
        "WebDriver BiDi session.unsubscribe command id is outside the js-uint range"
    );
    assert!(range.source().is_none());
    assert!(matches!(
        &range,
        WebDriverBiDiNavigationCommittedUnsubscribeCommandError::CommandIdOutOfRange {
            command_id,
            maximum_command_id,
        } if *command_id == MAX_WEBDRIVER_BIDI_JS_UINT + 1
            && *maximum_command_id == MAX_WEBDRIVER_BIDI_JS_UINT
    ));

    let subscription = standalone_subscription_receipt()?;
    let subscription_id = subscription.subscription_id().to_owned();
    let command = WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(8, subscription)?;
    let debug = format!("{command:?}");
    assert!(debug.contains("command_id: 8"));
    assert!(debug.contains(&format!("subscription_id_len: {}", subscription_id.len())));
    assert!(!debug.contains(&subscription_id));
    Ok(())
}

#[test]
fn duplicate_command_id_is_rejected_before_unsubscribe_write() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = spawn_no_command_server(listener);
    let (established, subscription) =
        obtain_subscription_receipt(establish_websocket(local_addr)?)?;
    let command = WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(8, subscription)?;

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation
        .register_command_for(8, WebDriverBiDiCommandKind::NavigationCommittedUnsubscribe)?;
    let result = command.send(
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
        Duration::from_millis(500),
    );
    let result = result.map(drop);
    server
        .join()
        .map_err(|_| io::Error::other("duplicate-command test server panicked"))??;
    let error = match result {
        Ok(_) => {
            return Err(io::Error::other("duplicate command id sent unsubscribe command").into());
        }
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.unsubscribe command correlation was rejected"
    );
    assert!(error.source().is_some());
    assert!(matches!(
        &error,
        WebDriverBiDiNavigationCommittedUnsubscribeCommandError::Correlation { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    Ok(())
}

#[test]
fn invalid_frame_timeout_fails_before_unsubscribe_correlation_or_write()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = spawn_no_command_server(listener);
    let (established, subscription) =
        obtain_subscription_receipt(establish_websocket(local_addr)?)?;
    let command = WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(8, subscription)?;

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let result = command.send(
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
        Duration::ZERO,
    );
    let result = result.map(drop);
    server
        .join()
        .map_err(|_| io::Error::other("frame-timeout test server panicked"))??;
    let error = match result {
        Ok(_) => {
            return Err(io::Error::other("zero frame timeout sent unsubscribe command").into());
        }
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.unsubscribe command frame write failed"
    );
    assert!(error.source().is_some());
    assert!(matches!(
        &error,
        WebDriverBiDiNavigationCommittedUnsubscribeCommandError::FrameWrite { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 0);

    Ok(())
}

#[test]
fn adjacent_mask_key_reuse_is_rejected_inside_unsubscribe_send_and_retires_correlation()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let mut stream = accept_subscription(listener)?;
        read_empty_masked_pong_frame(&mut stream, [5, 6, 7, 8])?;
        require_no_client_command(&mut stream)
    });

    let masking_key = WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]);
    let (established, subscription) =
        obtain_subscription_receipt(establish_websocket(local_addr)?)?;
    let established = established.write_pong_frame(&[], masking_key, Duration::from_millis(500))?;
    let command = WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(8, subscription)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let result = command.send(
        established,
        &mut correlation,
        masking_key,
        Duration::from_millis(500),
    );
    let result = result.map(drop);
    server
        .join()
        .map_err(|_| io::Error::other("mask-reuse test server panicked"))??;
    let error = match result {
        Ok(_) => {
            return Err(io::Error::other("reused masking key sent unsubscribe command").into());
        }
        Err(error) => error,
    };
    assert!(matches!(
        &error,
        WebDriverBiDiNavigationCommittedUnsubscribeCommandError::FrameWrite {
            source: WebDriverBiDiWebSocketFrameError::MalformedFrame { .. },
        }
    ));
    assert_eq!(correlation.outstanding_count(), 0);

    Ok(())
}

#[test]
fn malformed_and_unknown_unsubscribe_responses_preserve_outstanding_correlation()
-> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation
        .register_command_for(8, WebDriverBiDiCommandKind::NavigationCommittedUnsubscribe)?;

    let malformed = read_text_over_loopback(MALFORMED_UNSUBSCRIBE_RESPONSE)?;
    let error = match WebDriverBiDiNavigationCommittedUnsubscribeResult::parse_and_correlate(
        &malformed,
        &mut correlation,
    ) {
        Ok(_) => {
            return Err(io::Error::other("malformed unsubscribe response was accepted").into());
        }
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.unsubscribe envelope is invalid"
    );
    assert!(error.source().is_some());
    assert!(matches!(
        &error,
        WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Envelope { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 1);

    let unknown = read_text_over_loopback(UNKNOWN_UNSUBSCRIBE_RESPONSE)?;
    let error = match WebDriverBiDiNavigationCommittedUnsubscribeResult::parse_and_correlate(
        &unknown,
        &mut correlation,
    ) {
        Ok(_) => {
            return Err(io::Error::other("unknown unsubscribe response id was accepted").into());
        }
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.unsubscribe response correlation failed"
    );
    assert!(error.source().is_some());
    assert!(matches!(
        &error,
        WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Correlation { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn unsubscribe_response_cannot_consume_subscription_command_kind() -> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation
        .register_command_for(8, WebDriverBiDiCommandKind::NavigationCommittedSubscription)?;
    let matched = read_text_over_loopback(MATCHED_UNSUBSCRIBE_SUCCESS)?;

    let error = match WebDriverBiDiNavigationCommittedUnsubscribeResult::parse_and_correlate(
        &matched,
        &mut correlation,
    ) {
        Ok(_) => {
            return Err(io::Error::other("unsubscribe consumed subscription correlation").into());
        }
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandKindMismatch {
                expected: WebDriverBiDiCommandKind::NavigationCommittedUnsubscribe,
                actual: WebDriverBiDiCommandKind::NavigationCommittedSubscription,
            },
        }
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn matched_unsubscribe_protocol_error_consumes_only_its_command() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let mut stream = accept_subscription(listener)?;
        let unsubscribe = read_masked_text_frame(&mut stream)?;
        if unsubscribe
            != r#"{"id":8,"method":"session.unsubscribe","params":{"subscriptions":["sub-\"\\\n\u0001-구독"]}}"#
                .as_bytes()
        {
            return Err(io::Error::other("unexpected session.unsubscribe command"));
        }
        write_unmasked_text_frame(&mut stream, MATCHED_UNSUBSCRIBE_ERROR)?;
        require_no_client_command(&mut stream)
    });
    let (established, subscription) =
        obtain_subscription_receipt(establish_websocket(local_addr)?)?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command_for(99, WebDriverBiDiCommandKind::SessionStatus)?;
    let established = WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(8, subscription)?
        .send(
            established,
            &mut correlation,
            WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
            Duration::from_millis(500),
        )?;
    let before_response = correlation.outstanding_count();
    let (established, matched) = read_received_text(established)?;
    let result = WebDriverBiDiNavigationCommittedUnsubscribeResult::parse_and_correlate(
        &matched,
        &mut correlation,
    );
    drop(established);
    server
        .join()
        .map_err(|_| io::Error::other("matched unsubscribe error server panicked"))??;
    let error = match result {
        Ok(_) => {
            return Err(
                io::Error::other("protocol-error unsubscribe response was accepted").into(),
            );
        }
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi session.unsubscribe returned a protocol error"
    );
    assert!(error.source().is_none());
    assert!(matches!(
        &error,
        WebDriverBiDiNavigationCommittedUnsubscribeResponseError::RemoteProtocolError {
            command_id: 8,
        }
    ));
    assert_eq!(before_response, 2);
    assert_eq!(correlation.outstanding_count(), 1);
    correlation.retire_command_for(99, WebDriverBiDiCommandKind::SessionStatus)?;
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}

#[test]
fn generic_unsubscribe_registration_cannot_admit_received_success_or_error()
-> Result<(), Box<dyn Error>> {
    for document in [MATCHED_UNSUBSCRIBE_SUCCESS, MATCHED_UNSUBSCRIBE_ERROR] {
        let mut correlation = WebDriverBiDiCommandCorrelation::new();
        correlation
            .register_command_for(8, WebDriverBiDiCommandKind::NavigationCommittedUnsubscribe)?;
        correlation.register_command_for(99, WebDriverBiDiCommandKind::SessionStatus)?;
        let received = read_text_over_loopback(document)?;
        let result = WebDriverBiDiNavigationCommittedUnsubscribeResult::parse_and_correlate(
            &received,
            &mut correlation,
        );
        assert!(matches!(
            result,
            Err(
                WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Correlation {
                    source:
                        WebDriverBiDiCommandCorrelationError::CommandConnectionProvenanceMissing {
                            command_id: 8,
                        }
                }
            )
        ));
        assert_eq!(correlation.outstanding_count(), 2);
        correlation
            .retire_command_for(8, WebDriverBiDiCommandKind::NavigationCommittedUnsubscribe)?;
        correlation.retire_command_for(99, WebDriverBiDiCommandKind::SessionStatus)?;
        assert_eq!(correlation.outstanding_count(), 0);
    }
    Ok(())
}
