use std::{collections::BTreeSet, error::Error, fmt};

use crate::{
    MAX_WEBDRIVER_BIDI_JS_UINT, WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeKind,
};

/// Maximum number of local WebDriver BiDi commands retained as outstanding at once.
///
/// WebDriver BiDi permits commands to complete out of order. OriginWeave therefore keeps a
/// bounded local correlation set instead of assuming response order, while this resource ceiling
/// prevents an unbounded remote-control session from growing local correlation state indefinitely.
pub const MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS: usize = 256;

/// Outcome of a response after it has consumed the matching outstanding command identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiCorrelatedResponseOutcome {
    /// The remote end returned a successful command response.
    Success,
    /// The remote end returned a protocol error for the command.
    Error,
}

/// Credential-free evidence that one parsed response consumed one outstanding local command.
///
/// This value carries only the matched command identifier and success/error classification. It
/// does not retain result bodies, error text, browser authority, transport authority, or secrets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiCorrelatedResponse {
    command_id: u64,
    outcome: WebDriverBiDiCorrelatedResponseOutcome,
}

impl WebDriverBiDiCorrelatedResponse {
    /// Return the local command identifier consumed by this response.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return whether the correlated response was successful or a protocol error.
    #[must_use]
    pub const fn outcome(&self) -> WebDriverBiDiCorrelatedResponseOutcome {
        self.outcome
    }
}

/// Fail-closed command-correlation failures at the local WebDriver BiDi response boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiCommandCorrelationError {
    /// A caller attempted to register an identifier outside WebDriver BiDi's `js-uint` range.
    CommandIdOutOfRange,
    /// The identifier is already outstanding and cannot become ambiguous.
    CommandAlreadyOutstanding,
    /// The reviewed outstanding-command resource budget has been reached.
    OutstandingCommandLimit,
    /// No currently outstanding command matches the requested or returned identifier.
    CommandNotOutstanding,
    /// An event is not a command response and cannot consume correlation state.
    EventIsNotResponse,
    /// A protocol error with a `null` id cannot be attributed to one outstanding command.
    UncorrelatableErrorResponse,
}

