use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserSessionId, BrowsingContextId, WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiNavigationCommittedSubscribedObservation,
    WebDriverBiDiNavigationCommittedSubscriptionAdmission,
    WebDriverBiDiNavigationCommittedSubscriptionCommand,
    WebDriverBiDiNavigationCommittedSubscriptionResult, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly,
};

const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const SUBSCRIBE_RESPONSE: &[u8] =
    br#"{"type":"success","id":7,"result":{"subscription":"subscription-a"}}"#;

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
                "fixture subscribe command unexpectedly required 64-bit framing",
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

fn write_text_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    stream.write_all(&[0x81])?;
    match payload.len() {
        0..=125 => stream.write_all(&[payload.len() as u8])?,
        126..=65_535 => {
            stream.write_all(&[126])?;
            stream.write_all(&(payload.len() as u16).to_be_bytes())?;
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "fixture event unexpectedly required 64-bit framing",
            ));
        }
    }
    stream.write_all(payload)
}

fn next_text(
    established: originweave_network::WebDriverBiDiWebSocketEstablished,
    assembler: &mut WebDriverBiDiWebSocketMessageAssembler,
) -> Result<
    (
        originweave_network::WebDriverBiDiWebSocketEstablished,
        originweave_network::WebDriverBiDiWebSocketTextMessage,
    ),
    Box<dyn Error>,
> {
    let (established, frame) = established.read_frame(Duration::from_millis(500))?;
    match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => Ok((established, text)),
        other => Err(io::Error::other(format!(
            "expected a complete WebDriver BiDi text message, got {other:?}"
        ))
        .into()),
    }
}

pub fn receive_subscribed_navigation_event(
    registry: &BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    session_id: &str,
    external_context: &str,
    expected_url: &str,
) -> Result<WebDriverBiDiNavigationCommittedSubscribedObservation, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let expected_command = format!(
        "{{\"id\":7,\"method\":\"session.subscribe\",\"params\":{{\"events\":[\"browsingContext.navigationCommitted\"],\"contexts\":[\"{external_context}\"]}}}}"
    )
    .into_bytes();
    let event = format!(
        "{{\"type\":\"event\",\"method\":\"browsingContext.navigationCommitted\",\"params\":{{\"context\":\"{external_context}\",\"navigation\":\"navigation-a\",\"timestamp\":17,\"url\":\"{expected_url}\"}}}}"
    )
    .into_bytes();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut stream)?;
        if command != expected_command {
            return Err(io::Error::other("unexpected session.subscribe fixture command"));
        }
        write_text_frame(&mut stream, SUBSCRIBE_RESPONSE)?;
        write_text_frame(&mut stream, &event)
    });

    let endpoint = format!("ws://{local_addr}/session/{session_id}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(session_id)?
        .into_explicit_connect_target()?;
    let connection =
        WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)?.connect()?;
    let established = WebDriverBiDiWebSocketHandshakePlan::new(
        connection,
        WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)?,
    )?
    .write_opening_request(Duration::from_millis(500))?
    .read_opening_response(Duration::from_millis(500))?;

    let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
        7,
        registry,
        browser_session,
        browsing_context,
        external_context,
    )?;
    let binding = command.admission_binding();
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let established = command.send(
        registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;

    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let (established, response) = next_text(established, &mut assembler)?;
    let result = WebDriverBiDiNavigationCommittedSubscriptionResult::parse_and_correlate(
        &response,
        &mut correlation,
    )?;
    let admission =
        WebDriverBiDiNavigationCommittedSubscriptionAdmission::new(result, binding, registry)?;
    let (_established, event) = next_text(established, &mut assembler)?;
    let observation = admission.admit(&event, registry, expected_url)?;

    server
        .join()
        .map_err(|_| io::Error::other("subscribed navigation fixture server panicked"))??;
    Ok(observation)
}
