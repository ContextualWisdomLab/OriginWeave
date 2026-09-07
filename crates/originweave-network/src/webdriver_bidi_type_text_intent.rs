use std::{error::Error, fmt, time::Duration};

use originweave_core::{
    AdmittedNodeHandle, BrowserAuthorityRegistry, ValidatedBrowserProtocolUse,
    WebDriverBiDiRemoteNodeReference,
};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiReceivedTextMessage,
    WebDriverBiDiTypeTextResponseError, WebDriverBiDiTypeTextResult,
    WebDriverBiDiTypeTextSendError, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketMaskKey, send_webdriver_bidi_type_text,
    webdriver_bidi_connection::WebDriverBiDiConnectionGeneration,
    webdriver_bidi_json_envelope::WebDriverBiDiJsonEnvelopeRouting, WebDriverBiDiJsonEnvelope,
};

/// Sender-minted one-shot witness for the exact non-secret text intent dispatched by one typed-input command.
///
/// The witness is created only after the reviewed typed-input sender successfully writes its frame.
/// It binds the exact command id, the private process-local connection generation, and the validated
/// text value. The text is retained only inside this opaque value until its exact protocol ACK is
/// admitted; `Debug`, errors, and public accessors never expose it. The witness is neither `Clone`
/// nor `Copy`, so a caller cannot duplicate one successful dispatch into multiple action intents.
pub struct WebDriverBiDiTypeTextIntentWitness {
    command_id: u64,
    connection_generation: WebDriverBiDiConnectionGeneration,
    expected_text: Box<str>,
}

impl fmt::Debug for WebDriverBiDiTypeTextIntentWitness {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiTypeTextIntentWitness")
            .field("command_id", &self.command_id)
            .field("expected_text_bytes", &self.expected_text.len())
            .finish()
    }
}

/// One-shot proof that the exact sender-minted typed-input intent received its correlated protocol ACK.
///
/// This value keeps the original non-secret text private until post-condition verification. It can
/// only be constructed by [`acknowledge_webdriver_bidi_type_text_intent`] after the ACK matches both
/// the witness command id and the witness's private connection generation.
pub struct WebDriverBiDiAcknowledgedTypeTextIntent {
    command_id: u64,
    expected_text: Box<str>,
}

impl fmt::Debug for WebDriverBiDiAcknowledgedTypeTextIntent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiAcknowledgedTypeTextIntent")
            .field("command_id", &self.command_id)
            .field("expected_text_bytes", &self.expected_text.len())
            .finish()
    }
}

impl WebDriverBiDiAcknowledgedTypeTextIntent {
    /// Return the exact local typed-input command identifier acknowledged for this intent.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    pub(crate) fn expected_text(&self) -> &str {
        &self.expected_text
    }
}

/// Fail-closed failures while binding a typed-input ACK to its sender-minted intent witness.
#[derive(Debug)]
pub enum WebDriverBiDiTypeTextIntentAcknowledgementError {
    /// The received message belongs to another verified connection.
    ResponseConnectionMismatch,
    /// The received response names a different command identifier than the witness.
    ResponseCommandMismatch,
    /// The existing typed response boundary rejected the ACK.
    Response {
        /// Exact typed response-admission failure.
        source: WebDriverBiDiTypeTextResponseError,
    },
}

impl fmt::Display for WebDriverBiDiTypeTextIntentAcknowledgementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ResponseConnectionMismatch => {
                "WebDriver BiDi text-input ACK arrived on a different connection than its intent"
            }
            Self::ResponseCommandMismatch => {
                "WebDriver BiDi text-input ACK does not match its sender-minted intent"
            }
            Self::Response { .. } => "WebDriver BiDi text-input intent acknowledgement failed",
        })
    }
}

impl Error for WebDriverBiDiTypeTextIntentAcknowledgementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Response { source } => Some(source),
            Self::ResponseConnectionMismatch | Self::ResponseCommandMismatch => None,
        }
    }
}

