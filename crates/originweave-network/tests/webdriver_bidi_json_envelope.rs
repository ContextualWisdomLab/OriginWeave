use std::{
    error::Error,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{
    MAX_WEBDRIVER_BIDI_JSON_DEPTH, WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError,
    WebDriverBiDiJsonEnvelopeKind, WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakePlan, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const RFC6455_SAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const OPENING_RESPONSE: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
const SUCCESS_MESSAGE: &[u8] = br#"{"type":"success","id":7,"result":{"ready":true}}"#;

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

fn parse_over_real_transport(
    payload: &[u8],
) -> Result<Result<WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError>, Box<dyn Error>> {
    if payload.len() > 125 {
        return Err(io::Error::other("test payload exceeded one-byte frame length").into());
    }

    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    let payload = payload.to_vec();
    let server = thread::spawn(move || -> io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        read_opening_request(&mut stream)?;
        stream.write_all(OPENING_RESPONSE)?;
        let payload_len = u8::try_from(payload.len())
            .map_err(|_| io::Error::other("test payload length does not fit u8"))?;
        stream.write_all(&[0x81, payload_len])?;
        stream.write_all(&payload)
    });

    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let admitted = WebDriverBiDiWebSocketEndpoint::new(&endpoint)?;
    let correlated = admitted.correlate_session_id(SESSION_ID)?;
    let target = correlated.into_explicit_connect_target()?;
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
    let parsed = WebDriverBiDiJsonEnvelope::parse(&text);

    server
        .join()
        .map_err(|_| io::Error::other("JSON-envelope server panicked"))??;
    Ok(parsed)
}

#[test]
fn real_transport_text_is_classified_as_bidi_success_envelope() -> Result<(), Box<dyn Error>> {
    let envelope = parse_over_real_transport(SUCCESS_MESSAGE)?;
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::kind),
        Ok(WebDriverBiDiJsonEnvelopeKind::Success)
    );
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::command_id),
        Ok(Some(7))
    );
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::method),
        Ok(None)
    );
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::error_code),
        Ok(None)
    );
    Ok(())
}

#[test]
fn real_transport_classifies_error_and_event_envelopes() -> Result<(), Box<dyn Error>> {
    let cases: &[&[u8]] = &[
        br#"{"type":"error","id":null,"error":"unknown error","message":"","stacktrace":"hidden"}"#,
        br#"{"type":"error","id":7,"error":"unknown error","message":""}"#,
    ];
    for document in cases {
        assert_eq!(
            parse_over_real_transport(document)?
                .as_ref()
                .map(WebDriverBiDiJsonEnvelope::kind),
            Ok(WebDriverBiDiJsonEnvelopeKind::Error)
        );
    }

    let event = parse_over_real_transport(
        br#"{"type":"event","method":"browsingContext.load","params":{}}"#,
    )?;
    assert_eq!(
        event.as_ref().map(WebDriverBiDiJsonEnvelope::kind),
        Ok(WebDriverBiDiJsonEnvelopeKind::Event)
    );
    assert_eq!(
        event.as_ref().map(WebDriverBiDiJsonEnvelope::method),
        Ok(Some("browsingContext.load"))
    );
    Ok(())
}

#[test]
fn real_transport_exercises_valid_json_boundaries() -> Result<(), Box<dyn Error>> {
    let cases: &[&[u8]] = &[
        br#"{"type":"success","id":1,"result":{"slash":"\/","upper":"\uABCD","edge":"\uFFFF"}}"#,
        br#"{"type":"success","id":1,"result":{"array":[1,2],"number":-2.5e3}}"#,
        br#"{"type":"success","id":1,"result":{"pair":"\ud83d\ude00","empty":[]}}"#,
        r#"{"type":"success","id":1,"result":{"unicode":"é"}}"#.as_bytes(),
    ];
    for document in cases {
        assert_eq!(
            parse_over_real_transport(document)?
                .as_ref()
                .map(WebDriverBiDiJsonEnvelope::kind),
            Ok(WebDriverBiDiJsonEnvelopeKind::Success)
        );
    }
    Ok(())
}

#[test]
fn real_transport_rejects_malformed_json_at_parser_boundaries() -> Result<(), Box<dyn Error>> {
    let cases: &[&[u8]] = &[
        br#"{"type":"success","id":1,"result":{}} trailing"#,
        br#"{"type":"success","unterminated"#,
        br#"{"type" "success"}"#,
        br#"{"type":"success" "id":1,"result":{}}"#,
        br#"{"type":"success","id":1,"result":{"x" 1}}"#,
        br#"{"type":"success","id":1,"result":{"x":1 "y":2}}"#,
        br#"{"type":"success","id":1,"result":{"x":[1 2]}}"#,
        br#"{"type":"success","id":1,"result":{"x":[1,]}}"#,
        b"{\"type\":\"success\",\"id\":1,\"result\":{\"x\":\"\\",
        br#"{"type":"success","id":1,"result":{"x":"\q"}}"#,
        br#"{"type":"success","id":1,"result":{"x":"\u12xz"}}"#,
        br#"{"type":"success","id":1,"result":{"x":"\udc00"}}"#,
        br#"{"type":"success","id":1,"result":{"x":"\ud800\x"}}"#,
        b"{\"type\":\"success\",\"id\":1,\"result\":{\"x\":\"\\ud800\\u",
    ];
    for document in cases {
        assert_eq!(
            parse_over_real_transport(document)?,
            Err(WebDriverBiDiJsonEnvelopeError::InvalidJson)
        );
    }
    Ok(())
}

