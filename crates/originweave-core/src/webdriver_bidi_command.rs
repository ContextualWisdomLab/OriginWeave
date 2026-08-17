use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, WEBDRIVER_BIDI_LOCATE_NODES_METHOD,
    WebDriverBiDiAccessibilityQuery, contains_disallowed_protocol_text,
};

/// Maximum WebDriver BiDi command identifier representable by the protocol `js-uint` type.
pub const MAX_WEBDRIVER_BIDI_COMMAND_ID: u64 = 9_007_199_254_740_991;

/// Fail-closed validation errors for one serialized WebDriver BiDi `locateNodes` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiLocateNodesCommandError {
    /// The command identifier exceeds WebDriver BiDi's unsigned safe-integer range.
    InvalidCommandId,
    /// The browsing-context identifier is empty, over budget, or contains disallowed text.
    InvalidBrowsingContext,
}

impl Display for WebDriverBiDiLocateNodesCommandError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCommandId => "WebDriver BiDi command id is outside the js-uint range",
            Self::InvalidBrowsingContext => {
                "WebDriver BiDi browsing context is empty, over budget, or contains disallowed text"
            }
        })
    }
}

impl Error for WebDriverBiDiLocateNodesCommandError {}

/// Deterministic serialized command envelope for one bounded WebDriver BiDi accessibility query.
///
/// Construction accepts only a WebDriver BiDi `js-uint` command identifier, a bounded opaque
/// browsing-context identifier, and an already validated [`WebDriverBiDiAccessibilityQuery`]. The
/// serialized envelope fixes the exact `browsingContext.locateNodes` method, accessibility locator,
/// finite node budget, and minimal serialization options carried by the query. String values are
/// JSON-escaped without interpreting their content.
///
/// This is an inert transport value. It performs no browser I/O, authenticates no browser or
/// adapter, grants no session/context/origin authority, and cannot authorize policy or typed input.
/// A trusted transport adapter must still bind the command to the exact authenticated browser
/// session and later admit any response through the reviewed current-authority boundary.
#[derive(Debug, PartialEq, Eq)]
pub struct WebDriverBiDiLocateNodesCommand {
    command_id: u64,
    browsing_context: String,
    json: String,
}

impl WebDriverBiDiLocateNodesCommand {
    /// Validate and serialize one bounded `browsingContext.locateNodes` command envelope.
    pub fn new(
        command_id: u64,
        browsing_context: &str,
        query: &WebDriverBiDiAccessibilityQuery,
    ) -> Result<Self, WebDriverBiDiLocateNodesCommandError> {
        if command_id > MAX_WEBDRIVER_BIDI_COMMAND_ID {
            return Err(WebDriverBiDiLocateNodesCommandError::InvalidCommandId);
        }
        if browsing_context.is_empty()
            || browsing_context.len() > MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES
            || contains_disallowed_protocol_text(browsing_context, false)
        {
            return Err(WebDriverBiDiLocateNodesCommandError::InvalidBrowsingContext);
        }

        let mut json = String::from("{\"id\":");
        json.push_str(&command_id.to_string());
        json.push_str(",\"method\":\"");
        json.push_str(WEBDRIVER_BIDI_LOCATE_NODES_METHOD);
        json.push_str("\",\"params\":{\"context\":");
        push_json_string(&mut json, browsing_context);
        json.push_str(",\"locator\":{\"type\":\"");
        json.push_str(query.locator_type());
        json.push_str("\",\"value\":{");

        if let Some(role) = query.role() {
            json.push_str("\"role\":");
            push_json_string(&mut json, role);
        }
        if let Some(name) = query.name() {
            if query.role().is_some() {
                json.push(',');
            }
            json.push_str("\"name\":");
            push_json_string(&mut json, name);
        }

        json.push_str("}},\"maxNodeCount\":");
        json.push_str(&query.max_node_count().to_string());
        json.push_str(",\"serializationOptions\":{\"maxDomDepth\":");
        json.push_str(&query.serialization_max_dom_depth().to_string());
        json.push_str(",\"maxObjectDepth\":");
        json.push_str(&query.serialization_max_object_depth().to_string());
        json.push_str(",\"includeShadowTree\":");
        push_json_string(&mut json, query.serialization_include_shadow_tree());
        json.push_str("}}}");

        Ok(Self {
            command_id,
            browsing_context: browsing_context.to_owned(),
            json,
        })
    }

    /// Return the validated WebDriver BiDi command identifier.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return the exact WebDriver BiDi method serialized by this command.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        WEBDRIVER_BIDI_LOCATE_NODES_METHOD
    }

    /// Return the exact validated browsing-context identifier.
    #[must_use]
    pub fn browsing_context(&self) -> &str {
        &self.browsing_context
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
