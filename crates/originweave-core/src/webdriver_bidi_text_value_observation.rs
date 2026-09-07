use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    AdmittedNodeHandle, BrowserAuthorityRegistry, BrowserRegistryError,
    MAX_WEBDRIVER_BIDI_COMMAND_ID, NodeHandleError, WebDriverBiDiRemoteNodeReference,
};

/// Exact WebDriver BiDi method used for the fixed text-value post-condition observation.
pub const WEBDRIVER_BIDI_SCRIPT_CALL_FUNCTION_METHOD: &str = "script.callFunction";

/// Product-owned function declaration used to read one admitted form control's current value.
///
/// Callers cannot replace this source string. It exists only as a reviewed adapter implementation
/// detail for a typed semantic observation and is not an arbitrary JavaScript capability.
pub const WEBDRIVER_BIDI_TEXT_VALUE_FUNCTION_DECLARATION: &str = "node => node.value";

/// Isolated WebDriver BiDi sandbox used for typed text-value post-condition observations.
pub const WEBDRIVER_BIDI_TEXT_VALUE_SANDBOX: &str = "originweave-postcondition-v1";

/// Fail-closed validation errors for one serialized text-value observation command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiTextValueObservationCommandError {
    /// The command identifier exceeds WebDriver BiDi's unsigned safe-integer range.
    InvalidCommandId,
}

impl Display for WebDriverBiDiTextValueObservationCommandError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCommandId => "WebDriver BiDi command id is outside the js-uint range",
        })
    }
}

impl Error for WebDriverBiDiTextValueObservationCommandError {}

/// Fail-closed authority errors while binding a text-value observation to an admitted current node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiTextValueObservationAuthorityError {
    /// The final deterministic observation command failed its bounded serialization contract.
    Command(WebDriverBiDiTextValueObservationCommandError),
    /// Current browser session, context, or origin authority could not be revalidated.
    BrowserAuthority(BrowserRegistryError),
    /// The observed node belongs to a stale or otherwise mismatched browser document lifetime.
    NodeHandle(NodeHandleError),
    /// The supplied wire node identifier is not the identifier admitted for this exact node handle.
    NodeExternalIdentifierMismatch,
}

impl Display for WebDriverBiDiTextValueObservationAuthorityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(error) => write!(formatter, "text-value observation rejected input: {error}"),
            Self::BrowserAuthority(error) => write!(
                formatter,
                "text-value observation browser authority rejected input: {error}"
            ),
            Self::NodeHandle(error) => write!(
                formatter,
                "text-value observation node authority rejected input: {error}"
            ),
            Self::NodeExternalIdentifierMismatch => formatter.write_str(
                "text-value observation wire node identifier does not match the admitted current node",
            ),
        }
    }
}

impl Error for WebDriverBiDiTextValueObservationAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Command(error) => Some(error),
            Self::BrowserAuthority(error) => Some(error),
            Self::NodeHandle(error) => Some(error),
            Self::NodeExternalIdentifierMismatch => None,
        }
    }
}

/// Deterministic sandboxed observation of one admitted text field's current value.
///
/// This is a typed semantic-observation adapter primitive, not a generic scripting surface. The
/// function declaration and sandbox are fixed product-owned constants, the caller can supply only
/// the command id plus already admitted browser/node authority, and construction revalidates the
/// current session, browsing context, origin, document epoch, and exact WebDriver BiDi `sharedId`.
/// The command performs no I/O and grants no new browser, policy, destination, secret, or Agent
/// authority. A matching protocol response must still be parsed and compared with the intended
/// non-secret text before the preceding text-input action can be treated as observed success.
#[derive(PartialEq, Eq)]
pub struct WebDriverBiDiTextValueObservationCommand {
    command_id: u64,
    browsing_context: String,
    json: String,
}

