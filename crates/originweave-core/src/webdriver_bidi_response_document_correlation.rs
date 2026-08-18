use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::webdriver_bidi_command::{
    CorrelatedWebDriverBiDiLocateNodesResponse, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiLocateNodesResponseEnvelopeError,
};
use crate::webdriver_bidi_response_document::BoundedWebDriverBiDiResponseDocument;
use crate::webdriver_bidi_response_envelope::WebDriverBiDiResponseEnvelopeParseError;

/// Fail-closed errors while parsing and correlating one bounded WebDriver BiDi response document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiLocateNodesResponseDocumentError {
    /// The bounded document failed complete WebDriver BiDi response-envelope parsing.
    Parse(WebDriverBiDiResponseEnvelopeParseError),
    /// The parsed envelope failed exact command correlation.
    Envelope(WebDriverBiDiLocateNodesResponseEnvelopeError),
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
        }
    }
}

impl Error for WebDriverBiDiLocateNodesResponseDocumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Envelope(error) => Some(error),
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
}
