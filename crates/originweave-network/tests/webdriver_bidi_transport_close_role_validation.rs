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
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketTransportClosureError, WebDriverBiDiWebSocketTransportClosureKind,
    WebDriverBiDiWebSocketTransportClosureObservation,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const CLOSE_MASK_KEY: [u8; 4] = [9, 10, 11, 12];
type EstablishedPeer = (
    originweave_network::WebDriverBiDiWebSocketEstablished,
    thread::JoinHandle<io::Result<Option<Vec<u8>>>>,
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

fn read_optional_masked_close(stream: &mut TcpStream) -> io::Result<Option<Vec<u8>>> {
    stream.set_read_timeout(Some(Duration::from_millis(200)))?;
    let mut header = [0_u8; 2];
    match stream.read(&mut header[..1]) {
        Ok(0) => return Ok(None),
        Ok(1) => {}
        Ok(_) => unreachable!("one-byte header read returned more than one byte"),
        Err(source)
            if matches!(
                source.kind(),
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
            ) =>
        {
            return Ok(None);
        }
        Err(source) => return Err(source),
    }
    stream.read_exact(&mut header[1..])?;
    if header[0] != 0x88 || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected one final masked client Close frame",
        ));
    }
    let payload_len = usize::from(header[1] & 0x7f);
    if payload_len > 125 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client Close payload exceeded RFC 6455 control-frame bound",
        ));
    }
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0_u8; payload_len];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(Some(payload))
}

fn established_with_server_close(status_code: u16) -> Result<EstablishedPeer, Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<Option<Vec<u8>>> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let [high, low] = status_code.to_be_bytes();
        stream.write_all(&[0x88, 0x02, high, low])?;
        read_optional_masked_close(&mut stream)
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

fn observe(
    established: originweave_network::WebDriverBiDiWebSocketEstablished,
) -> Result<
    WebDriverBiDiWebSocketTransportClosureObservation,
    WebDriverBiDiWebSocketTransportClosureError,
> {
    WebDriverBiDiWebSocketTransportClosureObservation::observe(
        established,
        &[],
        WebDriverBiDiWebSocketMaskKey::new(CLOSE_MASK_KEY),
        Duration::from_millis(250),
    )
}

#[test]
fn server_close_1010_is_rejected_before_reply_or_closure_evidence() -> Result<(), Box<dyn Error>> {
    let (established, server) = established_with_server_close(1010)?;
    let result = observe(established);
    let reply = server
        .join()
        .map_err(|_| io::Error::other("role-invalid Close peer panicked"))??;

    assert!(
        reply.is_none(),
        "server Close(1010) was mirrored by the client"
    );
    assert!(matches!(
        result,
        Err(WebDriverBiDiWebSocketTransportClosureError::PeerCloseStatusNotAllowed {
            status_code: 1010
        })
    ));
    Ok(())
}

#[test]
fn server_close_1011_remains_valid_and_is_mirrored() -> Result<(), Box<dyn Error>> {
    let (established, server) = established_with_server_close(1011)?;
    let observation = observe(established)?;
    let reply = server
        .join()
        .map_err(|_| io::Error::other("valid server Close peer panicked"))??;

    assert_eq!(reply, Some(1011_u16.to_be_bytes().to_vec()));
    assert_eq!(
        observation.kind(),
        WebDriverBiDiWebSocketTransportClosureKind::PeerCloseThenEof
    );
    assert_eq!(observation.peer_close_status_code(), Some(1011));
    Ok(())
}
