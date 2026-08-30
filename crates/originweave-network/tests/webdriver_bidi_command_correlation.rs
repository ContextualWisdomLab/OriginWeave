use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    MAX_WEBDRIVER_BIDI_JS_UINT, MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS,
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCorrelatedResponseOutcome, WebDriverBiDiJsonEnvelope,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly,
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

fn parse_over_loopback(document: &'static [u8]) -> Result<WebDriverBiDiJsonEnvelope, Box<dyn Error>> {
    if document.len() > 125 {
        return Err(io::Error::other("test JSON document exceeded one-byte frame length").into());
    }
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        stream.write_all(&[0x81, document.len() as u8])?;
        stream.write_all(document)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(SESSION_ID)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?;
    let established = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)?
        .write_opening_request(Duration::from_millis(500))?
        .read_opening_response(Duration::from_millis(500))?;
    let (_established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "validated text frame produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    let envelope = WebDriverBiDiJsonEnvelope::parse(&text)?;
    server
        .join()
        .map_err(|_| io::Error::other("command-correlation test server panicked"))??;
    Ok(envelope)
}

#[test]
fn responses_correlate_out_of_order_and_ids_can_be_reused_after_completion() -> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(7)?;
    correlation.register_command(8)?;
    assert_eq!(correlation.outstanding_count(), 2);

    let success = parse_over_loopback(br#"{"type":"success","id":8,"result":{}}"#)?;
    let completed = correlation.correlate_response(&success)?;
    assert_eq!(completed.command_id(), 8);
    assert_eq!(completed.outcome(), WebDriverBiDiCorrelatedResponseOutcome::Success);
    assert_eq!(correlation.outstanding_count(), 1);

    let error = parse_over_loopback(
        br#"{"type":"error","id":7,"error":"invalid argument","message":"redacted by parser"}"#,
    )?;
    let completed = correlation.correlate_response(&error)?;
    assert_eq!(completed.command_id(), 7);
    assert_eq!(completed.outcome(), WebDriverBiDiCorrelatedResponseOutcome::Error);
    assert_eq!(correlation.outstanding_count(), 0);

    correlation.register_command(8)?;
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn correlation_fails_closed_without_consuming_unrelated_outstanding_commands() -> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    correlation.register_command(7)?;

    let unknown = parse_over_loopback(br#"{"type":"success","id":8,"result":{}}"#)?;
    assert_eq!(
        correlation.correlate_response(&unknown),
        Err(WebDriverBiDiCommandCorrelationError::CommandNotOutstanding)
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let event = parse_over_loopback(br#"{"type":"event","method":"log.entryAdded","params":{}}"#)?;
    assert_eq!(
        correlation.correlate_response(&event),
        Err(WebDriverBiDiCommandCorrelationError::EventIsNotResponse)
    );
    assert_eq!(correlation.outstanding_count(), 1);

    let uncorrelatable = parse_over_loopback(
        br#"{"type":"error","id":null,"error":"invalid argument","message":"no command id"}"#,
    )?;
    assert_eq!(
        correlation.correlate_response(&uncorrelatable),
        Err(WebDriverBiDiCommandCorrelationError::UncorrelatableErrorResponse)
    );
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn outstanding_command_budget_and_retirement_are_bounded() -> Result<(), Box<dyn Error>> {
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    assert_eq!(
        correlation.register_command(MAX_WEBDRIVER_BIDI_JS_UINT + 1),
        Err(WebDriverBiDiCommandCorrelationError::CommandIdOutOfRange)
    );
    correlation.register_command(1)?;
    assert_eq!(
        correlation.register_command(1),
        Err(WebDriverBiDiCommandCorrelationError::CommandAlreadyOutstanding)
    );
    correlation.retire_command(1)?;
    assert_eq!(correlation.outstanding_count(), 0);
    assert_eq!(
        correlation.retire_command(1),
        Err(WebDriverBiDiCommandCorrelationError::CommandNotOutstanding)
    );

    for command_id in 0..MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS as u64 {
        correlation.register_command(command_id)?;
    }
    assert_eq!(correlation.outstanding_count(), MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS);
    assert_eq!(
        correlation.register_command(MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS as u64),
        Err(WebDriverBiDiCommandCorrelationError::OutstandingCommandLimit)
    );
    correlation.retire_command(0)?;
    correlation.register_command(MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS as u64)?;
    assert_eq!(correlation.outstanding_count(), MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS);
    Ok(())
}
