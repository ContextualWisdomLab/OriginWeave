use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, WEBDRIVER_BIDI_LOCATE_NODES_METHOD,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiAccessibilityQueryError,
    contains_disallowed_protocol_text,
};

/// Maximum WebDriver BiDi command identifier representable by the protocol `js-uint` type.
pub const MAX_WEBDRIVER_BIDI_COMMAND_ID: u64 = 9_007_199_254_740_991;

/// WebDriver BiDi method used for one bounded typed pointer action sequence.
pub const WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD: &str = "input.performActions";

/// Fail-closed validation errors for one serialized WebDriver BiDi pointer click command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiPointerClickCommandError {
    /// The command identifier exceeds WebDriver BiDi's unsigned safe-integer range.
    InvalidCommandId,
    /// The browsing-context identifier is empty, over budget, or contains disallowed text.
    InvalidBrowsingContext,
}

impl Display for WebDriverBiDiPointerClickCommandError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCommandId => "WebDriver BiDi command id is outside the js-uint range",
            Self::InvalidBrowsingContext => {
                "WebDriver BiDi browsing context is empty, over budget, or contains disallowed text"
            }
        })
    }
}

impl Error for WebDriverBiDiPointerClickCommandError {}

/// Deterministic command for one primary-button click on an admitted remote node.
///
/// The fixed mouse action sequence moves to the element origin, presses button zero, and releases
/// button zero. Construction accepts an already admitted remote node reference and does not grant
/// browser-session, context, origin, document-epoch, policy, approval, or Agent authority. A trusted
/// adapter must bind this inert command to current authority before transport.
#[derive(Debug, PartialEq, Eq)]
pub struct WebDriverBiDiPointerClickCommand {
    command_id: u64,
    browsing_context: String,
    json: String,
}

impl WebDriverBiDiPointerClickCommand {
    /// Validate and serialize one bounded `input.performActions` pointer click command.
    pub fn new(
        command_id: u64,
        browsing_context: &str,
        node: &crate::WebDriverBiDiRemoteNodeReference,
    ) -> Result<Self, WebDriverBiDiPointerClickCommandError> {
        if command_id > MAX_WEBDRIVER_BIDI_COMMAND_ID {
            return Err(WebDriverBiDiPointerClickCommandError::InvalidCommandId);
        }
        if browsing_context.is_empty()
            || browsing_context.len() > MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES
            || contains_disallowed_protocol_text(browsing_context, false)
        {
            return Err(WebDriverBiDiPointerClickCommandError::InvalidBrowsingContext);
        }

        let mut json = String::from("{\"id\":");
        json.push_str(&command_id.to_string());
        json.push_str(",\"method\":\"");
        json.push_str(WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD);
        json.push_str("\",\"params\":{\"context\":");
        push_json_string(&mut json, browsing_context);
        json.push_str(",\"actions\":[{\"type\":\"pointer\",\"id\":\"originweave-mouse\",\"parameters\":{\"pointerType\":\"mouse\"},\"actions\":[{\"type\":\"pointerMove\",\"x\":0,\"y\":0,\"origin\":{\"type\":\"element\",\"element\":{\"sharedId\":");
        push_json_string(&mut json, node.shared_id());
        json.push_str("}}},{\"type\":\"pointerDown\",\"button\":0},{\"type\":\"pointerUp\",\"button\":0}]}]}}");

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
        WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD
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

/// Fail-closed errors while correlating one WebDriver BiDi response with its exact command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiLocateNodesResponseCorrelationError {
    /// The returned response identifier exceeds WebDriver BiDi's `js-uint` range.
    InvalidResponseId,
    /// The returned response identifier belongs to a different in-flight command.
    ResponseIdMismatch {
        /// Exact command identifier that this response must carry.
        expected: u64,
        /// Untrusted response identifier returned by the adapter.
        actual: u64,
    },
}

impl Display for WebDriverBiDiLocateNodesResponseCorrelationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidResponseId => {
                formatter.write_str("WebDriver BiDi response id is outside the js-uint range")
            }
            Self::ResponseIdMismatch { expected, actual } => write!(
                formatter,
                "WebDriver BiDi response id {actual} does not match command id {expected}"
            ),
        }
    }
}

impl Error for WebDriverBiDiLocateNodesResponseCorrelationError {}

/// Structured WebDriver BiDi command-response envelope kind retained through correlation.
///
/// A later trusted parser must derive this classification from the exact wire envelope. This value
/// does not validate raw JSON or grant browser, node, policy, or Agent authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiCommandResponseKind {
    /// A WebDriver BiDi command success response.
    Success,
    /// A WebDriver BiDi command error response.
    Error,
}

