use std::{
    error::Error,
    fmt,
    time::{Duration, Instant},
};

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
/// Protocol-shape and exhausted-deadline refusals have no nested source because neither masks an
/// underlying I/O or parser failure.
#[derive(Debug)]
pub enum WebDriverBiDiLocateNodesExchangeError {
    /// Bounded WebSocket frame write or read failed.
    Frame(WebDriverBiDiWebSocketFrameError),
    /// The single end-to-end exchange deadline was exhausted before response read could proceed.
    ExchangeDeadlineExceeded {
        /// Original caller-supplied deadline budget for the complete write/read exchange.
        exchange_timeout: Duration,
    },
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
            Self::ExchangeDeadlineExceeded { exchange_timeout } => write!(
                formatter,
                "WebDriver BiDi locateNodes exchange exhausted its {exchange_timeout:?} end-to-end deadline before response read"
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
            Self::ExchangeDeadlineExceeded { .. } | Self::UnexpectedResponseFrame { .. } => None,
        }
    }
}

fn remaining_exchange_budget(
    exchange_timeout: Duration,
    elapsed: Duration,
) -> Result<Duration, WebDriverBiDiLocateNodesExchangeError> {
    match exchange_timeout.checked_sub(elapsed) {
        Some(remaining) if !remaining.is_zero() => Ok(remaining),
        Some(_) | None => Err(
            WebDriverBiDiLocateNodesExchangeError::ExchangeDeadlineExceeded { exchange_timeout },
        ),
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
    /// `exchange_timeout` is one end-to-end budget for the write/read exchange. The bounded write
    /// receives that budget first; after it succeeds, elapsed time is subtracted and only the
    /// positive remainder is supplied to the bounded read. The budget is never reset between those
    /// operations. Any failure consumes this transport state and yields no reusable WebSocket stream,
    /// preventing a partially written/read protocol state from being promoted into later authority.
    ///
    /// Success returns the same exact peer-verified WebSocket stream plus untrusted normalized node
    /// evidence. It does not authenticate Chromium/ChromeDriver process provenance, prove current
    /// OriginWeave session/context/origin/document authority, authorize policy or typed input, mint
    /// node handles, execute a browser action, or prove a post-condition.
    pub fn exchange_locate_nodes(
        self,
        command: WebDriverBiDiLocateNodesCommand,
        masking_key: WebDriverBiDiWebSocketMaskKey,
        exchange_timeout: Duration,
    ) -> Result<
        (Self, ValidatedWebDriverBiDiLocateNodesResult),
        WebDriverBiDiLocateNodesExchangeError,
    > {
        let started_at = Instant::now();
        let established = self
            .write_text_frame(command.as_json(), masking_key, exchange_timeout)
            .map_err(WebDriverBiDiLocateNodesExchangeError::Frame)?;
        let remaining_timeout = remaining_exchange_budget(exchange_timeout, started_at.elapsed())?;
        let (established, frame) = established
            .read_frame(remaining_timeout)
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

    use super::{WebDriverBiDiLocateNodesExchangeError, remaining_exchange_budget};

    #[test]
    fn exchange_budget_consumes_elapsed_time_instead_of_resetting_for_read() {
        let total = Duration::from_millis(500);
        assert!(matches!(
            remaining_exchange_budget(total, Duration::from_millis(175)),
            Ok(remaining) if remaining == Duration::from_millis(325)
        ));
        assert!(matches!(
            remaining_exchange_budget(total, total),
            Err(WebDriverBiDiLocateNodesExchangeError::ExchangeDeadlineExceeded {
                exchange_timeout
            }) if exchange_timeout == total
        ));
        assert!(matches!(
            remaining_exchange_budget(total, Duration::from_millis(501)),
            Err(WebDriverBiDiLocateNodesExchangeError::ExchangeDeadlineExceeded {
                exchange_timeout
            }) if exchange_timeout == total
        ));
    }

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

        let deadline = WebDriverBiDiLocateNodesExchangeError::ExchangeDeadlineExceeded {
            exchange_timeout: Duration::from_millis(500),
        };
        assert!(deadline.source().is_none());
        assert!(deadline.to_string().contains("end-to-end deadline"));

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
