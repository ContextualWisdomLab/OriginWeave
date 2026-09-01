use std::{error::Error, fmt};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiTextValueObservationResponseError,
    WebDriverBiDiTextValueObservationResult, WebDriverBiDiWebSocketTextMessage,
};

/// Credential-minimal proof that one exact correlated text observation matched the authorized
/// expected value.
///
/// The page-controlled string is discarded by the lower observation boundary before this value is
/// constructed. This type therefore carries only the consumed command identifier and observed byte
/// count. A caller can obtain this value only after exact equality succeeds; a mere command
/// response or successful parser result is not sufficient post-condition evidence.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WebDriverBiDiTextValuePostcondition {
    command_id: u64,
    observed_text_bytes: usize,
}

impl fmt::Debug for WebDriverBiDiTextValuePostcondition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiTextValuePostcondition")
            .field("command_id", &self.command_id)
            .field("observed_text_bytes", &self.observed_text_bytes)
            .finish()
    }
}

impl WebDriverBiDiTextValuePostcondition {
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
    /// The underlying bounded response admission or correlation failed.
    Observation {
        /// Exact typed lower-boundary failure.
        source: WebDriverBiDiTextValueObservationResponseError,
    },
    /// The response was structurally valid and correlated, but the observed page value differed
    /// from the exact already-authorized expected text.
    PostconditionMismatch {
        /// Exact local command identifier consumed by the negative observation.
        command_id: u64,
        /// UTF-8 byte count of the mismatched page value; the page text itself is not retained.
        observed_text_bytes: usize,
    },
}

impl fmt::Display for WebDriverBiDiTextValuePostconditionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Observation { .. } => {
                formatter.write_str("WebDriver BiDi text-value postcondition observation failed")
            }
            Self::PostconditionMismatch { .. } => formatter.write_str(
                "WebDriver BiDi text-value postcondition did not match the authorized expected text",
            ),
        }
    }
}

impl Error for WebDriverBiDiTextValuePostconditionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Observation { source } => Some(source),
            Self::PostconditionMismatch { .. } => None,
        }
    }
}

/// Admit one bounded correlated text observation and return success only when its page value
/// exactly matches the already-authorized expected text.
///
/// The lower boundary validates expected-text policy, response structure, script result shape, and
/// exact command correlation before this function evaluates the post-condition. A mismatching
/// observation consumes its correlated command because the response is complete, but returns a
/// typed negative result rather than `Ok`. This prevents command acknowledgment, parser success, or
/// correlation success from being mistaken for successful browser state mutation.
///
/// No page-controlled text, expected text, realm identifier, credential, secret, browser authority,
/// or policy authority is retained in the returned value or error diagnostics.
pub fn verify_webdriver_bidi_text_value_postcondition(
    message: &WebDriverBiDiWebSocketTextMessage,
    expected_text: &str,
    correlation: &mut WebDriverBiDiCommandCorrelation,
) -> Result<WebDriverBiDiTextValuePostcondition, WebDriverBiDiTextValuePostconditionError> {
    let observation = WebDriverBiDiTextValueObservationResult::parse_correlate_and_compare(
        message,
        expected_text,
        correlation,
    )
    .map_err(|source| WebDriverBiDiTextValuePostconditionError::Observation { source })?;

    if !observation.matches_expected_text() {
        return Err(
            WebDriverBiDiTextValuePostconditionError::PostconditionMismatch {
                command_id: observation.command_id(),
                observed_text_bytes: observation.observed_text_bytes(),
            },
        );
    }

    Ok(WebDriverBiDiTextValuePostcondition {
        command_id: observation.command_id(),
        observed_text_bytes: observation.observed_text_bytes(),
    })
}
