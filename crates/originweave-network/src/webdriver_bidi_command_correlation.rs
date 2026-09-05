use std::{collections::BTreeMap, error::Error, fmt};

use crate::{MAX_WEBDRIVER_BIDI_JS_UINT, WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeKind};

/// Maximum number of local WebDriver BiDi commands retained as outstanding at once.
///
/// WebDriver BiDi permits commands to complete out of order. OriginWeave therefore keeps a
/// bounded local correlation map instead of assuming response order, while this resource ceiling
/// prevents an unbounded remote-control session from growing local correlation state indefinitely.
pub const MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS: usize = 256;

/// Typed WebDriver BiDi command families that require response-family provenance.
///
/// Existing legacy command slices may still use the unclassified correlation API while they are
/// migrated independently. A typed command is deliberately segregated from that compatibility
/// path: an unclassified response consumer cannot consume its id, and a typed consumer cannot
/// consume an unclassified id merely because the numeric correlation value matches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiCommandKind {
    /// Fixed product-owned `script.callFunction` text-value observation.
    TextValueObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutstandingCommandKind {
    Unclassified,
    Typed(WebDriverBiDiCommandKind),
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
    /// The response consumer does not match the command provenance registered for this identifier.
    CommandKindMismatch,
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
            Self::CommandKindMismatch => {
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
/// The compatibility [`Self::register_command`] path retains unclassified command provenance for
/// existing typed boundaries that have not yet migrated. Security-sensitive consumers that require
/// an exact command family use [`Self::register_command_for`] and [`Self::correlate_response_for`].
/// Typed and unclassified entries cannot consume one another. Events, null-id errors, unknown ids,
/// and provenance mismatches leave outstanding state untouched. This type performs no I/O, retry,
/// command serialization, browser authentication, or authority grant. Debug output reports only
/// the outstanding-count summary; identifiers and command provenance remain private state.
#[derive(Default)]
pub struct WebDriverBiDiCommandCorrelation {
    outstanding: BTreeMap<u64, OutstandingCommandKind>,
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

    /// Register one legacy/unclassified local command id before its response can be accepted.
    ///
    /// This compatibility path cannot later be consumed by a typed response-family boundary.
    /// Identifiers are unique only while outstanding. A completed or explicitly retired id may be
    /// reused later, matching WebDriver BiDi's local-end correlation semantics.
    pub fn register_command(
        &mut self,
        command_id: u64,
    ) -> Result<(), WebDriverBiDiCommandCorrelationError> {
        self.register(command_id, OutstandingCommandKind::Unclassified)
    }

    /// Register one local command id together with its exact typed command family.
    ///
    /// A typed entry cannot later be consumed by the unclassified compatibility response path or by
    /// another typed command family. Registration remains local correlation bookkeeping only and
    /// does not authorize or dispatch the command.
    pub fn register_command_for(
        &mut self,
        command_id: u64,
        command_kind: WebDriverBiDiCommandKind,
    ) -> Result<(), WebDriverBiDiCommandCorrelationError> {
        self.register(command_id, OutstandingCommandKind::Typed(command_kind))
    }

    /// Explicitly retire one outstanding command without accepting a response for it.
    ///
    /// This supports caller-owned cancellation or session teardown without retaining stale ids.
    /// Retirement does not produce success evidence and therefore may remove either compatibility
    /// or typed provenance when the caller owns the exact outstanding identifier.
    pub fn retire_command(
        &mut self,
        command_id: u64,
    ) -> Result<(), WebDriverBiDiCommandCorrelationError> {
        if self.outstanding.remove(&command_id).is_some() {
            Ok(())
        } else {
            Err(WebDriverBiDiCommandCorrelationError::CommandNotOutstanding)
        }
    }

    /// Correlate one parsed local-end envelope with an unclassified outstanding command.
    ///
    /// This compatibility consumer cannot consume an id that was registered through the typed
    /// provenance API. Unknown ids and provenance mismatches fail without consuming state. Events
    /// and null-id errors fail before touching the map.
    pub fn correlate_response(
        &mut self,
        envelope: &WebDriverBiDiJsonEnvelope,
    ) -> Result<WebDriverBiDiCorrelatedResponse, WebDriverBiDiCommandCorrelationError> {
        self.correlate_response_kind(envelope, OutstandingCommandKind::Unclassified)
    }

    /// Correlate one parsed local-end envelope with the exact typed outstanding command family.
    ///
    /// A matching numeric id is insufficient: the registered family must also match. This prevents
    /// a response produced for an unrelated outstanding command from being consumed as typed
    /// evidence by another protocol boundary.
    pub fn correlate_response_for(
        &mut self,
        envelope: &WebDriverBiDiJsonEnvelope,
        expected_kind: WebDriverBiDiCommandKind,
    ) -> Result<WebDriverBiDiCorrelatedResponse, WebDriverBiDiCommandCorrelationError> {
        self.correlate_response_kind(envelope, OutstandingCommandKind::Typed(expected_kind))
    }

    fn register(
        &mut self,
        command_id: u64,
        command_kind: OutstandingCommandKind,
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

    fn correlate_response_kind(
        &mut self,
        envelope: &WebDriverBiDiJsonEnvelope,
        expected_kind: OutstandingCommandKind,
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
            WebDriverBiDiJsonEnvelopeKind::Success => self.complete(
                envelope.success_command_id(),
                expected_kind,
                WebDriverBiDiCorrelatedResponseOutcome::Success,
            ),
        }
    }

    fn complete(
        &mut self,
        command_id: u64,
        expected_kind: OutstandingCommandKind,
        outcome: WebDriverBiDiCorrelatedResponseOutcome,
    ) -> Result<WebDriverBiDiCorrelatedResponse, WebDriverBiDiCommandCorrelationError> {
        let actual_kind = self
            .outstanding
            .get(&command_id)
            .copied()
            .ok_or(WebDriverBiDiCommandCorrelationError::CommandNotOutstanding)?;
        if actual_kind != expected_kind {
            return Err(WebDriverBiDiCommandCorrelationError::CommandKindMismatch);
        }
        let _removed = self.outstanding.remove(&command_id);
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
                WebDriverBiDiCommandCorrelationError::CommandKindMismatch,
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