#[test]
fn real_transport_rejects_missing_required_envelope_members() -> Result<(), Box<dyn Error>> {
    let cases: &[(&[u8], WebDriverBiDiJsonEnvelopeError)] = &[
        (
            br#"{}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "type" },
        ),
        (
            br#"{"type":"success","result":{}}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "id" },
        ),
        (
            br#"{"type":"success","id":1}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "result" },
        ),
        (
            br#"{"type":"error","error":"x","message":"m"}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "id" },
        ),
        (
            br#"{"type":"error","id":null,"message":"m"}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "error" },
        ),
        (
            br#"{"type":"error","id":null,"error":"x"}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "message" },
        ),
        (
            br#"{"type":"event","params":{}}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "method" },
        ),
        (
            br#"{"type":"event","method":"x"}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "params" },
        ),
    ];
    for (document, expected) in cases {
        assert_eq!(parse_over_real_transport(document)?, Err(expected.clone()));
    }
    Ok(())
}

#[test]
fn real_transport_debug_and_error_display_remain_payload_minimal() -> Result<(), Box<dyn Error>> {
    let parsed = parse_over_real_transport(
        br#"{"type":"success","id":7,"result":{"ready":"sensitive-result"}}"#,
    )?;
    let envelope = parsed.map_err(|error| io::Error::other(error.to_string()))?;
    let debug = format!("{envelope:?}");
    assert!(debug.contains("Success"));
    assert!(!debug.contains("ready"));
    assert!(!debug.contains("sensitive-result"));

    let cases: &[(&[u8], WebDriverBiDiJsonEnvelopeError)] = &[
        (br#"[]"#, WebDriverBiDiJsonEnvelopeError::RootMustBeObject),
        (
            br#"{"type":"success","type":"event","id":1,"result":{}}"#,
            WebDriverBiDiJsonEnvelopeError::DuplicateTopLevelMember,
        ),
        (
            br#"{"type":"other"}"#,
            WebDriverBiDiJsonEnvelopeError::UnsupportedEnvelopeType,
        ),
        (
            br#"{}"#,
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "type" },
        ),
        (
            br#"{"type":"error","id":-1,"error":"x","message":"secret-message"}"#,
            WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "id" },
        ),
        (
            br#"{"type":"success","id":1,"result":{}} trailing secret"#,
            WebDriverBiDiJsonEnvelopeError::InvalidJson,
        ),
    ];
    for (document, expected) in cases {
        let parsed = parse_over_real_transport(document)?;
        let error = parsed
            .err()
            .ok_or_else(|| io::Error::other("invalid envelope unexpectedly parsed"))?;
        assert_eq!(error, expected.clone());
        let display = error.to_string();
        assert!(!display.is_empty());
        assert!(!display.contains("secret-message"));
        assert!(!display.contains("trailing secret"));
    }

    let nesting_error = WebDriverBiDiJsonEnvelopeError::NestingTooDeep {
        maximum_depth: MAX_WEBDRIVER_BIDI_JSON_DEPTH,
    };
    let display = nesting_error.to_string();
    assert!(display.contains("maximum depth"));
    Ok(())
}

#[test]
fn real_transport_rejects_negative_error_response_id() -> Result<(), Box<dyn Error>> {
    let error = parse_over_real_transport(
        br#"{"type":"error","id":-1,"error":"invalid argument","message":"bad id"}"#,
    )?;
    assert_eq!(
        error,
        Err(WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "id" })
    );
    Ok(())
}

#[test]
fn real_transport_rejects_non_protocol_error_code() -> Result<(), Box<dyn Error>> {
    let error = parse_over_real_transport(
        br#"{"type":"error","id":7,"error":"attacker-defined-code","message":"bad code"}"#,
    )?;
    assert_eq!(
        error,
        Err(WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "error" })
    );
    Ok(())
}

#[test]
fn real_transport_accepts_spec_defined_client_window_error() -> Result<(), Box<dyn Error>> {
    let envelope = parse_over_real_transport(
        br#"{"type":"error","id":7,"error":"no such client window","message":"unknown client window"}"#,
    )?;
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::kind),
        Ok(WebDriverBiDiJsonEnvelopeKind::Error)
    );
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::command_id),
        Ok(Some(7))
    );
    assert_eq!(
        envelope.as_ref().map(WebDriverBiDiJsonEnvelope::error_code),
        Ok(Some("no such client window"))
    );
    Ok(())
}