impl std::fmt::Debug for WebDriverBiDiTextValueObservationCommand {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiTextValueObservationCommand")
            .field("command_id", &self.command_id)
            .field("method", &WEBDRIVER_BIDI_SCRIPT_CALL_FUNCTION_METHOD)
            .field("sandbox", &WEBDRIVER_BIDI_TEXT_VALUE_SANDBOX)
            .finish_non_exhaustive()
    }
}

impl WebDriverBiDiTextValueObservationCommand {
    /// Bind one fixed text-value observation to the exact current semantic node admitted here.
    pub fn new_for_current_node(
        command_id: u64,
        browsing_context: &str,
        handle: &AdmittedNodeHandle,
        node: &WebDriverBiDiRemoteNodeReference,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<Self, WebDriverBiDiTextValueObservationAuthorityError> {
        registry
            .require_context_external_identifier(
                handle.browser_session(),
                handle.browsing_context(),
                browsing_context,
            )
            .map_err(WebDriverBiDiTextValueObservationAuthorityError::BrowserAuthority)?;

        let current_epoch = registry
            .require_context_origin(
                handle.browser_session(),
                handle.browsing_context(),
                handle.origin(),
            )
            .map_err(WebDriverBiDiTextValueObservationAuthorityError::BrowserAuthority)?;
        handle
            .validate_current(
                handle.browser_session(),
                handle.browsing_context(),
                handle.origin(),
                current_epoch,
            )
            .map_err(WebDriverBiDiTextValueObservationAuthorityError::NodeHandle)?;

        if !registry.node_external_identifier_matches(handle, node.shared_id()) {
            return Err(
                WebDriverBiDiTextValueObservationAuthorityError::NodeExternalIdentifierMismatch,
            );
        }

        Self::new(command_id, browsing_context, node)
            .map_err(WebDriverBiDiTextValueObservationAuthorityError::Command)
    }

    fn new(
        command_id: u64,
        browsing_context: &str,
        node: &WebDriverBiDiRemoteNodeReference,
    ) -> Result<Self, WebDriverBiDiTextValueObservationCommandError> {
        if command_id > MAX_WEBDRIVER_BIDI_COMMAND_ID {
            return Err(WebDriverBiDiTextValueObservationCommandError::InvalidCommandId);
        }

        let mut json = String::from("{\"id\":");
        json.push_str(&command_id.to_string());
        json.push_str(",\"method\":\"");
        json.push_str(WEBDRIVER_BIDI_SCRIPT_CALL_FUNCTION_METHOD);
        json.push_str("\",\"params\":{\"functionDeclaration\":\"");
        json.push_str(WEBDRIVER_BIDI_TEXT_VALUE_FUNCTION_DECLARATION);
        json.push_str("\",\"awaitPromise\":false,\"target\":{\"context\":");
        push_json_string(&mut json, browsing_context);
        json.push_str(",\"sandbox\":\"");
        json.push_str(WEBDRIVER_BIDI_TEXT_VALUE_SANDBOX);
        json.push_str("\"},\"arguments\":[{\"sharedId\":");
        push_json_string(&mut json, node.shared_id());
        json.push_str("}],\"resultOwnership\":\"none\"}}");

        Ok(Self {
            command_id,
            browsing_context: browsing_context.to_owned(),
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
        WEBDRIVER_BIDI_SCRIPT_CALL_FUNCTION_METHOD
    }

    /// Return the exact validated browsing-context identifier.
    #[must_use]
    pub fn browsing_context(&self) -> &str {
        &self.browsing_context
    }

    /// Return the fixed isolated sandbox used for this typed observation.
    #[must_use]
    pub const fn sandbox(&self) -> &'static str {
        WEBDRIVER_BIDI_TEXT_VALUE_SANDBOX
    }

    /// Return the fixed product-owned function declaration used by this typed observation.
    #[must_use]
    pub const fn function_declaration(&self) -> &'static str {
        WEBDRIVER_BIDI_TEXT_VALUE_FUNCTION_DECLARATION
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
