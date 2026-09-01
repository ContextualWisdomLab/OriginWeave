use std::{error::Error, fmt};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCommandKind, WebDriverBiDiCorrelatedResponseOutcome,
    WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError,
    WebDriverBiDiWebSocketTextMessage,
};

/// Typed protocol acknowledgment for one correlated WebDriver BiDi `input.performActions`
/// primary-button pointer-click command.
///
/// WebDriver BiDi defines `input.PerformActionsResult` as the extensible `EmptyResult` object. The
/// common local-end envelope parser already validates the complete JSON document and requires a
/// success `result` object, so this command-specific boundary intentionally retains no generic
/// result body and accepts extension members. This value proves only that the remote end returned a
/// correlated protocol success; it does not prove target activation, DOM mutation, navigation, or
/// any other OriginWeave post-condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiPointerClickResult {
    command_id: u64,
}

impl WebDriverBiDiPointerClickResult {
    /// Parse one bounded local-end message and consume its exact outstanding pointer-click command.
    ///
    /// Complete JSON and common WebDriver BiDi envelope validation occur before correlation state
    /// can be consumed. Successful responses retain only the matched command id. A correlatable
    /// protocol-error response consumes its matching pointer-click id and returns a typed remote
    /// failure. A response for another typed command family, an event, a null-id error, a malformed
    /// envelope, or an unknown id fails closed without consuming unrelated outstanding state.
    pub fn parse_and_correlate(
        message: &WebDriverBiDiWebSocketTextMessage,
        correlation: &mut WebDriverBiDiCommandCorrelation,
    ) -> Result<Self, WebDriverBiDiPointerClickResponseError> {
        let envelope = WebDriverBiDiJsonEnvelope::parse(message)
            .map_err(|source| WebDriverBiDiPointerClickResponseError::Envelope { source })?;
        let completed = correlation
            .correlate_response_for(&envelope, WebDriverBiDiCommandKind::PointerClick)
            .map_err(|source| WebDriverBiDiPointerClickResponseError::Correlation { source })?;

        match completed.outcome() {
            WebDriverBiDiCorrelatedResponseOutcome::Success => Ok(Self {
                command_id: completed.command_id(),
            }),
            WebDriverBiDiCorrelatedResponseOutcome::Error => Err(
                WebDriverBiDiPointerClickResponseError::RemoteProtocolError {
                    command_id: completed.command_id(),
                },
            ),
        }
    }

    /// Return the exact local command identifier consumed by this protocol acknowledgment.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }
}

/// Fail-closed failures while admitting one typed WebDriver BiDi pointer-click response.
#[derive(Debug)]
pub enum WebDriverBiDiPointerClickResponseError {
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

impl fmt::Display for WebDriverBiDiPointerClickResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Envelope { .. } => {
                formatter.write_str("WebDriver BiDi pointer-click envelope is invalid")
            }
            Self::Correlation { .. } => {
                formatter.write_str("WebDriver BiDi pointer-click response correlation failed")
            }
            Self::RemoteProtocolError { .. } => {
                formatter.write_str("WebDriver BiDi pointer-click returned a protocol error")
            }
        }
    }
}

impl Error for WebDriverBiDiPointerClickResponseError {
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
        let envelope = WebDriverBiDiPointerClickResponseError::Envelope {
            source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
        };
        assert_eq!(
            envelope.to_string(),
            "WebDriver BiDi pointer-click envelope is invalid"
        );
        assert!(envelope.source().is_some());

        let correlation = WebDriverBiDiPointerClickResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
        };
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi pointer-click response correlation failed"
        );
        assert!(correlation.source().is_some());

        let remote = WebDriverBiDiPointerClickResponseError::RemoteProtocolError { command_id: 42 };
        assert_eq!(
            remote.to_string(),
            "WebDriver BiDi pointer-click returned a protocol error"
        );
        assert!(remote.source().is_none());
    }
}
