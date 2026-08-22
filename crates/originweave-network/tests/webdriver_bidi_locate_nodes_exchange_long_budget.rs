use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use originweave_core::{
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiWebSocketEndpoint,
};
use originweave_network::{
    WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMaskKey,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const RESPONSE_DOCUMENT: &str =
    r#"{"type":"success","id":7,"result":{"nodes":[{"type":"node","sharedId":"shared-1"}]}}"#;

fn read_client_text_frame(stream: &mut impl Read) {
    let mut header = [0_u8; 2];
    stream
        .read_exact(&mut header)
        .expect("command frame header must arrive");
    assert_eq!(header[0], 0x81);
    assert_ne!(header[1] & 0x80, 0);

    let payload_length = match header[1] & 0x7f {
        value @ 0..=125 => usize::from(value),
        126 => {
            let mut extended = [0_u8; 2];
            stream
                .read_exact(&mut extended)
                .expect("16-bit command length must arrive");
            usize::from(u16::from_be_bytes(extended))
        }
        127 => panic!("test command must not require a 64-bit WebSocket length"),
        _ => unreachable!("7-bit WebSocket payload marker"),
    };

    let mut mask = [0_u8; 4];
    stream
        .read_exact(&mut mask)
        .expect("command mask must arrive");
    let mut payload = vec![0_u8; payload_length];
    stream
        .read_exact(&mut payload)
        .expect("command payload must arrive");
}

#[test]
fn exchange_budget_above_per_frame_ceiling_remains_a_valid_end_to_end_budget() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("test listener must bind");
    let address = listener
        .local_addr()
        .expect("test listener address must be available");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("test server must accept");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("test read timeout must configure");

        let mut opening = Vec::new();
        let mut byte = [0_u8; 1];
        while !opening.ends_with(b"\r\n\r\n") {
            stream
                .read_exact(&mut byte)
                .expect("opening request must arrive");
            opening.push(byte[0]);
        }
        stream
            .write_all(
                b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
            )
            .expect("opening response must be written");

        read_client_text_frame(&mut stream);

        let response = RESPONSE_DOCUMENT.as_bytes();
        let response_length = u8::try_from(response.len()).expect("response must fit short frame");
        stream
            .write_all(&[0x81, response_length])
            .expect("response header must be written");
        stream
            .write_all(response)
            .expect("response payload must be written");
    });

    let endpoint =
        WebDriverBiDiWebSocketEndpoint::new(&format!("ws://{address}/session/{SESSION_ID}"))
            .expect("test endpoint must be valid");
    let correlated = endpoint
        .correlate_session_id(SESSION_ID)
        .expect("test session must correlate");
    let target = correlated
        .into_explicit_connect_target()
        .expect("test target must be explicit");
    let connection = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)
        .expect("test connection plan must be valid")
        .connect()
        .expect("test connection must succeed");
    let key = WebDriverBiDiWebSocketClientKey::new(RFC6455_SAMPLE_KEY)
        .expect("test client key must be valid");
    let established = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)
        .expect("test handshake plan must be valid")
        .write_opening_request(Duration::from_millis(500))
        .expect("opening request must be written")
        .read_opening_response(Duration::from_millis(500))
        .expect("opening response must be valid");

    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Checkout"), 2)
        .expect("test query must be valid");
    let command = WebDriverBiDiLocateNodesCommand::new(7, "top-level-context", &query)
        .expect("test command must be valid");

    let exchanged = established.exchange_locate_nodes(
        command,
        WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]),
        &mut || None,
        Duration::from_secs(6),
    );
    assert!(exchanged.is_ok(), "{exchanged:?}");
    assert!(server.join().is_ok());
}