/// Fail-closed errors while admitting a structured WebDriver BiDi response envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiLocateNodesResponseEnvelopeError {
    /// A success envelope did not carry the required command response identifier.
    MissingResponseId,
    /// An error envelope carried no recoverable command identifier and cannot be correlated.
    UncorrelatableErrorResponse,
    /// A correlated error envelope cannot be converted into success response evidence.
    CorrelatedErrorResponse,
    /// The present response identifier failed exact command correlation.
    Correlation(WebDriverBiDiLocateNodesResponseCorrelationError),
}

impl Display for WebDriverBiDiLocateNodesResponseEnvelopeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingResponseId => {
                formatter.write_str("WebDriver BiDi success response is missing its command id")
            }
            Self::UncorrelatableErrorResponse => formatter.write_str(
                "WebDriver BiDi error response has no recoverable command id for correlation",
            ),
            Self::CorrelatedErrorResponse => formatter
                .write_str("WebDriver BiDi error response cannot become success response evidence"),
            Self::Correlation(error) => write!(
                formatter,
                "WebDriver BiDi response envelope rejected command correlation: {error}"
            ),
        }
    }
}

impl Error for WebDriverBiDiLocateNodesResponseEnvelopeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Correlation(error) => Some(error),
            Self::MissingResponseId
            | Self::UncorrelatableErrorResponse
            | Self::CorrelatedErrorResponse => None,
        }
    }
}

/// Non-cloneable evidence that one `locateNodes` response matched the exact command id.
///
/// Only [`WebDriverBiDiLocateNodesCommand::correlate_response_id`] can construct this value. It
/// retains the exact command identifier, bounded browsing-context identifier, and exact serialized
/// result budget so a later trusted transport boundary can carry correlation evidence forward
/// without reconstructing authority from ambient query state. It does not authenticate a browser or
/// adapter, prove current OriginWeave session/context/origin authority, validate response payload
/// shape, admit nodes, or authorize an Agent action.
#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedWebDriverBiDiLocateNodesResponse {
    command_id: u64,
    browsing_context: String,
    max_node_count: u16,
}

impl ValidatedWebDriverBiDiLocateNodesResponse {
    /// Return the exact command identifier proven to match the response.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return the bounded browsing-context identifier serialized by the matched command.
    #[must_use]
    pub fn browsing_context(&self) -> &str {
        &self.browsing_context
    }

    /// Return the exact `maxNodeCount` serialized by the matched command.
    #[must_use]
    pub const fn max_node_count(&self) -> u16 {
        self.max_node_count
    }

    /// Validate a parsed `locateNodes` result count against the matched command's exact budget.
    ///
    /// This check is intentionally carried by command-correlation evidence rather than by a
    /// separately supplied query value, preventing downstream code from validating an untrusted
    /// response against a different, more permissive result budget. Zero through the serialized
    /// maximum are valid; any larger result fails closed before node normalization or admission.
    pub fn validate_result_count(
        &self,
        returned_node_count: usize,
    ) -> Result<(), WebDriverBiDiAccessibilityQueryError> {
        if returned_node_count > usize::from(self.max_node_count) {
            return Err(WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded);
        }
        Ok(())
    }
}

/// Non-cloneable structured-envelope evidence for one correlated `locateNodes` response.
///
/// This value deliberately keeps success and error envelopes distinguishable after exact response
/// id correlation. The only conversion into [`ValidatedWebDriverBiDiLocateNodesResponse`] is
/// [`Self::into_validated_success`], which fails closed for a correlated error envelope. A later
/// trusted response parser must classify the exact wire envelope before calling
/// [`WebDriverBiDiLocateNodesCommand::correlate_response_envelope`]. This value performs no raw JSON
/// parsing, browser or adapter authentication, node admission, policy authorization, or Agent
/// action authorization.
#[derive(Debug, PartialEq, Eq)]
pub struct CorrelatedWebDriverBiDiLocateNodesResponse {
    kind: WebDriverBiDiCommandResponseKind,
    correlated: ValidatedWebDriverBiDiLocateNodesResponse,
}

impl CorrelatedWebDriverBiDiLocateNodesResponse {
    /// Return whether the exact correlated envelope was classified as success or error.
    #[must_use]
    pub const fn kind(&self) -> WebDriverBiDiCommandResponseKind {
        self.kind
    }

