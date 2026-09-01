use std::{collections::BTreeMap, error::Error, fmt};

use crate::{MAX_WEBDRIVER_BIDI_JS_UINT, WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeKind};

/// Maximum number of local WebDriver BiDi commands retained as outstanding at once.
///
/// WebDriver BiDi permits commands to complete out of order. OriginWeave therefore keeps a
/// bounded local correlation map instead of assuming response order, while this resource ceiling
/// prevents an unbounded remote-control session from growing local correlation state indefinitely.
pub const MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS: usize = 256;

/// Exact WebDriver BiDi command family bound to one outstanding local correlation identifier.
///
/// Command identifiers are local-end routing values rather than command-type provenance. Keeping
/// the reviewed command family beside each outstanding id prevents a success or protocol error for
/// one command from being consumed by a different typed response boundary that happens to receive
/// the same id. Additional command families are introduced by their owning typed command slices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiCommandKind {
    /// WebDriver BiDi `session.status`.
    SessionStatus,
    /// WebDriver BiDi `session.end`.
    SessionEnd,
    /// WebDriver BiDi `input.performActions` pointer click.
    PointerClick,
}

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
    /// The typed consumer does not match the command family registered for this identifier.
    CommandKindMismatch {
        /// Command family required by the typed consumer.
        expected: WebDriverBiDiCommandKind,
        /// Command family actually registered for the outstanding identifier.
        actual: WebDriverBiDiCommandKind,
    },
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
            Self::CommandKindMismatch { .. } => {
                "WebDriver BiDi response command kind does not match the outstanding command"
            }
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
/// Register an id together with its exact typed command family only after the caller has committed
/// to that outbound command. A success or correlatable error response consumes the id exactly once
/// only through a matching typed consumer. Events, null-id errors, and command-kind mismatches leave
/// outstanding state untouched. This type performs no I/O, retry, command serialization, browser
/// authentication, or authority grant. Debug output reports only the outstanding-count summary;
/// command identifiers and command families remain private correlation state.
#[derive(Default)]
pub struct WebDriverBiDiCommandCorrelation {
    outstanding: BTreeMap<u64, WebDriverBiDiCommandKind>,
}

impl fmt::Debug for WebDriverBiDiCommandCorrelation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiCommandCorrelation")
            .field("outstanding_count", &self.outstanding.len())
            .finish()
    }
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

    /// Register one local command id and its exact command family before its response can be accepted.
    ///
    /// Identifiers are unique only while outstanding. A completed or explicitly retired id may be
    /// reused later, matching WebDriver BiDi's local-end correlation semantics. Reusing an id while
    /// any command family is still outstanding fails before replacing its provenance.
    pub fn register_command_for(
        &mut self,
        command_id: u64,
        command_kind: WebDriverBiDiCommandKind,
    ) -> Result<(), WebDriverBiDiCommandCorrelationError> {
        if command_id > MAX_WEBDRIVER_BIDI_JS_UINT {
            return Err(WebDriverBiDiCommandCorrelationError::CommandIdOutOfRange);
        }
        if self.outstanding.contains_key(&command_id) {
            return Err(WebDriverBiDiCommandCorrelationError::CommandAlreadyOutstanding);
        }
        if self.outstanding.len() >= MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS {
            return Err(WebDriverBiDiCommandCorrelationError::OutstandingCommandLimit);
        }
        let _previous = self.outstanding.insert(command_id, command_kind);
        Ok(())
    }

    /// Explicitly retire one exact outstanding command without accepting a response for it.
    ///
    /// The expected command family must match the registered provenance. A mismatched caller cannot
    /// retire another typed command merely by knowing or reusing its local correlation identifier.
    pub fn retire_command_for(
        &mut self,
        command_id: u64,
        expected_kind: WebDriverBiDiCommandKind,
    ) -> Result<(), WebDriverBiDiCommandCorrelationError> {
        self.require_command_kind(command_id, expected_kind)?;
        let _removed = self.outstanding.remove(&command_id);
        Ok(())
    }

    /// Correlate one parsed local-end envelope with an exact outstanding command family.
    ///
    /// Successful responses and error responses with ids consume exactly one matching command.
    /// Unknown ids and command-kind mismatches fail without consuming state. Events and null-id
    /// errors fail before touching the map.
    pub fn correlate_response_for(
        &mut self,
        envelope: &WebDriverBiDiJsonEnvelope,
        expected_kind: WebDriverBiDiCommandKind,
    ) -> Result<WebDriverBiDiCorrelatedResponse, WebDriverBiDiCommandCorrelationError> {
        match envelope.kind() {
            WebDriverBiDiJsonEnvelopeKind::Event => {
                Err(WebDriverBiDiCommandCorrelationError::EventIsNotResponse)
            }
            WebDriverBiDiJsonEnvelopeKind::Error => {
                let Some(command_id) = envelope.command_id() else {
                    return Err(WebDriverBiDiCommandCorrelationError::UncorrelatableErrorResponse);
                };
                self.complete(
                    command_id,
                    expected_kind,
                    WebDriverBiDiCorrelatedResponseOutcome::Error,
                )
            }
            WebDriverBiDiJsonEnvelopeKind::Success => {
                let Some(command_id) = envelope.command_id() else {
                    return Err(WebDriverBiDiCommandCorrelationError::CommandNotOutstanding);
                };
                self.complete(
                    command_id,
                    expected_kind,
                    WebDriverBiDiCorrelatedResponseOutcome::Success,
                )
            }
        }
    }

    fn require_command_kind(
        &self,
        command_id: u64,
        expected_kind: WebDriverBiDiCommandKind,
    ) -> Result<(), WebDriverBiDiCommandCorrelationError> {
        let actual = self
            .outstanding
            .get(&command_id)
            .copied()
            .ok_or(WebDriverBiDiCommandCorrelationError::CommandNotOutstanding)?;
        if actual != expected_kind {
            return Err(WebDriverBiDiCommandCorrelationError::CommandKindMismatch {
                expected: expected_kind,
                actual,
            });
        }
        Ok(())
    }

    fn complete(
        &mut self,
        command_id: u64,
        expected_kind: WebDriverBiDiCommandKind,
        outcome: WebDriverBiDiCorrelatedResponseOutcome,
    ) -> Result<WebDriverBiDiCorrelatedResponse, WebDriverBiDiCommandCorrelationError> {
        self.require_command_kind(command_id, expected_kind)?;
        let _removed = self.outstanding.remove(&command_id);
        Ok(WebDriverBiDiCorrelatedResponse {
            command_id,
            outcome,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{WebDriverBiDiCommandCorrelationError, WebDriverBiDiCommandKind};

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
                WebDriverBiDiCommandCorrelationError::CommandKindMismatch {
                    expected: WebDriverBiDiCommandKind::SessionEnd,
                    actual: WebDriverBiDiCommandKind::SessionStatus,
                },
                "WebDriver BiDi response command kind does not match the outstanding command",
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
