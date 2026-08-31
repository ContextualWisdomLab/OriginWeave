use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    AdmittedNodeHandle, BrowserAuthorityRegistry, BrowserRegistryError,
    MAX_WEBDRIVER_BIDI_COMMAND_ID, NodeHandleError, WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD,
    WebDriverBiDiRemoteNodeReference, contains_disallowed_protocol_text,
};

/// Maximum UTF-8 bytes accepted by one non-secret WebDriver BiDi text-input command.
pub const MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES: usize = 512;

/// Fail-closed validation errors for one serialized WebDriver BiDi text-input command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiTypeTextCommandError {
    /// The command identifier exceeds WebDriver BiDi's unsigned safe-integer range.
    InvalidCommandId,
    /// The text payload is empty.
    EmptyText,
    /// The text payload exceeds the reviewed local UTF-8 byte budget.
    TextTooLong,
    /// The text payload contains a control, non-space whitespace, or reviewed format character.
    InvalidText,
}

impl Display for WebDriverBiDiTypeTextCommandError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCommandId => "WebDriver BiDi command id is outside the js-uint range",
            Self::EmptyText => "WebDriver BiDi text input must not be empty",
            Self::TextTooLong => "WebDriver BiDi text input exceeds the local byte budget",
            Self::InvalidText => {
                "WebDriver BiDi text input contains a control, non-space whitespace, or reviewed format character"
            }
        })
    }
}

impl Error for WebDriverBiDiTypeTextCommandError {}

/// Fail-closed authority errors while binding text input to an admitted current node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiTypeTextAuthorityError {
    /// The final deterministic text-input command failed its bounded serialization contract.
    Command(WebDriverBiDiTypeTextCommandError),
    /// Current browser session, context, or origin authority could not be revalidated.
    BrowserAuthority(BrowserRegistryError),
    /// The observed node belongs to a stale or otherwise mismatched browser document lifetime.
    NodeHandle(NodeHandleError),
    /// The supplied wire node identifier is not the identifier admitted for this exact node handle.
    NodeExternalIdentifierMismatch,
}

impl Display for WebDriverBiDiTypeTextAuthorityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(error) => write!(formatter, "text-input command rejected input: {error}"),
            Self::BrowserAuthority(error) => {
                write!(
                    formatter,
                    "text-input browser authority rejected input: {error}"
                )
            }
            Self::NodeHandle(error) => {
                write!(
                    formatter,
                    "text-input node authority rejected input: {error}"
                )
            }
            Self::NodeExternalIdentifierMismatch => formatter.write_str(
                "text-input wire node identifier does not match the admitted current node",
            ),
        }
    }
}

impl Error for WebDriverBiDiTypeTextAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Command(error) => Some(error),
            Self::BrowserAuthority(error) => Some(error),
            Self::NodeHandle(error) => Some(error),
            Self::NodeExternalIdentifierMismatch => None,
        }
    }
}

/// Deterministic `input.performActions` command that focuses one admitted node and types text.
///
/// The command starts with a primary-button pointer move/down/up sequence on the exact admitted
/// element and then emits key-down/key-up pairs after three synchronized keyboard pauses. Pointer
/// pauses fill the remaining ticks, keeping the two WebDriver action sources aligned. This avoids
/// inheriting ambient focus from an unrelated element before keyboard input begins.
///
/// The payload is intentionally limited to non-secret, single-line protocol-safe text. Secret
/// material must use the separately governed broker/fill path rather than this public text value.
/// Construction grants no policy, destination, secret, or Agent authority and performs no I/O.
#[derive(Debug, PartialEq, Eq)]
pub struct WebDriverBiDiTypeTextCommand {
    command_id: u64,
    browsing_context: String,
    text_bytes: usize,
    json: String,
}

