use std::{error::Error, fmt};

use crate::{
    WebDriverBiDiAcknowledgedTypeTextIntent, WebDriverBiDiCommandCorrelation,
    WebDriverBiDiReceivedTextMessage, WebDriverBiDiTextValueObservationResponseError,
    WebDriverBiDiTextValueObservationResult,
};

/// Credential-minimal proof that one exact correlated text observation matched the exact
/// sender-minted and acknowledged typed-input intent that preceded it.
///
/// The page-controlled string is discarded by the lower observation boundary before this value is
/// constructed. This type therefore carries only the typed-input command identifier, the consumed
/// observation command identifier, and observed byte count. A caller can obtain this value only
/// after the original one-shot typed-input intent received its exact protocol ACK and the later
/// observation exactly matched that retained intent on the same verified WebDriver BiDi connection;
/// a command ACK, parser success, same textual value on another connection, or verification-time
/// caller value is not sufficient post-condition evidence.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WebDriverBiDiTextValuePostcondition {
    type_text_command_id: u64,
    command_id: u64,
    observed_text_bytes: usize,
}

impl fmt::Debug for WebDriverBiDiTextValuePostcondition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiTextValuePostcondition")
            .field("type_text_command_id", &self.type_text_command_id)
            .field("command_id", &self.command_id)
            .field("observed_text_bytes", &self.observed_text_bytes)
            .finish()
    }
}

impl WebDriverBiDiTextValuePostcondition {
    /// Return the exact local typed-input command identifier whose acknowledged intent was verified.
    #[must_use]
    pub const fn type_text_command_id(&self) -> u64 {
        self.type_text_command_id
    }

    /// Return the exact local command identifier consumed by the verified observation response.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return the UTF-8 byte count of the matched page value without retaining that value.
    #[must_use]
    pub const fn observed_text_bytes(&self) -> usize {
        self.observed_text_bytes
    }
}

/// Failure to produce positive text-value post-condition evidence from one correlated response.
#[derive(Debug)]
pub enum WebDriverBiDiTextValuePostconditionError {
    /// The received observation belongs to a different verified connection than the acknowledged
    /// typed-input intent. This check runs before observation correlation can consume pending state.
    ObservationConnectionMismatch,
    /// The underlying bounded response admission or correlation failed.
    Observation {
        /// Exact typed lower-boundary failure.
        source: WebDriverBiDiTextValueObservationResponseError,
    },
    /// The response was structurally valid and correlated, but the observed page value differed
    /// from the exact sender-minted typed-input intent.
    PostconditionMismatch {
        /// Exact acknowledged typed-input command whose retained intent was compared.
        type_text_command_id: u64,
        /// Exact local observation command identifier consumed by the negative observation.
        command_id: u64,
        /// UTF-8 byte count of the mismatched page value; the page text itself is not retained.
        observed_text_bytes: usize,
    },
}

impl fmt::Display for WebDriverBiDiTextValuePostconditionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ObservationConnectionMismatch => formatter.write_str(
                "WebDriver BiDi text-value postcondition observation arrived on a different connection than the acknowledged typed-input intent",
            ),
            Self::Observation { .. } => {
                formatter.write_str("WebDriver BiDi text-value postcondition observation failed")
            }
            Self::PostconditionMismatch { .. } => formatter.write_str(
                "WebDriver BiDi text-value postcondition did not match the acknowledged typed-input intent",
            ),
        }
    }
}

impl Error for WebDriverBiDiTextValuePostconditionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Observation { source } => Some(source),
            Self::ObservationConnectionMismatch | Self::PostconditionMismatch { .. } => None,
        }
    }
}

/// Admit one bounded correlated text observation and return success only when its page value
/// exactly matches the sender-minted typed-input intent that already received its exact ACK.
///
/// The caller must transfer a one-shot [`WebDriverBiDiAcknowledgedTypeTextIntent`]. That value is
/// produced only from the reviewed typed-input sender and its exact connection-bound protocol ACK,
/// so the expected value can no longer be selected at verification time. The received observation
/// must first match the acknowledged intent's private connection generation; that check happens
/// before lower response admission or correlation can consume pending state. The lower observation
/// boundary then validates response structure, script result shape, and exact observation-command
/// correlation before comparison. A mismatching observation consumes its correlated observation
/// command because the response is complete, but returns a typed negative result rather than `Ok`.
///
/// No page-controlled text, expected text, connection-generation identifier, realm identifier,
/// credential, secret, browser authority, or policy authority is retained in the returned value or
/// error diagnostics. The acknowledged intent is consumed exactly once by this call and its private
/// text and connection generation are dropped afterward.
pub fn verify_webdriver_bidi_text_value_postcondition(
    message: &WebDriverBiDiReceivedTextMessage,
    acknowledged_intent: WebDriverBiDiAcknowledgedTypeTextIntent,
    correlation: &mut WebDriverBiDiCommandCorrelation,
) -> Result<WebDriverBiDiTextValuePostcondition, WebDriverBiDiTextValuePostconditionError> {
    if !acknowledged_intent.matches_connection_generation(message.connection_generation()) {
        return Err(WebDriverBiDiTextValuePostconditionError::ObservationConnectionMismatch);
    }

    let type_text_command_id = acknowledged_intent.command_id();
    let observation = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        message,
        acknowledged_intent.expected_text(),
        correlation,
    )
    .map_err(|source| WebDriverBiDiTextValuePostconditionError::Observation { source })?;

    if !observation.matches_expected_text() {
        return Err(
            WebDriverBiDiTextValuePostconditionError::PostconditionMismatch {
                type_text_command_id,
                command_id: observation.command_id(),
                observed_text_bytes: observation.observed_text_bytes(),
            },
        );
    }

    Ok(WebDriverBiDiTextValuePostcondition {
        type_text_command_id,
        command_id: observation.command_id(),
        observed_text_bytes: observation.observed_text_bytes(),
    })
}
