use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiSessionEndCommand, WebDriverBiDiSessionEndResult,
    WebDriverBiDiSessionTeardownAssessment, WebDriverBiDiSessionTeardownDisposition,
    WebDriverBiDiSessionTeardownObservations, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketMessageAssembler, WebDriverBiDiWebSocketMessageAssembly,
    WebDriverBiDiWebSocketTransportClosureKind, WebDriverBiDiWebSocketTransportClosureObservation,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const END_SUCCESS_RESPONSE: &[u8] = br#"{"type":"success","id":7,"result":{}}"#;
const NORMAL_CLOSE_FRAME: &[u8] = &[0x88, 0x02, 0x03, 0xe8];

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

fn correlated_session_end_ack_and_transport() -> Result<
    (
        WebDriverBiDiSessionEndResult,
        WebDriverBiDiWebSocketEstablished,
    ),
    Box<dyn Error>,
> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut stream)?;
        if command != br#"{"id":7,"method":"session.end","params":{}}"# {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected session.end command",
            ));
        }
        stream.write_all(&[0x81, END_SUCCESS_RESPONSE.len() as u8])?;
        stream.write_all(END_SUCCESS_RESPONSE)?;
        stream.write_all(NORMAL_CLOSE_FRAME)
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

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let established = WebDriverBiDiSessionEndCommand::new(7)?.send(
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;
    let (established, frame) = established.read_frame(Duration::from_millis(500))?;
    let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
    let text = match assembler.push_frame(frame)? {
        WebDriverBiDiWebSocketMessageAssembly::Text(text) => text,
        other => {
            return Err(io::Error::other(format!(
                "session.end response produced unexpected assembly state: {other:?}"
            ))
            .into());
        }
    };
    let acknowledged = WebDriverBiDiSessionEndResult::parse_and_correlate(&text, &mut correlation)?;
    server
        .join()
        .map_err(|_| io::Error::other("session.end teardown test server panicked"))??;
    Ok((acknowledged, established))
}

fn observed_transport_closure(
    established: WebDriverBiDiWebSocketEstablished,
) -> Result<WebDriverBiDiWebSocketTransportClosureObservation, Box<dyn Error>> {
    let observation = WebDriverBiDiWebSocketTransportClosureObservation::observe(
        established,
        Duration::from_millis(500),
    )?;
    assert_eq!(
        observation.kind(),
        WebDriverBiDiWebSocketTransportClosureKind::PeerCloseFrame
    );
    assert_eq!(observation.peer_close_status_code(), Some(1000));
    Ok(observation)
}

#[test]
fn missing_typed_transport_or_process_profile_observation_keeps_teardown_pending()
-> Result<(), Box<dyn Error>> {
    let incomplete_observations = [
        (false, true, true),
        (true, false, true),
        (true, true, false),
    ];

    for (transport_closed, browser_exited, profile_removed) in incomplete_observations {
        let (acknowledged, established) = correlated_session_end_ack_and_transport()?;
        let transport_closure = if transport_closed {
            Some(observed_transport_closure(established)?)
        } else {
            drop(established);
            None
        };
        let assessment = WebDriverBiDiSessionTeardownAssessment::from_protocol_ack(
            acknowledged,
            WebDriverBiDiSessionTeardownObservations::new(
                transport_closure,
                browser_exited,
                profile_removed,
            ),
        );

        assert_eq!(assessment.command_id(), 7);
        assert_eq!(
            assessment.observations().transport_closed_observed(),
            transport_closed
        );
        assert_eq!(
            assessment
                .observations()
                .transport_closure_observation()
                .map(WebDriverBiDiWebSocketTransportClosureObservation::kind),
            transport_closed.then_some(WebDriverBiDiWebSocketTransportClosureKind::PeerCloseFrame)
        );
        assert_eq!(
            assessment.observations().browser_process_exited_observed(),
            browser_exited
        );
        assert_eq!(
            assessment.observations().task_profile_removed_observed(),
            profile_removed
        );
        assert!(!assessment.is_operationally_complete());
        assert_eq!(
            assessment.disposition(),
            WebDriverBiDiSessionTeardownDisposition::OperationalTeardownPending
        );
    }
    Ok(())
}

#[test]
fn caller_asserted_process_and_profile_placeholders_cannot_complete_teardown()
-> Result<(), Box<dyn Error>> {
    let (acknowledged, established) = correlated_session_end_ack_and_transport()?;
    let transport_closure = observed_transport_closure(established)?;
    let assessment = WebDriverBiDiSessionTeardownAssessment::from_protocol_ack(
        acknowledged,
        WebDriverBiDiSessionTeardownObservations::new(Some(transport_closure), true, true),
    );

    assert!(!assessment.is_operationally_complete());
    assert_eq!(
        assessment.disposition(),
        WebDriverBiDiSessionTeardownDisposition::OperationalTeardownPending
    );
    assert_eq!(assessment.command_id(), 7);
    assert_eq!(
        assessment
            .observations()
            .transport_closure_observation()
            .map(WebDriverBiDiWebSocketTransportClosureObservation::peer_close_status_code),
        Some(Some(1000))
    );
    Ok(())
}