impl fmt::Display for WebDriverBiDiCommandCorrelationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CommandIdOutOfRange => "WebDriver BiDi command id is outside the js-uint range",
            Self::CommandAlreadyOutstanding => "WebDriver BiDi command id is already outstanding",
            Self::OutstandingCommandLimit => "WebDriver BiDi outstanding-command limit reached",
            Self::CommandNotOutstanding => "WebDriver BiDi command id is not outstanding",
            Self::EventIsNotResponse => "WebDriver BiDi event cannot be correlated as a response",
            Self::UncorrelatableErrorResponse => {
                "WebDriver BiDi error response has no correlatable command id"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for WebDriverBiDiCommandCorrelationError {}

/// Bounded local WebDriver BiDi command-response correlation state.
///
/// Register an id only after the caller has committed to one outbound command. A success or
/// correlatable error response consumes the id exactly once. Events and null-id errors leave all
/// outstanding state untouched. This type performs no I/O, retry, command serialization, browser
/// authentication, or authority grant.
#[derive(Debug, Default)]
pub struct WebDriverBiDiCommandCorrelation {
    outstanding: BTreeSet<u64>,
}

impl WebDriverBiDiCommandCorrelation {
    /// Create empty correlation state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the number of commands currently awaiting a correlatable response.
    #[must_use]
    pub fn outstanding_count(&self) -> usize {
        self.outstanding.len()
    }

    /// Register one local command id before its response can be accepted.
    ///
    /// Identifiers are unique only while outstanding. A completed or explicitly retired id may be
    /// reused later, matching WebDriver BiDi's local-end correlation semantics.
    pub fn register_command(
        &mut self,
        command_id: u64,
    ) -> Result<(), WebDriverBiDiCommandCorrelationError> {
        if command_id > MAX_WEBDRIVER_BIDI_JS_UINT {
            return Err(WebDriverBiDiCommandCorrelationError::CommandIdOutOfRange);
        }
        if self.outstanding.contains(&command_id) {
            return Err(WebDriverBiDiCommandCorrelationError::CommandAlreadyOutstanding);
        }
        if self.outstanding.len() >= MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS {
            return Err(WebDriverBiDiCommandCorrelationError::OutstandingCommandLimit);
        }
        self.outstanding.insert(command_id);
        Ok(())
    }

    /// Explicitly retire one outstanding command without accepting a response for it.
    ///
    /// This supports caller-owned cancellation or session teardown without retaining stale ids.
    pub fn retire_command(
        &mut self,
        command_id: u64,
    ) -> Result<(), WebDriverBiDiCommandCorrelationError> {
        if self.outstanding.remove(&command_id) {
            Ok(())
        } else {
            Err(WebDriverBiDiCommandCorrelationError::CommandNotOutstanding)
        }
    }

    /// Correlate one already parsed local-end envelope with the outstanding command set.
    ///
    /// Successful responses and error responses with ids consume exactly one matching command.
    /// Unknown ids fail without consuming unrelated state. Events and null-id errors fail before
    /// touching the set.
    pub fn correlate_response(
        &mut self,
        envelope: &WebDriverBiDiJsonEnvelope,
    ) -> Result<WebDriverBiDiCorrelatedResponse, WebDriverBiDiCommandCorrelationError> {
        match envelope.kind() {
            WebDriverBiDiJsonEnvelopeKind::Event => {
                Err(WebDriverBiDiCommandCorrelationError::EventIsNotResponse)
            }
            WebDriverBiDiJsonEnvelopeKind::Error => {
                let Some(command_id) = envelope.command_id() else {
                    return Err(
                        WebDriverBiDiCommandCorrelationError::UncorrelatableErrorResponse,
                    );
                };
                self.complete(command_id, WebDriverBiDiCorrelatedResponseOutcome::Error)
            }
            WebDriverBiDiJsonEnvelopeKind::Success => self.complete(
                envelope
                    .command_id()
                    .unwrap_or(MAX_WEBDRIVER_BIDI_JS_UINT.saturating_add(1)),
                WebDriverBiDiCorrelatedResponseOutcome::Success,
            ),
        }
    }

    fn complete(
        &mut self,
        command_id: u64,
        outcome: WebDriverBiDiCorrelatedResponseOutcome,
    ) -> Result<WebDriverBiDiCorrelatedResponse, WebDriverBiDiCommandCorrelationError> {
        if !self.outstanding.remove(&command_id) {
            return Err(WebDriverBiDiCommandCorrelationError::CommandNotOutstanding);
        }
        Ok(WebDriverBiDiCorrelatedResponse {
            command_id,
            outcome,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::WebDriverBiDiCommandCorrelationError;

    #[test]
    fn correlation_errors_have_stable_nonempty_operator_messages() {
        let cases = [
            (
                WebDriverBiDiCommandCorrelationError::CommandIdOutOfRange,
                "WebDriver BiDi command id is outside the js-uint range",
            ),
            (
                WebDriverBiDiCommandCorrelationError::CommandAlreadyOutstanding,
                "WebDriver BiDi command id is already outstanding",
            ),
            (
                WebDriverBiDiCommandCorrelationError::OutstandingCommandLimit,
                "WebDriver BiDi outstanding-command limit reached",
            ),
            (
                WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
                "WebDriver BiDi command id is not outstanding",
            ),
            (
                WebDriverBiDiCommandCorrelationError::EventIsNotResponse,
                "WebDriver BiDi event cannot be correlated as a response",
            ),
            (
                WebDriverBiDiCommandCorrelationError::UncorrelatableErrorResponse,
                "WebDriver BiDi error response has no correlatable command id",
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }
}
