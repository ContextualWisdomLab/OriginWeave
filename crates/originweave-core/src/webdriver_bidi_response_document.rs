use std::fmt;

/// Maximum raw WebDriver BiDi response-document size admitted before parsing.
///
/// This is an OriginWeave product safety budget, not a WebDriver BiDi protocol
/// limit. Browser adapters must enforce it before handing raw response text to a
/// JSON parser so an untrusted or malfunctioning peer cannot cause unbounded
/// parser input allocation.
pub const MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES: usize = 65_536;

/// Fail-closed reasons for rejecting a raw WebDriver BiDi response document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiResponseDocumentAdmissionError {
    /// The response contains no JSON document after removing JSON whitespace.
    EmptyDocument,
    /// The raw response exceeds the OriginWeave pre-parser byte budget.
    DocumentTooLarge,
    /// The first and last non-whitespace bytes do not delimit a JSON object.
    InvalidObjectBoundary,
}

impl fmt::Display for WebDriverBiDiResponseDocumentAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDocument => formatter.write_str("WebDriver BiDi response document is empty"),
            Self::DocumentTooLarge => write!(
                formatter,
                "WebDriver BiDi response document exceeds {MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES} bytes"
            ),
            Self::InvalidObjectBoundary => formatter.write_str(
                "WebDriver BiDi response document must have a top-level JSON object boundary",
            ),
        }
    }
}

impl std::error::Error for WebDriverBiDiResponseDocumentAdmissionError {}

/// Exact raw WebDriver BiDi response text admitted to the parser boundary.
///
/// Construction proves only the OriginWeave byte budget and an obvious
/// top-level object boundary. It deliberately does not claim JSON validity,
/// response correlation, browser authenticity, or action authority. The exact
/// text is retained so downstream parsing/evidence can remain bound to the
/// admitted bytes.
#[derive(Debug, PartialEq, Eq)]
pub struct BoundedWebDriverBiDiResponseDocument {
    raw: String,
}

impl BoundedWebDriverBiDiResponseDocument {
    /// Admits exact raw response text under the pre-parser safety contract.
    pub fn new(raw: &str) -> Result<Self, WebDriverBiDiResponseDocumentAdmissionError> {
        if raw.len() > MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES {
            return Err(WebDriverBiDiResponseDocumentAdmissionError::DocumentTooLarge);
        }

        let bounded = raw.trim_matches(|character| matches!(character, ' ' | '\t' | '\r' | '\n'));
        if bounded.is_empty() {
            return Err(WebDriverBiDiResponseDocumentAdmissionError::EmptyDocument);
        }
        if !bounded.starts_with('{') || !bounded.ends_with('}') {
            return Err(WebDriverBiDiResponseDocumentAdmissionError::InvalidObjectBoundary);
        }

        Ok(Self {
            raw: raw.to_owned(),
        })
    }

    /// Returns the exact admitted response text, including surrounding JSON whitespace.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}
