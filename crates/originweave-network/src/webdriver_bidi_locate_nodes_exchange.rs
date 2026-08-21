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
/// Protocol-shape, exhausted-deadline, and missing caller entropy refusals have no nested source
/// because none masks an underlying I/O or parser failure.
#[derive(Debug)]
pub enum WebDriverBiDiLocateNodesExchangeError {
    /// Bounded WebSocket frame write or read failed.
    Frame(WebDriverBiDiWebSocketFrameError),
    /// The single end-to-end exchange deadline was exhausted before the next operation could proceed.
    ExchangeDeadlineExceeded {
        /// Original caller-supplied deadline budget for the complete exchange.
        exchange_timeout: Duration,
    },
    /// A server Ping required a fresh client masking key, but the caller supplied none.
    PongMaskingKeyUnavailable,
    /// The returned frame was neither an admissible control frame nor one complete text response.
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
                "WebDriver BiDi locateNodes exchange exhausted its {exchange_timeout:?} end-to-end deadline before the next operation"
            ),
            Self::PongMaskingKeyUnavailable => formatter.write_str(
                "WebDriver BiDi locateNodes exchange received Ping without a fresh caller-supplied Pong masking key",
            ),
            Self::UnexpectedResponseFrame { fin, opcode } => write!(
                formatter,
                "WebDriver BiDi locateNodes exchange requires control handling or one final text response frame; received fin={fin}, opcode=0x{opcode:02x}"
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
            Self::ExchangeDeadlineExceeded { .. }
            | Self::PongMaskingKeyUnavailable
            | Self::UnexpectedResponseFrame { .. } => None,
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

fn next_pong_masking_key(
    next_key: &mut dyn FnMut() -> Option<WebDriverBiDiWebSocketMaskKey>,
) -> Result<WebDriverBiDiWebSocketMaskKey, WebDriverBiDiLocateNodesExchangeError> {
    next_key().ok_or(WebDriverBiDiLocateNodesExchangeError::PongMaskingKeyUnavailable)
}

fn map_established_frame_result(
    result: Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError>,
) -> Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiLocateNodesExchangeError> {
    result.map_err(WebDriverBiDiLocateNodesExchangeError::Frame)
}

impl WebDriverBiDiWebSocketEstablished {
    /// Exchange one exact bounded `browsingContext.locateNodes` command on this verified stream.
    ///
    /// The command is serialized by the reviewed core boundary and written as one masked client
    /// text frame using `command_masking_key`. Valid server Ping frames are answered with a masked
    /// Pong carrying the exact Ping application data, while unsolicited valid Pong frames are
    /// consumed without changing BiDi state. Each Ping obtains a fresh unpredictable client mask
    /// from `next_pong_key`; exhausting that caller-owned entropy source fails closed and
    /// consumes the transport rather than reusing a masking key. Close, binary, continuation,
    /// fragmented data, and reserved shapes are not reinterpreted as a BiDi response.
    ///
    /// `exchange_timeout` is one end-to-end budget for every command write, control-frame read/write,
    /// and response read. Elapsed time is subtracted before every subsequent operation and the budget
    /// is never reset. The underlying frame boundary independently caps each frame at its existing
    /// size ceiling, while the single exchange deadline bounds a peer that sends repeated valid
    /// control frames. Any failure consumes this transport state and yields no reusable WebSocket
    /// stream, preventing a partially written/read protocol state from becoming later authority.
    ///
    /// The final complete text payload passes the existing bounded UTF-8/document admission,
    /// complete WebDriver BiDi response parser, exact command-id correlation, and wire-derived node
    /// admission. Success returns the same exact peer-verified WebSocket stream plus untrusted
    /// normalized node evidence. It does not authenticate Chromium/ChromeDriver process provenance,
    /// prove current OriginWeave session/context/origin/document authority, authorize policy or typed
    /// input, mint node handles, execute a browser action, or prove a post-condition.
    pub fn exchange_locate_nodes(
        self,
        command: WebDriverBiDiLocateNodesCommand,
        command_masking_key: WebDriverBiDiWebSocketMaskKey,
        next_pong_key: &mut dyn FnMut() -> Option<WebDriverBiDiWebSocketMaskKey>,
        exchange_timeout: Duration,
    ) -> Result<
        (Self, ValidatedWebDriverBiDiLocateNodesResult),
        WebDriverBiDiLocateNodesExchangeError,
    > {
        let started_at = Instant::now();
        let mut established = map_established_frame_result(self.write_text_frame(
            command.as_json(),
            command_masking_key,
            exchange_timeout,
        ))?;

        loop {
            let remaining_timeout =
                remaining_exchange_budget(exchange_timeout, started_at.elapsed())?;
            let (next_established, frame) = established
                .read_frame(remaining_timeout)
                .map_err(WebDriverBiDiLocateNodesExchangeError::Frame)?;
            established = next_established;

            match frame.opcode() {
                0x9 => {
                    let masking_key = next_pong_masking_key(next_pong_key)?;
                    let remaining_timeout =
                        remaining_exchange_budget(exchange_timeout, started_at.elapsed())?;
                    established = map_established_frame_result(established.write_pong_frame(
                        frame.payload(),
                        masking_key,
                        remaining_timeout,
                    ))?;
                }
                0xa => {}
                0x1 if frame.fin() => {
                    let document =
                        BoundedWebDriverBiDiResponseDocument::from_utf8_bytes(frame.payload())
                            .map_err(WebDriverBiDiLocateNodesExchangeError::ResponseDocument)?;
                    let result = command
                        .admit_response_document_nodes(document)
                        .map_err(WebDriverBiDiLocateNodesExchangeError::LocateNodesResponse)?;
                    return Ok((established, result));
                }
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesExchangeError::UnexpectedResponseFrame {
                            fin: frame.fin(),
                            opcode: frame.opcode(),
                        },
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error as _, time::Duration};

    use originweave_core::{
        WebDriverBiDiLocateNodesResponseDocumentError, WebDriverBiDiResponseDocumentAdmissionError,
    };

    use crate::{MAX_WEBSOCKET_FRAME_TIMEOUT, WebDriverBiDiWebSocketFrameError};

    use super::{
        WebDriverBiDiLocateNodesExchangeError, next_pong_masking_key, remaining_exchange_budget,
    };

    #[test]
    fn exchange_budget_consumes_elapsed_time_instead_of_resetting() {
        let total = Duration::from_millis(500);
        assert_eq!(
            format!(
                "{:?}",
                remaining_exchange_budget(total, Duration::from_millis(175))
            ),
            "Ok(325ms)"
        );
        assert_eq!(
            format!("{:?}", remaining_exchange_budget(total, total)),
            "Err(ExchangeDeadlineExceeded { exchange_timeout: 500ms })"
        );
        assert_eq!(
            format!(
                "{:?}",
                remaining_exchange_budget(total, Duration::from_millis(501))
            ),
            "Err(ExchangeDeadlineExceeded { exchange_timeout: 500ms })"
        );
    }

    #[test]
    fn pong_masking_key_source_fails_closed_when_entropy_is_unavailable() {
        let expected = crate::WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]);
        let mut available = || Some(expected);
        assert_eq!(next_pong_masking_key(&mut available).ok(), Some(expected));

        let mut unavailable = || None;
        assert_eq!(
            format!("{:?}", next_pong_masking_key(&mut unavailable)),
            "Err(PongMaskingKeyUnavailable)"
        );
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

        let missing_mask = WebDriverBiDiLocateNodesExchangeError::PongMaskingKeyUnavailable;
        assert!(missing_mask.source().is_none());
        assert!(
            missing_mask
                .to_string()
                .contains("fresh caller-supplied Pong masking key")
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
