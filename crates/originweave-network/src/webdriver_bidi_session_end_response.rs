use std::{error::Error, fmt};

use crate::{
    webdriver_bidi_connection::WebDriverBiDiConnectionGeneration, WebDriverBiDiCommandCorrelation,
    WebDriverBiDiCommandCorrelationError, WebDriverBiDiCommandKind,
    WebDriverBiDiCorrelatedResponseOutcome, WebDriverBiDiJsonEnvelope,
    WebDriverBiDiJsonEnvelopeError, WebDriverBiDiReceivedTextMessage,
};

/// Typed protocol acknowledgment for one correlated WebDriver BiDi `session.end` command.
///
/// WebDriver BiDi defines `session.EndResult` as the extensible `EmptyResult` object. The common
/// local-end envelope parser already validates the complete JSON document and requires a success
/// `result` object, so this command-specific boundary intentionally retains no generic result body
/// and accepts extension members. The result retains the private process-local generation of the
/// exact connection on which the command was registered before I/O, and response admission requires
/// the complete received message to carry the same non-forgeable generation. This prevents another
/// verified connection from acknowledging the command even when both reuse the same WebDriver
/// session and command id. This value proves only a correlated protocol success; it does not prove
/// Chromium process exit, profile deletion, resource release, or any other OriginWeave operational
/// teardown postcondition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiSessionEndResult {
    command_id: u64,
    connection_generation: WebDriverBiDiConnectionGeneration,
}

impl WebDriverBiDiSessionEndResult {
    /// Parse one connection-bound local-end message and consume its exact outstanding command.
    ///
    /// Complete JSON and common WebDriver BiDi envelope validation occur before correlation state
    /// can be consumed. The response must have been assembled by the connection-bound message reader
    /// on the same private transport generation registered by the typed `session.end` sender.
    /// Connection mismatch or missing command provenance fails without consuming the outstanding id.
    /// A correlatable protocol-error response on the correct connection consumes its matching id and
    /// returns a typed remote failure, while events, null-id errors, malformed envelopes, unknown ids,
    /// and command-kind mismatches leave unrelated outstanding state untouched.
    pub fn parse_and_correlate(
        message: &WebDriverBiDiReceivedTextMessage,
        correlation: &mut WebDriverBiDiCommandCorrelation,
    ) -> Result<Self, WebDriverBiDiSessionEndResponseError> {
        let envelope = WebDriverBiDiJsonEnvelope::parse(message.message())
            .map_err(|source| WebDriverBiDiSessionEndResponseError::Envelope { source })?;
        let completed = correlation
            .correlate_response_for_connection(
                &envelope,
                WebDriverBiDiCommandKind::SessionEnd,
                message.connection_generation(),
            )
            .map_err(|source| match source {
                WebDriverBiDiCommandCorrelationError::CommandConnectionProvenanceMissing {
                    command_id,
                } => WebDriverBiDiSessionEndResponseError::MissingConnectionProvenance {
                    command_id,
                },
                WebDriverBiDiCommandCorrelationError::ResponseConnectionMismatch { command_id } => {
                    WebDriverBiDiSessionEndResponseError::TransportConnectionMismatch {
                        command_id,
                    }
                }
                source => WebDriverBiDiSessionEndResponseError::Correlation { source },
            })?;

        match completed.outcome() {
            WebDriverBiDiCorrelatedResponseOutcome::Success => Ok(Self {
                command_id: completed.command_id(),
                connection_generation: message.connection_generation(),
            }),
            WebDriverBiDiCorrelatedResponseOutcome::Error => {
                Err(WebDriverBiDiSessionEndResponseError::RemoteProtocolError {
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

    pub(crate) const fn connection_generation(&self) -> WebDriverBiDiConnectionGeneration {
        self.connection_generation
    }
}

/// Fail-closed failures while admitting one typed WebDriver BiDi `session.end` response.
#[derive(Debug)]
pub enum WebDriverBiDiSessionEndResponseError {
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
    /// The outstanding `session.end` command lacked connection provenance required for admission.
    MissingConnectionProvenance {
        /// Exact local command identifier left outstanding after rejection.
        command_id: u64,
    },
    /// The response was received on a different verified transport from the outstanding command.
    TransportConnectionMismatch {
        /// Exact local command identifier left outstanding after rejection.
        command_id: u64,
    },
    /// The remote end returned a correlatable WebDriver BiDi protocol error for this command.
    RemoteProtocolError {
        /// Exact local command identifier consumed by the protocol-error response.
        command_id: u64,
    },
}

impl fmt::Display for WebDriverBiDiSessionEndResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Envelope { .. } => {
                formatter.write_str("WebDriver BiDi session.end envelope is invalid")
            }
            Self::Correlation { .. } => {
                formatter.write_str("WebDriver BiDi session.end response correlation failed")
            }
            Self::MissingConnectionProvenance { .. } => formatter
                .write_str("WebDriver BiDi session.end response lacks connection provenance"),
            Self::TransportConnectionMismatch { .. } => formatter.write_str(
                "WebDriver BiDi session.end response arrived on a different connection",
            ),
            Self::RemoteProtocolError { .. } => {
                formatter.write_str("WebDriver BiDi session.end returned a protocol error")
            }
        }
    }
}

impl Error for WebDriverBiDiSessionEndResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Envelope { source } => Some(source),
            Self::Correlation { source } => Some(source),
            Self::MissingConnectionProvenance { .. }
            | Self::TransportConnectionMismatch { .. }
            | Self::RemoteProtocolError { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_errors_have_stable_messages_and_typed_sources() {
        let envelope = WebDriverBiDiSessionEndResponseError::Envelope {
            source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
        };
        assert_eq!(
            envelope.to_string(),
            "WebDriver BiDi session.end envelope is invalid"
        );
        assert!(envelope.source().is_some());

        let correlation = WebDriverBiDiSessionEndResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
        };
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi session.end response correlation failed"
        );
        assert!(correlation.source().is_some());

        let missing = WebDriverBiDiSessionEndResponseError::MissingConnectionProvenance {
            command_id: 7,
        };
        assert_eq!(
            missing.to_string(),
            "WebDriver BiDi session.end response lacks connection provenance"
        );
        assert!(missing.source().is_none());

        let mismatch = WebDriverBiDiSessionEndResponseError::TransportConnectionMismatch {
            command_id: 7,
        };
        assert_eq!(
            mismatch.to_string(),
            "WebDriver BiDi session.end response arrived on a different connection"
        );
        assert!(mismatch.source().is_none());

        let remote = WebDriverBiDiSessionEndResponseError::RemoteProtocolError { command_id: 7 };
        assert_eq!(
            remote.to_string(),
            "WebDriver BiDi session.end returned a protocol error"
        );
        assert!(remote.source().is_none());
    }
}
