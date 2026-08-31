use std::{error::Error, fmt};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCorrelatedResponseOutcome, WebDriverBiDiJsonEnvelope,
    WebDriverBiDiJsonEnvelopeError, WebDriverBiDiWebSocketTextMessage,
};

/// Typed protocol acknowledgment for one correlated WebDriver BiDi `session.unsubscribe` command.
///
/// WebDriver BiDi defines `session.UnsubscribeResult` as the extensible `EmptyResult` object. The
/// common local-end envelope parser validates the complete JSON document and requires a success
/// result object, so this boundary retains only the matched command identifier. A successful value
/// proves protocol acknowledgment only; it does not itself prove that no already-in-flight event can
/// arrive or grant any replacement browser, policy, origin, secret, or Agent authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiNavigationCommittedUnsubscribeResult {
    command_id: u64,
}

impl WebDriverBiDiNavigationCommittedUnsubscribeResult {
    /// Parse one bounded local-end message and consume its exact outstanding command on response.
    ///
    /// Complete JSON and common WebDriver BiDi envelope validation occur before correlation state
    /// can be consumed. A correlatable protocol-error response consumes its matching identifier and
    /// returns a typed remote failure. Events, null-id errors, malformed envelopes, and unknown ids
    /// fail closed without consuming unrelated outstanding command state.
    pub fn parse_and_correlate(
        message: &WebDriverBiDiWebSocketTextMessage,
        correlation: &mut WebDriverBiDiCommandCorrelation,
    ) -> Result<Self, WebDriverBiDiNavigationCommittedUnsubscribeResponseError> {
        let envelope = WebDriverBiDiJsonEnvelope::parse(message).map_err(|source| {
            WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Envelope { source }
        })?;
        let completed = correlation
            .correlate_response(&envelope)
            .map_err(|source| {
                WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Correlation { source }
            })?;

        match completed.outcome() {
            WebDriverBiDiCorrelatedResponseOutcome::Success => Ok(Self {
                command_id: completed.command_id(),
            }),
            WebDriverBiDiCorrelatedResponseOutcome::Error => Err(
                WebDriverBiDiNavigationCommittedUnsubscribeResponseError::RemoteProtocolError {
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

/// Fail-closed failures while admitting one typed WebDriver BiDi `session.unsubscribe` response.
#[derive(Debug)]
pub enum WebDriverBiDiNavigationCommittedUnsubscribeResponseError {
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

impl fmt::Display for WebDriverBiDiNavigationCommittedUnsubscribeResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Envelope { .. } => {
                formatter.write_str("WebDriver BiDi session.unsubscribe envelope is invalid")
            }
            Self::Correlation { .. } => formatter
                .write_str("WebDriver BiDi session.unsubscribe response correlation failed"),
            Self::RemoteProtocolError { .. } => {
                formatter.write_str("WebDriver BiDi session.unsubscribe returned a protocol error")
            }
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedUnsubscribeResponseError {
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
        let envelope = WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Envelope {
            source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
        };
        assert_eq!(
            envelope.to_string(),
            "WebDriver BiDi session.unsubscribe envelope is invalid"
        );
        assert!(envelope.source().is_some());

        let correlation = WebDriverBiDiNavigationCommittedUnsubscribeResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
        };
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi session.unsubscribe response correlation failed"
        );
        assert!(correlation.source().is_some());

        let remote =
            WebDriverBiDiNavigationCommittedUnsubscribeResponseError::RemoteProtocolError {
                command_id: 8,
            };
        assert_eq!(
            remote.to_string(),
            "WebDriver BiDi session.unsubscribe returned a protocol error"
        );
        assert!(remote.source().is_none());
    }
}
