use std::error::Error;
use std::fmt::{Display, Formatter};

mod locate_nodes_result_document;

use crate::webdriver_bidi_command::{
    CorrelatedWebDriverBiDiLocateNodesResponse, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiLocateNodesResponseEnvelopeError,
};
use crate::webdriver_bidi_response_document::BoundedWebDriverBiDiResponseDocument;
use crate::webdriver_bidi_response_envelope::WebDriverBiDiResponseEnvelopeParseError;
use crate::webdriver_bidi_result::{
    ValidatedWebDriverBiDiLocateNodesResult, WebDriverBiDiLocateNodesResultAdmissionError,
};

/// Fail-closed errors while parsing, correlating, and admitting one bounded WebDriver BiDi
/// `locateNodes` response document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiLocateNodesResponseDocumentError {
    /// The bounded document failed complete WebDriver BiDi response-envelope parsing.
    Parse(WebDriverBiDiResponseEnvelopeParseError),
    /// The parsed envelope failed exact command correlation or success-only conversion.
    Envelope(WebDriverBiDiLocateNodesResponseEnvelopeError),
    /// The correlated success result omitted its required `nodes` field.
    MissingResultNodes,
    /// The correlated success result's `nodes` field was not a JSON array.
    InvalidResultNodes,
    /// The correlated success result repeated the decoded `nodes` field.
    DuplicateResultNodes,
    /// One in-budget `nodes` array item was not a JSON object.
    InvalidResultNode,
    /// One in-budget node object repeated decoded `type` or `sharedId` authority-relevant metadata.
    DuplicateResultNodeField,
    /// One in-budget node object omitted its required WebDriver BiDi remote-value `type` field.
    MissingResultNodeType,
    /// One in-budget node object's `type` field was not a JSON string.
    InvalidResultNodeType,
    /// One present in-budget node `sharedId` field was not a JSON string.
    InvalidResultNodeSharedId,
    /// Exact command-budget or remote-node admission rejected the wire-derived node batch.
    ResultAdmission(WebDriverBiDiLocateNodesResultAdmissionError),
    /// A second-pass result parser invariant failed after complete envelope parsing succeeded.
    ResultParserInvariant,
}

impl Display for WebDriverBiDiLocateNodesResponseDocumentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => write!(
                formatter,
                "WebDriver BiDi response document rejected envelope parsing: {error}"
            ),
            Self::Envelope(error) => write!(
                formatter,
                "WebDriver BiDi response document rejected command correlation: {error}"
            ),
            Self::MissingResultNodes => {
                formatter.write_str("WebDriver BiDi locateNodes result is missing its nodes field")
            }
            Self::InvalidResultNodes => formatter
                .write_str("WebDriver BiDi locateNodes result nodes field is not a JSON array"),
            Self::DuplicateResultNodes => formatter.write_str(
                "WebDriver BiDi locateNodes result contains duplicate decoded nodes fields",
            ),
            Self::InvalidResultNode => formatter
                .write_str("WebDriver BiDi locateNodes result contains a non-object node item"),
            Self::DuplicateResultNodeField => formatter.write_str(
                "WebDriver BiDi locateNodes node contains duplicate authority-relevant fields",
            ),
            Self::MissingResultNodeType => formatter
                .write_str("WebDriver BiDi locateNodes node is missing its remote-value type"),
            Self::InvalidResultNodeType => formatter
                .write_str("WebDriver BiDi locateNodes node type is not a JSON string"),
            Self::InvalidResultNodeSharedId => formatter
                .write_str("WebDriver BiDi locateNodes node sharedId is not a JSON string"),
            Self::ResultAdmission(error) => write!(
                formatter,
                "WebDriver BiDi locateNodes wire result rejected node admission: {error}"
            ),
            Self::ResultParserInvariant => formatter.write_str(
                "WebDriver BiDi locateNodes result parser invariant failed after envelope validation",
            ),
        }
    }
}

impl Error for WebDriverBiDiLocateNodesResponseDocumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Envelope(error) => Some(error),
            Self::ResultAdmission(error) => Some(error),
            Self::MissingResultNodes
            | Self::InvalidResultNodes
            | Self::DuplicateResultNodes
            | Self::InvalidResultNode
            | Self::DuplicateResultNodeField
            | Self::MissingResultNodeType
            | Self::InvalidResultNodeType
            | Self::InvalidResultNodeSharedId
            | Self::ResultParserInvariant => None,
        }
    }
}