impl WebDriverBiDiTypeTextCommand {
    /// Bind one text-input command to the exact current semantic node admitted by this registry.
    pub fn new_for_current_node(
        command_id: u64,
        browsing_context: &str,
        text: &str,
        handle: &AdmittedNodeHandle,
        node: &WebDriverBiDiRemoteNodeReference,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<Self, WebDriverBiDiTypeTextAuthorityError> {
        registry
            .require_context_external_identifier(
                handle.browser_session(),
                handle.browsing_context(),
                browsing_context,
            )
            .map_err(WebDriverBiDiTypeTextAuthorityError::BrowserAuthority)?;

        let current_epoch = registry
            .require_context_origin(
                handle.browser_session(),
                handle.browsing_context(),
                handle.origin(),
            )
            .map_err(WebDriverBiDiTypeTextAuthorityError::BrowserAuthority)?;
        handle
            .validate_current(
                handle.browser_session(),
                handle.browsing_context(),
                handle.origin(),
                current_epoch,
            )
            .map_err(WebDriverBiDiTypeTextAuthorityError::NodeHandle)?;

        if !registry.node_external_identifier_matches(handle, node.shared_id()) {
            return Err(WebDriverBiDiTypeTextAuthorityError::NodeExternalIdentifierMismatch);
        }

        Self::new(command_id, browsing_context, text, node)
            .map_err(WebDriverBiDiTypeTextAuthorityError::Command)
    }

    fn new(
        command_id: u64,
        browsing_context: &str,
        text: &str,
        node: &WebDriverBiDiRemoteNodeReference,
    ) -> Result<Self, WebDriverBiDiTypeTextCommandError> {
        if command_id > MAX_WEBDRIVER_BIDI_COMMAND_ID {
            return Err(WebDriverBiDiTypeTextCommandError::InvalidCommandId);
        }
        if text.is_empty() {
            return Err(WebDriverBiDiTypeTextCommandError::EmptyText);
        }
        if text.len() > MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES {
            return Err(WebDriverBiDiTypeTextCommandError::TextTooLong);
        }
        if contains_disallowed_protocol_text(text, true) {
            return Err(WebDriverBiDiTypeTextCommandError::InvalidText);
        }

        let character_count = text.chars().count();
        let mut json = String::from("{\"id\":");
        json.push_str(&command_id.to_string());
        json.push_str(",\"method\":\"");
        json.push_str(WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD);
        json.push_str("\",\"params\":{\"context\":");
        push_json_string(&mut json, browsing_context);
        json.push_str(",\"actions\":[{\"type\":\"pointer\",\"id\":\"originweave-mouse\",\"parameters\":{\"pointerType\":\"mouse\"},\"actions\":[{\"type\":\"pointerMove\",\"x\":0,\"y\":0,\"origin\":{\"type\":\"element\",\"element\":{\"sharedId\":");
        push_json_string(&mut json, node.shared_id());
        json.push_str(
            "}}},{\"type\":\"pointerDown\",\"button\":0},{\"type\":\"pointerUp\",\"button\":0}",
        );
        for _ in 0..character_count.saturating_mul(2) {
            json.push_str(",{\"type\":\"pause\"}");
        }
        json.push_str("]},{\"type\":\"key\",\"id\":\"originweave-keyboard\",\"actions\":[{\"type\":\"pause\"},{\"type\":\"pause\"},{\"type\":\"pause\"}");
        for character in text.chars() {
            json.push_str(",{\"type\":\"keyDown\",\"value\":");
            push_json_character(&mut json, character);
            json.push_str("},{\"type\":\"keyUp\",\"value\":");
            push_json_character(&mut json, character);
            json.push('}');
        }
        json.push_str("]}]}}");

        Ok(Self {
            command_id,
            browsing_context: browsing_context.to_owned(),
            text_bytes: text.len(),
            json,
        })
    }

    /// Return the validated command identifier.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return the exact WebDriver BiDi method serialized by this command.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD
    }

    /// Return the exact validated browsing-context identifier.
    #[must_use]
    pub fn browsing_context(&self) -> &str {
        &self.browsing_context
    }

    /// Return the bounded UTF-8 byte length of the typed text without exposing a second copy.
    #[must_use]
    pub const fn text_bytes(&self) -> usize {
        self.text_bytes
    }

    /// Return the deterministic JSON command envelope.
    #[must_use]
    pub fn as_json(&self) -> &str {
        &self.json
    }
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            character => output.push(character),
        }
    }
    output.push('"');
}

fn push_json_character(output: &mut String, value: char) {
    output.push('"');
    match value {
        '"' => output.push_str("\\\""),
        '\\' => output.push_str("\\\\"),
        character => output.push(character),
    }
    output.push('"');
}
