#[path = "support/type_text_intent.rs"]
pub mod type_text_intent;

use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiConnectionMessageRead, WebDriverBiDiJsonEnvelopeError,
    WebDriverBiDiReceivedTextMessage, WebDriverBiDiTypeTextIntentAcknowledgementError as AckError,
    WebDriverBiDiTypeTextIntentWitness, WebDriverBiDiTypeTextResponseError as ResponseError,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketMessageReader,
    acknowledge_webdriver_bidi_type_text_intent,
    send_webdriver_bidi_type_text_with_postcondition_intent,
};
use std::{error::Error, io, io::Read, time::Duration};

const PRIVATE_TEXT: &str = "private-action";
const SUCCESS_ACK: &[u8] = br#"{"type":"success","id":42,"result":{}}"#;
type IntentReply = (
    WebDriverBiDiTypeTextIntentWitness,
    WebDriverBiDiReceivedTextMessage,
    WebDriverBiDiCommandCorrelation,
);

fn intent_reply(reply: &[u8]) -> Result<IntentReply, Box<dyn Error>> {
    let reply = reply.to_vec();
    let (established, peer) = type_text_intent::established_with_peer_script(move |stream| {
        let command = type_text_intent::read_masked_text_frame(stream)?;
        assert!(command.starts_with(br#"{"id":42,"method":"input.performActions""#));
        type_text_intent::write_text_frame(stream, &reply)
    })?;
    let (registry, handle, remote) = type_text_intent::admitted_type_text_fixture()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let (established, witness) = send_webdriver_bidi_type_text_with_postcondition_intent(
        type_text_intent::typed_input_proof()?,
        42,
        "context-a",
        PRIVATE_TEXT,
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_secs(1),
    )?;
    assert_eq!(
        format!("{witness:?}"),
        "WebDriverBiDiTypeTextIntentWitness { command_id: 42, expected_text_bytes: 14 }"
    );
    let received =
        WebDriverBiDiWebSocketMessageReader::new(established).read_next(Duration::from_secs(1));
    peer.join()
        .map_err(|_| io::Error::other("ACK peer panicked"))??;
    match received? {
        WebDriverBiDiConnectionMessageRead::Text { message, .. } => {
            Ok((witness, message, correlation))
        }
        other => Err(io::Error::other(format!("expected ACK text: {other:?}")).into()),
    }
}

#[test]
fn ack_admission_preserves_correlation_and_opaque_diagnostics() -> Result<(), Box<dyn Error>> {
    for (reply, expected, pending) in [
        (SUCCESS_ACK, Ok(()), 0),
        (
            &b"{"[..],
            Err(AckError::Response {
                source: ResponseError::Envelope {
                    source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
                },
            }),
            1,
        ),
        (
            &br#"{"type":"success","id":43,"result":{}}"#[..],
            Err(AckError::ResponseCommandMismatch),
            1,
        ),
        (
            &br#"{"type":"error","id":43,"error":"unknown error","message":"private-action"}"#[..],
            Err(AckError::ResponseCommandMismatch),
            1,
        ),
        (
            &br#"{"type":"error","id":42,"error":"unknown error","message":"private-action"}"#[..],
            Err(AckError::Response {
                source: ResponseError::RemoteProtocolError { command_id: 42 },
            }),
            0,
        ),
        (
            &br#"{"type":"error","id":null,"error":"unknown error","message":"private-action"}"#[..],
            Err(AckError::Response {
                source: ResponseError::Correlation {
                    source: WebDriverBiDiCommandCorrelationError::UncorrelatableErrorResponse,
                },
            }),
            1,
        ),
        (
            &br#"{"type":"event","method":"log.entryAdded","params":{}}"#[..],
            Err(AckError::Response {
                source: ResponseError::Correlation {
                    source: WebDriverBiDiCommandCorrelationError::EventIsNotResponse,
                },
            }),
            1,
        ),
    ] {
        let (witness, message, mut correlation) = intent_reply(reply)?;
        let result =
            acknowledge_webdriver_bidi_type_text_intent(&message, witness, &mut correlation);
        if let (Err(actual), Err(expected)) = (&result, &expected) {
            assert_eq!(actual.to_string(), expected.to_string());
            assert_eq!(actual.source().is_some(), expected.source().is_some());
        }
        let result = result.map(|ack| {
            assert_eq!(ack.command_id(), 42);
            assert_eq!(format!("{ack:?}"),
                "WebDriverBiDiAcknowledgedTypeTextIntent { command_id: 42, expected_text_bytes: 14 }");
        });
        assert_eq!(format!("{result:?}"), format!("{expected:?}"));
        assert_eq!(correlation.outstanding_count(), pending);
    }
    Ok(())
}

#[test]
fn foreign_ack_cannot_consume_original_intent_correlation() -> Result<(), Box<dyn Error>> {
    let (witness, _original, mut correlation) = intent_reply(SUCCESS_ACK)?;
    let (_foreign_witness, foreign, foreign_correlation) = intent_reply(SUCCESS_ACK)?;
    let Err(error) =
        acknowledge_webdriver_bidi_type_text_intent(&foreign, witness, &mut correlation)
    else {
        return Err(io::Error::other("foreign ACK must fail").into());
    };
    assert_eq!(format!("{error:?}"), "ResponseConnectionMismatch");
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-input ACK arrived on a different connection than its intent"
    );
    assert!(error.source().is_none());
    assert_eq!(correlation.outstanding_count(), 1);
    assert_eq!(foreign_correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn rejected_sender_cannot_mint_a_witness_or_emit_command_bytes() -> Result<(), Box<dyn Error>> {
    let (established, peer) = type_text_intent::established_with_peer_script(|stream| {
        let mut byte = [0];
        assert_eq!(stream.read(&mut byte)?, 0);
        Ok(())
    })?;
    let (registry, handle, remote) = type_text_intent::admitted_type_text_fixture()?;
    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let result = send_webdriver_bidi_type_text_with_postcondition_intent(
        type_text_intent::typed_input_proof()?,
        42,
        "context-a",
        "",
        &handle,
        &remote,
        &registry,
        established,
        &mut correlation,
        WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
        Duration::from_secs(1),
    );
    peer.join()
        .map_err(|_| io::Error::other("rejected sender peer panicked"))??;
    assert_eq!(
        format!("{result:?}"),
        "Err(Authority { source: Command(EmptyText) })"
    );
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}