impl WebDriverBiDiLocateNodesCommand {
    /// Consume this command and one bounded raw response through parsing and exact correlation.
    ///
    /// The document must first pass complete response-envelope parsing. Only the resulting typed
    /// response kind and protocol-range response id are then admitted to the existing exact command
    /// correlation boundary. Parser and correlation failures remain distinguishable and preserve
    /// their causal error sources. This boundary does not authenticate Chromium, ChromeDriver, or
    /// WebSocket transport provenance, validate `locateNodes` result nodes, mint node authority,
    /// authorize an Agent action, execute browser input, or prove a post-condition.
    pub fn correlate_response_document(
        self,
        document: BoundedWebDriverBiDiResponseDocument,
    ) -> Result<
        CorrelatedWebDriverBiDiLocateNodesResponse,
        WebDriverBiDiLocateNodesResponseDocumentError,
    > {
        let parsed = document
            .parse_command_response()
            .map_err(WebDriverBiDiLocateNodesResponseDocumentError::Parse)?;
        self.correlate_response_envelope(parsed.kind(), parsed.response_id())
            .map_err(WebDriverBiDiLocateNodesResponseDocumentError::Envelope)
    }

    /// Consume one bounded raw `locateNodes` response through exact wire-derived node admission.
    ///
    /// The same bounded document first passes the complete response-envelope parser, exact command
    /// correlation, and success-only conversion. Only then does the result parser derive the exact
    /// `result.nodes` array from that already-validated wire document. The command's exact
    /// `maxNodeCount` is carried into this parser so overflow items are consumed only as generic JSON
    /// and produce the existing result-budget failure before authority-relevant node metadata is
    /// decoded or normalized. Decoded duplicate `nodes`, and duplicate or malformed in-budget
    /// `type`/`sharedId` fields, fail closed. JSON-escaped protocol metadata is decoded before
    /// admission, and callers cannot supply replacement node metadata to this method.
    ///
    /// Success remains untrusted transport evidence. It does not authenticate Chromium,
    /// ChromeDriver, WebSocket/TLS provenance, or an adapter process; prove current
    /// session/context/origin/document authority; mint OriginWeave node handles; authorize policy
    /// or typed input; execute browser I/O; or prove a post-condition.
    pub fn admit_response_document_nodes(
        self,
        document: BoundedWebDriverBiDiResponseDocument,
    ) -> Result<
        ValidatedWebDriverBiDiLocateNodesResult,
        WebDriverBiDiLocateNodesResponseDocumentError,
    > {
        let parsed = document
            .parse_command_response()
            .map_err(WebDriverBiDiLocateNodesResponseDocumentError::Parse)?;
        let correlated = self
            .correlate_response_envelope(parsed.kind(), parsed.response_id())
            .map_err(WebDriverBiDiLocateNodesResponseDocumentError::Envelope)?;
        let validated = correlated
            .into_validated_success()
            .map_err(WebDriverBiDiLocateNodesResponseDocumentError::Envelope)?;
        let wire_nodes = locate_nodes_result_document::parse_wire_locate_nodes_result_bounded(
            parsed.as_str(),
            validated.max_node_count(),
        )?;
        let admission_parts = wire_nodes
            .iter()
            .map(locate_nodes_result_document::WireLocateNodesNode::as_admission_parts)
            .collect::<Vec<_>>();
        validated
            .admit_result_nodes(&admission_parts)
            .map_err(WebDriverBiDiLocateNodesResponseDocumentError::ResultAdmission)
    }
}

#[cfg(test)]
mod tests {
    use super::locate_nodes_result_document::{
        parse_wire_locate_nodes_result, parse_wire_locate_nodes_result_bounded,
    };
    use super::WebDriverBiDiLocateNodesResponseDocumentError;

    #[test]
    fn wire_node_admission_parts_preserve_wire_derived_metadata() {
        let nodes = parse_wire_locate_nodes_result(concat!(
            "{\"result\":{\"nodes\":[",
            "{\"type\":\"node\",\"sharedId\":\"shared-1\"},",
            "{\"type\":\"window\"}",
            "]}}"
        ))
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].as_admission_parts(), ("node", Some("shared-1")));
        assert_eq!(nodes[1].as_admission_parts(), ("window", None));
    }

    #[test]
    fn bounded_overflow_parser_preserves_invalid_generic_json_invariant() {
        let result = parse_wire_locate_nodes_result_bounded(
            concat!(
                "{\"result\":{\"nodes\":[",
                "{\"type\":\"node\",\"sharedId\":\"shared-1\"},",
                "?]}}"
            ),
            1,
        );

        assert!(matches!(
            result,
            Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant)
        ));
    }
}