/// Dispatch one node-bound text action and mint the only witness that can later authorize its
/// text-value post-condition comparison.
///
/// The underlying sender performs all existing protocol-use, node/session/document authority,
/// correlation, deadline, and frame-write checks. A witness is returned only after that sender
/// succeeds. It is bound to the exact private connection generation and command id and retains the
/// validated non-secret text only in an opaque non-cloneable value. Command ACK still is not browser
/// success: callers must admit the exact ACK with [`acknowledge_webdriver_bidi_type_text_intent`]
/// and then consume the acknowledged intent in the text-value post-condition boundary.
#[expect(
    clippy::too_many_arguments,
    reason = "this immediate-use wrapper preserves the typed sender's explicit authority, transport, correlation, masking, and deadline inputs while adding only a one-shot post-condition intent witness"
)]
pub fn send_webdriver_bidi_type_text_with_postcondition_intent(
    validated: ValidatedBrowserProtocolUse,
    command_id: u64,
    browsing_context: &str,
    text: &str,
    handle: &AdmittedNodeHandle,
    node: &WebDriverBiDiRemoteNodeReference,
    registry: &BrowserAuthorityRegistry,
    established: WebDriverBiDiWebSocketEstablished,
    correlation: &mut WebDriverBiDiCommandCorrelation,
    masking_key: WebDriverBiDiWebSocketMaskKey,
    frame_timeout: Duration,
) -> Result<
    (
        WebDriverBiDiWebSocketEstablished,
        WebDriverBiDiTypeTextIntentWitness,
    ),
    WebDriverBiDiTypeTextSendError,
> {
    let connection_generation = established.transport_evidence().connection_generation();
    let established = send_webdriver_bidi_type_text(
        validated,
        command_id,
        browsing_context,
        text,
        handle,
        node,
        registry,
        established,
        correlation,
        masking_key,
        frame_timeout,
    )?;

    Ok((
        established,
        WebDriverBiDiTypeTextIntentWitness {
            command_id,
            connection_generation,
            expected_text: text.into(),
        },
    ))
}

/// Admit one typed-input protocol ACK and consume the exact sender-minted intent witness.
///
/// Connection and response-id checks run before typed response correlation can consume pending
/// state. A foreign connection or a response naming another command therefore cannot retire an
/// unrelated action. Successful admission returns a one-shot acknowledged intent whose private text
/// is the original value passed to the reviewed sender, not a value supplied at verification time.
pub fn acknowledge_webdriver_bidi_type_text_intent(
    message: &WebDriverBiDiReceivedTextMessage,
    witness: WebDriverBiDiTypeTextIntentWitness,
    correlation: &mut WebDriverBiDiCommandCorrelation,
) -> Result<WebDriverBiDiAcknowledgedTypeTextIntent, WebDriverBiDiTypeTextIntentAcknowledgementError>
{
    if message.connection_generation() != witness.connection_generation {
        return Err(WebDriverBiDiTypeTextIntentAcknowledgementError::ResponseConnectionMismatch);
    }

    let envelope = WebDriverBiDiJsonEnvelope::parse(message.message()).map_err(|source| {
        WebDriverBiDiTypeTextIntentAcknowledgementError::Response {
            source: WebDriverBiDiTypeTextResponseError::Envelope { source },
        }
    })?;
    let response_command_id = match envelope.routing() {
        WebDriverBiDiJsonEnvelopeRouting::CommandSuccess { command_id }
        | WebDriverBiDiJsonEnvelopeRouting::CommandError {
            command_id: Some(command_id),
        } => Some(command_id),
        WebDriverBiDiJsonEnvelopeRouting::Event
        | WebDriverBiDiJsonEnvelopeRouting::CommandError { command_id: None } => None,
    };
    if response_command_id.is_some_and(|command_id| command_id != witness.command_id) {
        return Err(WebDriverBiDiTypeTextIntentAcknowledgementError::ResponseCommandMismatch);
    }

    let result = WebDriverBiDiTypeTextResult::parse_and_correlate(message, correlation).map_err(
        |source| WebDriverBiDiTypeTextIntentAcknowledgementError::Response { source },
    )?;
    if result.command_id() != witness.command_id {
        return Err(WebDriverBiDiTypeTextIntentAcknowledgementError::ResponseCommandMismatch);
    }

    Ok(WebDriverBiDiAcknowledgedTypeTextIntent {
        command_id: witness.command_id,
        expected_text: witness.expected_text,
    })
}
