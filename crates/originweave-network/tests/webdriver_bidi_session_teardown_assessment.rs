use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiConnectionMessageRead,
    WebDriverBiDiSessionEndCommand, WebDriverBiDiSessionEndResult,
    WebDriverBiDiSessionTeardownAssessment, WebDriverBiDiSessionTeardownAssessmentError,
    WebDriverBiDiSessionTeardownDisposition, WebDriverBiDiSessionTeardownObservations,
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketMessageReader,
    WebDriverBiDiWebSocketTransportClosureKind, WebDriverBiDiWebSocketTransportClosureObservation,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const SECOND_SESSION_ID: &str = "fedcba98-7654-3210-fedc-ba9876543210";
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

fn correlated_session_end_ack_and_transport_on(
    listener: &TcpListener,
    session_id: &str,
) -> Result<
    (
        WebDriverBiDiSessionEndResult,
        WebDriverBiDiWebSocketEstablished,
    ),
    Box<dyn Error>,
> {
    let local_addr = listener.local_addr()?;
    let server_listener = listener.try_clone()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = server_listener.accept()?;
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

    let endpoint = format!("ws://{local_addr}/session/{session_id}");
    let target = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?
        .correlate_session_id(session_id)?
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
    let mut reader = WebDriverBiDiWebSocketMessageReader::new(established);
    let (established, text) = loop {
        match reader.read_next(Duration::from_millis(500))? {
            WebDriverBiDiConnectionMessageRead::Pending(next) => reader = next,
            WebDriverBiDiConnectionMessageRead::Text {
                established,
                message,
            } => break (established, message),
            WebDriverBiDiConnectionMessageRead::Control { message, .. } => {
                return Err(io::Error::other(format!(
                    "session.end response produced unexpected control message: {message:?}"
                ))
                .into());
            }
        }
    };
    let acknowledged = WebDriverBiDiSessionEndResult::parse_and_correlate(&text, &mut correlation)?;
    server
        .join()
        .map_err(|_| io::Error::other("session.end teardown test server panicked"))??;
    Ok((acknowledged, established))
}

fn correlated_session_end_ack_and_transport_for(
    session_id: &str,
) -> Result<
    (
        WebDriverBiDiSessionEndResult,
        WebDriverBiDiWebSocketEstablished,
    ),
    Box<dyn Error>,
> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    correlated_session_end_ack_and_transport_on(&listener, session_id)
}

fn correlated_session_end_ack_and_transport() -> Result<
    (
        WebDriverBiDiSessionEndResult,
        WebDriverBiDiWebSocketEstablished,
    ),
    Box<dyn Error>,
> {
    correlated_session_end_ack_and_transport_for(SESSION_ID)
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
fn missing_typed_transport_observation_keeps_teardown_pending() -> Result<(), Box<dyn Error>> {
    for transport_closed in [false, true] {
        let (acknowledged, established) = correlated_session_end_ack_and_transport()?;
        let transport_closure = if transport_closed {
            Some(observed_transport_closure(established)?)
        } else {
            drop(established);
            None
        };
        let assessment = WebDriverBiDiSessionTeardownAssessment::from_protocol_ack(
            acknowledged,
            WebDriverBiDiSessionTeardownObservations::new(transport_closure),
        )?;

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
        assert!(!assessment.is_operationally_complete());
        assert_eq!(
            assessment.disposition(),
            WebDriverBiDiSessionTeardownDisposition::OperationalTeardownPending
        );
    }
    Ok(())
}

#[test]
fn typed_transport_closure_cannot_complete_teardown_without_process_and_profile_evidence()
-> Result<(), Box<dyn Error>> {
    let (acknowledged, established) = correlated_session_end_ack_and_transport()?;
    let transport_closure = observed_transport_closure(established)?;
    let assessment = WebDriverBiDiSessionTeardownAssessment::from_protocol_ack(
        acknowledged,
        WebDriverBiDiSessionTeardownObservations::new(Some(transport_closure)),
    )?;

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

fn assert_cross_connection_closure_rejected(
    acknowledged: WebDriverBiDiSessionEndResult,
    foreign_established: WebDriverBiDiWebSocketEstablished,
) -> Result<(), Box<dyn Error>> {
    let foreign_closure = observed_transport_closure(foreign_established)?;
    let result = WebDriverBiDiSessionTeardownAssessment::from_protocol_ack(
        acknowledged,
        WebDriverBiDiSessionTeardownObservations::new(Some(foreign_closure)),
    );

    let error = result
        .err()
        .ok_or_else(|| io::Error::other("closure from another connection was accepted"))?;
    assert_eq!(
        error,
        WebDriverBiDiSessionTeardownAssessmentError::TransportConnectionMismatch
    );
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi transport closure does not match the acknowledged connection"
    );
    assert!(error.source().is_none());
    Ok(())
}

#[test]
fn reconnected_same_session_and_endpoint_cannot_supply_closure_for_prior_transport()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let (acknowledged_a, established_a) =
        correlated_session_end_ack_and_transport_on(&listener, SESSION_ID)?;
    drop(established_a);
    let (_acknowledged_b, established_b) =
        correlated_session_end_ack_and_transport_on(&listener, SESSION_ID)?;
    assert_cross_connection_closure_rejected(acknowledged_a, established_b)
}

#[test]
fn another_session_cannot_supply_closure_for_the_acknowledged_transport()
-> Result<(), Box<dyn Error>> {
    let (acknowledged_a, established_a) = correlated_session_end_ack_and_transport_for(SESSION_ID)?;
    drop(established_a);
    let (_acknowledged_b, established_b) =
        correlated_session_end_ack_and_transport_for(SECOND_SESSION_ID)?;
    assert_cross_connection_closure_rejected(acknowledged_a, established_b)
}
