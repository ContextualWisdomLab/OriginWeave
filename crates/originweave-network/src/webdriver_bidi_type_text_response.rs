use std::{error::Error, fmt};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCorrelatedResponseOutcome, WebDriverBiDiJsonEnvelope,
    WebDriverBiDiJsonEnvelopeError, WebDriverBiDiWebSocketTextMessage,
};

/// Typed protocol acknowledgment for one correlated WebDriver BiDi `input.performActions`
/// node-bound text-input command.
///
/// The WebDriver BiDi `input.performActions` command returns the extensible `EmptyResult` shape.
/// The common local-end envelope parser already validates the complete JSON document and requires a
/// success `result` object, so this command-specific boundary intentionally retains no generic
/// result body and accepts extension members. This value proves only that the remote end returned a
/// correlated protocol success; it does not prove that the target received text, that a DOM or
/// accessibility state changed, or that any other OriginWeave post-condition was observed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiTypeTextResult {
    command_id: u64,
}

impl WebDriverBiDiTypeTextResult {
    /// Parse one bounded local-end message and consume its exact outstanding command on response.
    ///
    /// Complete JSON and common WebDriver BiDi envelope validation occur before correlation state
    /// can be consumed. Successful responses retain only the matched command id. A correlatable
    /// protocol-error response consumes its matching id and returns a typed remote failure, while
    /// events, null-id errors, malformed envelopes, and unknown ids fail closed without consuming
    /// unrelated outstanding state.
    pub fn parse_and_correlate(
        message: &WebDriverBiDiWebSocketTextMessage,
        correlation: &mut WebDriverBiDiCommandCorrelation,
    ) -> Result<Self, WebDriverBiDiTypeTextResponseError> {
        let envelope = WebDriverBiDiJsonEnvelope::parse(message)
            .map_err(|source| WebDriverBiDiTypeTextResponseError::Envelope { source })?;
        let completed = correlation
            .correlate_response(&envelope)
            .map_err(|source| WebDriverBiDiTypeTextResponseError::Correlation { source })?;

        match completed.outcome() {
            WebDriverBiDiCorrelatedResponseOutcome::Success => Ok(Self {
                command_id: completed.command_id(),
            }),
            WebDriverBiDiCorrelatedResponseOutcome::Error => {
                Err(WebDriverBiDiTypeTextResponseError::RemoteProtocolError {
                    command_id: completed.command_id(),
                })
            }
        }
    }

    /// Return the exact local command identifier consumed by this protocol acknowledgment.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }
}

/// Fail-closed failures while admitting one typed WebDriver BiDi text-input response.
#[derive(Debug)]
pub enum WebDriverBiDiTypeTextResponseError {
    /// Common local-end JSON envelope validation failed before correlation state was touched.
    Envelope {
        /// Exact common-envelope validation failure.
        source: WebDriverBiDiJsonEnvelopeError,
    },
    /// Exact command-response correlation failed without consuming unrelated state.
    Correlation {
        /// Exact typed correlation failure.
        source: WebDriverBiDiCommandCorrelationError,
    },
    /// The remote end returned a correlatable WebDriver BiDi protocol error for this command.
    RemoteProtocolError {
        /// Exact local command identifier consumed by the protocol-error response.
        command_id: u64,
    },
}

impl fmt::Display for WebDriverBiDiTypeTextResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Envelope { .. } => {
                formatter.write_str("WebDriver BiDi text-input envelope is invalid")
            }
            Self::Correlation { .. } => {
                formatter.write_str("WebDriver BiDi text-input response correlation failed")
            }
            Self::RemoteProtocolError { .. } => {
                formatter.write_str("WebDriver BiDi text-input returned a protocol error")
            }
        }
    }
}

impl Error for WebDriverBiDiTypeTextResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Envelope { source } => Some(source),
            Self::Correlation { source } => Some(source),
            Self::RemoteProtocolError { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_errors_have_stable_messages_and_typed_sources() {
        let envelope = WebDriverBiDiTypeTextResponseError::Envelope {
            source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
        };
        assert_eq!(
            envelope.to_string(),
            "WebDriver BiDi text-input envelope is invalid"
        );
        assert!(envelope.source().is_some());

        let correlation = WebDriverBiDiTypeTextResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
        };
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi text-input response correlation failed"
        );
        assert!(correlation.source().is_some());

        let remote = WebDriverBiDiTypeTextResponseError::RemoteProtocolError { command_id: 42 };
        assert_eq!(
            remote.to_string(),
            "WebDriver BiDi text-input returned a protocol error"
        );
        assert!(remote.source().is_none());
    }
}
