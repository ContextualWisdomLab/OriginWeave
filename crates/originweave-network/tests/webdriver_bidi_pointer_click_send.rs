use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::Duration,
};

use originweave_core::{
    WebDriverBiDiPointerClickCommand, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandKind, WebDriverBiDiTcpConnectionPlan,
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, send_webdriver_bidi_pointer_click,
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

fn read_masked_text_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;
    if header[0] != 0x81 || header[1] & 0x80 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected one final masked client text frame",
        ));
    }

    let marker = header[1] & 0x7f;
    let length = match marker {
        0..=125 => usize::from(marker),
        126 => {
            let mut extended = [0_u8; 2];
            stream.read_exact(&mut extended)?;
            let length = usize::from(u16::from_be_bytes(extended));
            if length <= 125 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "client text frame used non-minimal 16-bit length encoding",
                ));
            }
            length
        }
        127 => {
            let mut extended = [0_u8; 8];
            stream.read_exact(&mut extended)?;
            let length = u64::from_be_bytes(extended);
            if length <= u64::from(u16::MAX) || length > usize::MAX as u64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "client text frame used invalid 64-bit length encoding",
                ));
            }
            length as usize
        }
        _ => unreachable!(),
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

#[test]
fn pointer_click_command_writes_exact_masked_bidi_frame_and_stays_outstanding()
-> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let expected = WebDriverBiDiPointerClickCommand::new(
        42,
        "context-a",
        &WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?,
    )?;
    let expected_json = expected.as_json().as_bytes().to_vec();

    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let command = read_masked_text_frame(&mut stream)?;
        if command != expected_json {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected input.performActions pointer-click command",
            ));
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

    let command = WebDriverBiDiPointerClickCommand::new(
        42,
        "context-a",
        &WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?,
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let _established = send_webdriver_bidi_pointer_click(
        &command,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_millis(500),
    )?;
    assert_eq!(correlation.outstanding_count(), 1);

    server
        .join()
        .map_err(|_| io::Error::other("pointer-click transport test server panicked"))??;
    Ok(())
}

#[test]
fn pointer_click_reused_mask_key_rejection_retires_correlation() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let seed = read_masked_text_frame(&mut stream)?;
        if seed != b"{}" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected seed frame before reused-key regression",
            ));
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
    let repeated_key = WebDriverBiDiWebSocketMaskKey::new([9, 10, 11, 12]);
    let established =
        established.write_text_frame("{}", repeated_key, Duration::from_millis(500))?;

    let command = WebDriverBiDiPointerClickCommand::new(
        43,
        "context-a",
        &WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-43"))?,
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let error = send_webdriver_bidi_pointer_click(
        &command,
        established,
        &mut correlation,
        repeated_key,
        Duration::from_millis(500),
    )
    .err()
    .ok_or_else(|| io::Error::other("reused masking key unexpectedly sent a pointer click"))?;
    assert_eq!(correlation.outstanding_count(), 0);
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi pointer-click command frame write failed"
    );

    server
        .join()
        .map_err(|_| io::Error::other("reused-mask-key pointer test server panicked"))??;
    Ok(())
}

#[test]
fn pointer_click_ambiguous_socket_write_keeps_correlation() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let (closed_sender, closed_receiver) = mpsc::channel();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let mut first_frame_byte = [0_u8; 1];
        stream.read_exact(&mut first_frame_byte)?;
        drop(stream);
        closed_sender
            .send(())
            .map_err(|_| io::Error::other("pointer-click close signal receiver disappeared"))
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
        .read_opening_response(Duration::from_millis(500))?
        .write_text_frame(
            "seed-frame",
            WebDriverBiDiWebSocketMaskKey::new([13, 14, 15, 16]),
            Duration::from_millis(500),
        )?;
    closed_receiver.recv_timeout(Duration::from_secs(1))?;

    let command = WebDriverBiDiPointerClickCommand::new(
        44,
        "context-a",
        &WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-44"))?,
    )?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let mut established = established;
    let mut observed_ambiguous_failure = false;
    for attempt in 0_u8..64 {
        match send_webdriver_bidi_pointer_click(
            &command,
            established,
            &mut correlation,
            WebDriverBiDiWebSocketMaskKey::new([17, 18, 19, attempt]),
            Duration::from_millis(500),
        ) {
            Ok(next) => {
                correlation.retire_command_for(44, WebDriverBiDiCommandKind::PointerClick)?;
                established = next;
            }
            Err(error) => {
                assert!(error.source().is_some());
                assert_eq!(correlation.outstanding_count(), 1);
                observed_ambiguous_failure = true;
                break;
            }
        }
    }
    assert!(observed_ambiguous_failure);

    server
        .join()
        .map_err(|_| io::Error::other("ambiguous-write pointer test server panicked"))??;
    Ok(())
}
