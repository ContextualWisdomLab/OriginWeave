use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketTransportClosureError,
    WebDriverBiDiWebSocketTransportClosureKind, WebDriverBiDiWebSocketTransportClosureObservation,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";

type EstablishedWithServer = (
    originweave_network::WebDriverBiDiWebSocketEstablished,
    thread::JoinHandle<io::Result<()>>,
);

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

fn established_with_server_frame(
    frame: Option<&'static [u8]>,
) -> Result<EstablishedWithServer, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        if let Some(frame) = frame {
            stream.write_all(frame)?;
        }
        Ok(())
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
    Ok((established, server))
}

#[test]
fn validated_peer_close_frame_yields_nonforgeable_transport_observation()
-> Result<(), Box<dyn Error>> {
    let (established, server) = established_with_server_frame(Some(&[0x88, 0x02, 0x03, 0xe8]))?;

    let observation = WebDriverBiDiWebSocketTransportClosureObservation::observe(
        established,
        Duration::from_millis(500),
    )?;

    server
        .join()
        .map_err(|_| io::Error::other("transport-close test server panicked"))??;
    assert_eq!(
        observation.kind(),
        WebDriverBiDiWebSocketTransportClosureKind::PeerCloseFrame
    );
    assert_eq!(observation.peer_close_status_code(), Some(1000));
    Ok(())
}

#[test]
fn one_unsolicited_pong_before_close_does_not_block_closure_observation()
-> Result<(), Box<dyn Error>> {
    let (established, server) =
        established_with_server_frame(Some(&[0x8a, 0x00, 0x88, 0x02, 0x03, 0xe8]))?;

    let observation = WebDriverBiDiWebSocketTransportClosureObservation::observe(
        established,
        Duration::from_millis(500),
    )?;

    server
        .join()
        .map_err(|_| io::Error::other("pong-before-close test server panicked"))??;
    assert_eq!(
        observation.kind(),
        WebDriverBiDiWebSocketTransportClosureKind::PeerCloseFrame
    );
    assert_eq!(observation.peer_close_status_code(), Some(1000));
    Ok(())
}

#[test]
fn repeated_pong_frames_remain_fail_closed_under_fixed_read_budget() -> Result<(), Box<dyn Error>> {
    let (established, server) =
        established_with_server_frame(Some(&[0x8a, 0x00, 0x8a, 0x00]))?;

    let Err(error) = WebDriverBiDiWebSocketTransportClosureObservation::observe(
        established,
        Duration::from_millis(500),
    ) else {
        return Err(io::Error::other(
            "repeated Pong frames unexpectedly became transport-closure evidence",
        )
        .into());
    };

    server
        .join()
        .map_err(|_| io::Error::other("repeated-pong test server panicked"))??;
    assert!(matches!(
        &error,
        WebDriverBiDiWebSocketTransportClosureError::UnexpectedFrame { opcode: 0xa }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi peer sent non-closure traffic instead of closing"
    );
    assert!(error.source().is_none());
    Ok(())
}

#[test]
fn clean_peer_eof_yields_transport_observation_without_inventing_close_status()
-> Result<(), Box<dyn Error>> {
    let (established, server) = established_with_server_frame(None)?;

    let observation = WebDriverBiDiWebSocketTransportClosureObservation::observe(
        established,
        Duration::from_millis(500),
    )?;

    server
        .join()
        .map_err(|_| io::Error::other("transport-eof test server panicked"))??;
    assert_eq!(
        observation.kind(),
        WebDriverBiDiWebSocketTransportClosureKind::PeerEof
    );
    assert_eq!(observation.peer_close_status_code(), None);
    Ok(())
}

#[test]
fn application_frame_after_teardown_does_not_become_closure_evidence() -> Result<(), Box<dyn Error>>
{
    let (established, server) = established_with_server_frame(Some(&[0x81, 0x02, b'o', b'k']))?;

    let Err(error) = WebDriverBiDiWebSocketTransportClosureObservation::observe(
        established,
        Duration::from_millis(500),
    ) else {
        return Err(io::Error::other(
            "application frame unexpectedly became transport-closure evidence",
        )
        .into());
    };

    server
        .join()
        .map_err(|_| io::Error::other("unexpected-frame test server panicked"))??;
    assert!(matches!(
        &error,
        WebDriverBiDiWebSocketTransportClosureError::UnexpectedFrame { opcode: 0x1 }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi peer sent non-closure traffic instead of closing"
    );
    assert!(error.source().is_none());
    Ok(())
}

#[test]
fn malformed_close_frame_remains_a_typed_frame_failure() -> Result<(), Box<dyn Error>> {
    let (established, server) = established_with_server_frame(Some(&[0x88, 0x01, 0x00]))?;

    let Err(error) = WebDriverBiDiWebSocketTransportClosureObservation::observe(
        established,
        Duration::from_millis(500),
    ) else {
        return Err(io::Error::other(
            "malformed close frame unexpectedly became transport-closure evidence",
        )
        .into());
    };

    server
        .join()
        .map_err(|_| io::Error::other("malformed-close test server panicked"))??;
    assert!(matches!(
        &error,
        WebDriverBiDiWebSocketTransportClosureError::Frame { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi transport closure could not be observed safely"
    );
    assert!(error.source().is_some());
    Ok(())
}