    /// Return the exact command identifier proven to match the response.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.correlated.command_id()
    }

    /// Return the bounded browsing-context identifier serialized by the matched command.
    #[must_use]
    pub fn browsing_context(&self) -> &str {
        self.correlated.browsing_context()
    }

    /// Consume this envelope and return correlation evidence only when it was a success response.
    ///
    /// A correlated WebDriver BiDi error envelope remains error evidence and is rejected as
    /// [`WebDriverBiDiLocateNodesResponseEnvelopeError::CorrelatedErrorResponse`]. This explicit
    /// fail-closed conversion prevents downstream result/node admission code from accidentally
    /// erasing the protocol response kind while reusing exact command correlation evidence.
    pub fn into_validated_success(
        self,
    ) -> Result<
        ValidatedWebDriverBiDiLocateNodesResponse,
        WebDriverBiDiLocateNodesResponseEnvelopeError,
    > {
        match self.kind {
            WebDriverBiDiCommandResponseKind::Success => Ok(self.correlated),
            WebDriverBiDiCommandResponseKind::Error => {
                Err(WebDriverBiDiLocateNodesResponseEnvelopeError::CorrelatedErrorResponse)
            }
        }
    }
}

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
    max_node_count: u16,
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
            max_node_count: query.max_node_count(),
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

    /// Consume this command and correlate one untrusted response identifier with it.
    ///
    /// The response identifier is validated against WebDriver BiDi's `js-uint` range before exact
    /// equality is checked. Success consumes the command and returns non-cloneable correlation
    /// evidence, preventing this command value from being reused to validate another response. The
    /// evidence also retains the exact `maxNodeCount` serialized by this command so later result
    /// admission cannot substitute a different query budget. This does not parse a response,
    /// authenticate the transport, or grant browser/Agent authority.
    pub fn correlate_response_id(
        self,
        response_id: u64,
    ) -> Result<
        ValidatedWebDriverBiDiLocateNodesResponse,
        WebDriverBiDiLocateNodesResponseCorrelationError,
    > {
        if response_id > MAX_WEBDRIVER_BIDI_COMMAND_ID {
            return Err(WebDriverBiDiLocateNodesResponseCorrelationError::InvalidResponseId);
        }
        if response_id != self.command_id {
            return Err(
                WebDriverBiDiLocateNodesResponseCorrelationError::ResponseIdMismatch {
                    expected: self.command_id,
                    actual: response_id,
                },
            );
        }

        Ok(ValidatedWebDriverBiDiLocateNodesResponse {
            command_id: self.command_id,
            browsing_context: self.browsing_context,
            max_node_count: self.max_node_count,
        })
    }

    /// Consume this command and admit one already classified response envelope for correlation.
    ///
    /// A success envelope must carry a response id. A WebDriver BiDi error envelope may have a null
    /// id when no valid command id can be recovered; that case returns
    /// [`WebDriverBiDiLocateNodesResponseEnvelopeError::UncorrelatableErrorResponse`] and produces
    /// no correlation evidence. When an id is present, the same protocol-range and exact-id checks
    /// as [`Self::correlate_response_id`] apply. The returned evidence retains whether the envelope
    /// was success or error so an error cannot silently become success evidence.
    ///
    /// The caller must obtain `kind` and `response_id` from a separately reviewed exact response
    /// parser. This method does not parse JSON, validate result payload shape, authenticate a browser
    /// or adapter, admit nodes, or grant policy, typed-input, secret, or Agent authority.
    pub fn correlate_response_envelope(
        self,
        kind: WebDriverBiDiCommandResponseKind,
        response_id: Option<u64>,
    ) -> Result<
        CorrelatedWebDriverBiDiLocateNodesResponse,
        WebDriverBiDiLocateNodesResponseEnvelopeError,
    > {
        let response_id = match (kind, response_id) {
            (WebDriverBiDiCommandResponseKind::Success, None) => {
                return Err(WebDriverBiDiLocateNodesResponseEnvelopeError::MissingResponseId);
            }
            (WebDriverBiDiCommandResponseKind::Error, None) => {
                return Err(
                    WebDriverBiDiLocateNodesResponseEnvelopeError::UncorrelatableErrorResponse,
                );
            }
            (_, Some(response_id)) => response_id,
        };
        let correlated = self
            .correlate_response_id(response_id)
            .map_err(WebDriverBiDiLocateNodesResponseEnvelopeError::Correlation)?;

        Ok(CorrelatedWebDriverBiDiLocateNodesResponse { kind, correlated })
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
