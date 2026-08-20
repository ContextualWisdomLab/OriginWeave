use std::{error::Error, fmt, time::Duration};

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, ValidatedWebDriverBiDiLocateNodesResult,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiLocateNodesResponseDocumentError,
    WebDriverBiDiResponseDocumentAdmissionError,
};

use crate::webdriver_bidi_websocket_handshake::{
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey,
};

/// Fail-closed failures while exchanging one bounded WebDriver BiDi `locateNodes` command.
///
/// Every variant preserves the first causal boundary. Frame I/O retains the existing bounded
/// WebSocket error, raw response bytes must pass the core pre-parser admission contract, and the
/// admitted document must correlate to the exact consumed command before result nodes are returned.
/// An unexpected frame shape has no nested source because it is a protocol-shape refusal rather than
/// an underlying I/O or parser failure.
#[derive(Debug)]
pub enum WebDriverBiDiLocateNodesExchangeError {
    /// Bounded WebSocket frame write or read failed.
    Frame(WebDriverBiDiWebSocketFrameError),
    /// The first returned frame was not one complete text message.
    UnexpectedResponseFrame {
        /// Whether the returned frame carried the RFC 6455 FIN bit.
        fin: bool,
        /// Exact returned RFC 6455 opcode.
        opcode: u8,
    },
    /// The exact response-frame payload failed bounded raw-document admission.
    ResponseDocument(WebDriverBiDiResponseDocumentAdmissionError),
    /// The admitted response document failed parsing, exact correlation, or node admission.
    LocateNodesResponse(WebDriverBiDiLocateNodesResponseDocumentError),
}

impl fmt::Display for WebDriverBiDiLocateNodesExchangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Frame(error) => write!(
                formatter,
                "WebDriver BiDi locateNodes WebSocket frame exchange failed: {error}"
            ),
            Self::UnexpectedResponseFrame { fin, opcode } => write!(
                formatter,
                "WebDriver BiDi locateNodes exchange requires one final text response frame; received fin={fin}, opcode=0x{opcode:02x}"
            ),
            Self::ResponseDocument(error) => write!(
                formatter,
                "WebDriver BiDi locateNodes response frame failed raw-document admission: {error}"
            ),
            Self::LocateNodesResponse(error) => write!(
                formatter,
                "WebDriver BiDi locateNodes response document failed exact wire admission: {error}"
            ),
        }
    }
}

impl Error for WebDriverBiDiLocateNodesExchangeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frame(error) => Some(error),
            Self::ResponseDocument(error) => Some(error),
            Self::LocateNodesResponse(error) => Some(error),
            Self::UnexpectedResponseFrame { .. } => None,
        }
    }
}

impl WebDriverBiDiWebSocketEstablished {
    /// Exchange one exact bounded `browsingContext.locateNodes` command on this verified stream.
    ///
    /// The command is serialized by the reviewed core boundary and written as one masked client
    /// text frame using the caller-supplied fresh masking key. The first returned server frame must
    /// be a complete unmasked text frame (`FIN=1`, opcode `0x1`); continuation, binary, ping, pong,
    /// close, and fragmented data fail closed rather than being reinterpreted as a BiDi response.
    /// Its exact payload bytes then pass the existing bounded UTF-8/document admission, complete
    /// WebDriver BiDi response parser, exact command-id correlation, and wire-derived node admission.
    ///
    /// `frame_timeout` is independently enforced by the existing bounded write and bounded read
    /// operations, so a successful exchange may consume up to two such operation budgets. Any
    /// failure consumes this transport state and yields no reusable WebSocket stream, preventing a
    /// partially written/read protocol state from being promoted into subsequent authority.
    ///
    /// Success returns the same exact peer-verified WebSocket stream plus untrusted normalized node
    /// evidence. It does not authenticate Chromium/ChromeDriver process provenance, prove current
    /// OriginWeave session/context/origin/document authority, authorize policy or typed input, mint
    /// node handles, execute a browser action, or prove a post-condition.
    pub fn exchange_locate_nodes(
        self,
        command: WebDriverBiDiLocateNodesCommand,
        masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<
        (Self, ValidatedWebDriverBiDiLocateNodesResult),
        WebDriverBiDiLocateNodesExchangeError,
    > {
        let established = self
            .write_text_frame(command.as_json(), masking_key, frame_timeout)
            .map_err(WebDriverBiDiLocateNodesExchangeError::Frame)?;
        let (established, frame) = established
            .read_frame(frame_timeout)
            .map_err(WebDriverBiDiLocateNodesExchangeError::Frame)?;

        if !frame.fin() || frame.opcode() != 0x1 {
            return Err(
                WebDriverBiDiLocateNodesExchangeError::UnexpectedResponseFrame {
                    fin: frame.fin(),
                    opcode: frame.opcode(),
                },
            );
        }

        let document = BoundedWebDriverBiDiResponseDocument::from_utf8_bytes(frame.payload())
            .map_err(WebDriverBiDiLocateNodesExchangeError::ResponseDocument)?;
        let result = command
            .admit_response_document_nodes(document)
            .map_err(WebDriverBiDiLocateNodesExchangeError::LocateNodesResponse)?;
        Ok((established, result))
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error as _, time::Duration};

    use originweave_core::{
        WebDriverBiDiLocateNodesResponseDocumentError, WebDriverBiDiResponseDocumentAdmissionError,
    };

    use crate::{MAX_WEBSOCKET_FRAME_TIMEOUT, WebDriverBiDiWebSocketFrameError};

    use super::WebDriverBiDiLocateNodesExchangeError;

    #[test]
    fn exchange_errors_preserve_typed_sources_and_protocol_shape() {
        let frame = WebDriverBiDiLocateNodesExchangeError::Frame(
            WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
                frame_timeout: Duration::ZERO,
                maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
            },
        );
        assert!(frame.source().is_some());
        assert!(
            frame
                .to_string()
                .contains("WebSocket frame exchange failed")
        );

        let shape = WebDriverBiDiLocateNodesExchangeError::UnexpectedResponseFrame {
            fin: false,
            opcode: 0x2,
        };
        assert!(shape.source().is_none());
        assert!(shape.to_string().contains("fin=false, opcode=0x02"));

        let document = WebDriverBiDiLocateNodesExchangeError::ResponseDocument(
            WebDriverBiDiResponseDocumentAdmissionError::InvalidUtf8,
        );
        assert!(document.source().is_some());
        assert!(document.to_string().contains("raw-document admission"));

        let response = WebDriverBiDiLocateNodesExchangeError::LocateNodesResponse(
            WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodes,
        );
        assert!(response.source().is_some());
        assert!(response.to_string().contains("exact wire admission"));
    }
}
