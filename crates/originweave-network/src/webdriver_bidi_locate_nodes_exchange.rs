use std::{
    error::Error,
    fmt,
    time::{Duration, Instant},
};

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextOriginEpochDispatchTarget, MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES,
    ObservedNodeHandle, ValidatedBrowserProtocolUse, ValidatedWebDriverBiDiLocateNodesResult,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiLocateNodesResponseDocumentError,
    WebDriverBiDiResponseDocumentAdmissionError,
};

use crate::{
    MAX_WEBSOCKET_FRAME_TIMEOUT, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketMaskKey,
};

/// Maximum number of valid RFC 6455 Ping/Pong control frames one `locateNodes` exchange will process.
///
/// RFC 6455 permits control frames to be interleaved with data frames; this OriginWeave-owned
/// resource budget prevents a peer from turning that permission into an unbounded control-frame loop
/// before the correlated BiDi response arrives. The end-to-end exchange deadline remains an
/// independent wall-clock bound.
pub const MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE: usize = 64;

/// Maximum number of data fragments accepted for one `locateNodes` response message.
///
/// RFC 6455 permits a text message to be split into an arbitrary number of continuation frames,
/// including empty fragments. This OriginWeave product-safety budget prevents a peer from turning
/// bounded response bytes into unbounded frame-processing work before the end-to-end deadline.
pub const MAX_WEBDRIVER_BIDI_RESPONSE_FRAGMENTS_PER_EXCHANGE: usize = 256;

/// Fail-closed failures while exchanging one bounded WebDriver BiDi `locateNodes` command.
///
/// Every variant preserves the first causal boundary. Frame I/O retains the existing bounded
/// WebSocket error, raw response bytes must pass the core pre-parser admission contract, and the
/// admitted document must correlate to the exact consumed command before result nodes are returned.
/// Protocol-shape, resource-budget, exhausted-deadline, missing caller entropy, and adjacent client
/// masking-key reuse refusals have no nested source because none masks an underlying I/O or parser
/// failure.
#[derive(Debug)]
pub enum WebDriverBiDiLocateNodesExchangeError {
    /// Bounded WebSocket frame write or read failed.
    Frame(WebDriverBiDiWebSocketFrameError),
    /// The single end-to-end exchange deadline was exhausted before the next operation could proceed.
    ExchangeDeadlineExceeded {
        /// Original caller-supplied deadline budget for the complete exchange.
        exchange_timeout: Duration,
    },
    /// The peer exceeded the local resource budget for interleaved Ping/Pong frames.
    ControlFrameLimitExceeded {
        /// Maximum number of control frames admitted for one exchange.
        maximum_control_frames: usize,
    },
    /// The peer exceeded the local resource budget for response-message data fragments.
    ResponseFragmentLimitExceeded {
        /// Maximum number of response-message data fragments admitted for one exchange.
        maximum_fragments: usize,
    },
    /// A server Ping required a fresh client masking key, but the caller supplied none.
    PongMaskingKeyUnavailable,
    /// A caller supplied the same Pong masking key as the immediately preceding client frame.
    PongMaskingKeyReused,
    /// The returned frame could not continue the one admissible text response message.
    UnexpectedResponseFrame {
        /// Whether the returned frame carried the RFC 6455 FIN bit.
        fin: bool,
        /// Exact returned RFC 6455 opcode.
        opcode: u8,
    },
    /// The exact response-message payload failed bounded raw-document admission.
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
            Self::ControlFrameLimitExceeded {
                maximum_control_frames,
            } => write!(
                formatter,
                "WebDriver BiDi locateNodes exchange exceeded the maximum {maximum_control_frames} interleaved control frames"
            ),
            Self::ResponseFragmentLimitExceeded { maximum_fragments } => write!(
                formatter,
                "WebDriver BiDi locateNodes exchange exceeded the maximum {maximum_fragments} response-message data fragments"
            ),
            Self::PongMaskingKeyUnavailable => formatter.write_str(
                "WebDriver BiDi locateNodes exchange received Ping without a fresh caller-supplied Pong masking key",
            ),
            Self::PongMaskingKeyReused => formatter.write_str(
                "WebDriver BiDi locateNodes exchange refused a Pong masking key matching the immediately preceding client frame",
            ),
            Self::UnexpectedResponseFrame { fin, opcode } => write!(
                formatter,
                "WebDriver BiDi locateNodes exchange requires control handling or one bounded text response message; received fin={fin}, opcode=0x{opcode:02x}"
            ),
            Self::ResponseDocument(error) => write!(
                formatter,
                "WebDriver BiDi locateNodes response message failed raw-document admission: {error}"
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
            | Self::ControlFrameLimitExceeded { .. }
            | Self::ResponseFragmentLimitExceeded { .. }
            | Self::PongMaskingKeyUnavailable
            | Self::PongMaskingKeyReused
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

fn remaining_frame_operation_budget(
    exchange_timeout: Duration,
    elapsed: Duration,
) -> Result<Duration, WebDriverBiDiLocateNodesExchangeError> {
    remaining_exchange_budget(exchange_timeout, elapsed)
        .map(|remaining| remaining.min(MAX_WEBSOCKET_FRAME_TIMEOUT))
}

fn next_pong_masking_key(
    next_key: &mut dyn FnMut() -> Option<WebDriverBiDiWebSocketMaskKey>,
) -> Result<WebDriverBiDiWebSocketMaskKey, WebDriverBiDiLocateNodesExchangeError> {
    next_key().ok_or(WebDriverBiDiLocateNodesExchangeError::PongMaskingKeyUnavailable)
}

fn next_pong_masking_key_before_deadline(
    next_key: &mut dyn FnMut() -> Option<WebDriverBiDiWebSocketMaskKey>,
    exchange_timeout: Duration,
    elapsed: Duration,
) -> Result<WebDriverBiDiWebSocketMaskKey, WebDriverBiDiLocateNodesExchangeError> {
    remaining_frame_operation_budget(exchange_timeout, elapsed)
        .and_then(|_| next_pong_masking_key(next_key))
}

fn map_established_frame_result(
    result: Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError>,
) -> Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiLocateNodesExchangeError> {
    result.map_err(WebDriverBiDiLocateNodesExchangeError::Frame)
}

fn read_frame_with_exchange_budget<T>(
    exchange_timeout: Duration,
    elapsed: Duration,
    read_frame: impl FnOnce(Duration) -> Result<T, WebDriverBiDiWebSocketFrameError>,
) -> Result<T, WebDriverBiDiLocateNodesExchangeError> {
    let remaining_timeout = remaining_frame_operation_budget(exchange_timeout, elapsed)?;
    read_frame(remaining_timeout).map_err(WebDriverBiDiLocateNodesExchangeError::Frame)
}

fn admit_response_fragment(
    response_fragment_count: &mut usize,
) -> Result<(), WebDriverBiDiLocateNodesExchangeError> {
    if *response_fragment_count == MAX_WEBDRIVER_BIDI_RESPONSE_FRAGMENTS_PER_EXCHANGE {
        return Err(
            WebDriverBiDiLocateNodesExchangeError::ResponseFragmentLimitExceeded {
                maximum_fragments: MAX_WEBDRIVER_BIDI_RESPONSE_FRAGMENTS_PER_EXCHANGE,
            },
        );
    }
    *response_fragment_count += 1;
    Ok(())
}

fn append_response_fragment(
    response_message: &mut Vec<u8>,
    payload: &[u8],
) -> Result<(), WebDriverBiDiLocateNodesExchangeError> {
    if response_message.len().saturating_add(payload.len())
        > MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES
    {
        return Err(WebDriverBiDiLocateNodesExchangeError::ResponseDocument(
            WebDriverBiDiResponseDocumentAdmissionError::DocumentTooLarge,
        ));
    }
    response_message.extend_from_slice(payload);
    Ok(())
}

fn admit_response_payload(
    command: WebDriverBiDiLocateNodesCommand,
    payload: &[u8],
) -> Result<ValidatedWebDriverBiDiLocateNodesResult, WebDriverBiDiLocateNodesExchangeError> {
    let document = BoundedWebDriverBiDiResponseDocument::from_utf8_bytes(payload)
        .map_err(WebDriverBiDiLocateNodesExchangeError::ResponseDocument)?;
    command
        .admit_response_document_nodes(document)
        .map_err(WebDriverBiDiLocateNodesExchangeError::LocateNodesResponse)
}

impl WebDriverBiDiWebSocketEstablished {
    /// Exchange one exact bounded `browsingContext.locateNodes` command on this verified stream.
    ///
    /// The command is serialized by the reviewed core boundary and written as one masked client
    /// text frame using `command_masking_key`. Valid server Ping frames are answered with a masked
    /// Pong carrying the exact Ping application data, while unsolicited valid Pong frames are
    /// consumed without changing BiDi state. Each Ping obtains a caller-supplied client mask from
    /// `next_pong_key` only after a positive remaining-budget check; exhausting that caller-owned
    /// entropy source or repeating the immediately preceding successful client-frame key fails
    /// closed before another client frame is emitted. A later random collision after a different
    /// client key remains admissible; the caller is responsible for deriving every key independently
    /// from a strong unpredictable entropy source. Callback time is charged by a second deadline
    /// check before the Pong write, and the adjacent-key guard does not claim to prove cryptographic
    /// unpredictability.
    ///
    /// RFC 6455 text-message fragmentation is reassembled only for one response message at a time.
    /// A non-final text frame starts that message, continuation frames extend it in order, and a final
    /// continuation completes it. Ping/Pong control frames remain admissible between fragments. The
    /// total assembled response is capped by [`MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES`] before
    /// allocation can grow beyond the existing pre-parser budget, and at most
    /// [`MAX_WEBDRIVER_BIDI_RESPONSE_FRAGMENTS_PER_EXCHANGE`] accepted data fragments may compose the
    /// message, so empty continuation frames cannot create unbounded processing work. Orphan
    /// continuations, a second data message before completion, binary/Close/reserved shapes, and
    /// malformed frame sequences fail closed and consume the transport state.
    ///
    /// `exchange_timeout` is one end-to-end budget for every command write, control-frame read/write,
    /// response-fragment read, and response read. Elapsed time is subtracted before every operation,
    /// including the initial command write, and the budget is never reset. Each individual frame
    /// operation is additionally capped at the established frame timeout ceiling, so a longer
    /// end-to-end exchange budget remains valid without widening the per-operation I/O bound. The
    /// underlying frame boundary independently caps each frame at its existing size ceiling. In
    /// addition, at most [`MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE`] valid Ping/Pong frames are
    /// processed before the exchange fails closed, so RFC 6455 control-frame interleaving cannot
    /// create an unbounded iteration budget even when the wall-clock deadline has not yet expired.
    /// Any failure consumes this transport state and yields no reusable WebSocket stream, preventing
    /// a partially written/read protocol state from becoming later authority.
    ///
    /// The final complete text message passes the existing bounded UTF-8/document admission,
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
        let write_timeout =
            remaining_frame_operation_budget(exchange_timeout, started_at.elapsed())?;
        let mut established = map_established_frame_result(self.write_text_frame(
            command.as_json(),
            command_masking_key,
            write_timeout,
        ))?;
        let mut control_frame_count = 0_usize;
        let mut previous_client_masking_key = command_masking_key;
        let mut response_fragment_count = 0_usize;
        let mut response_message = Vec::new();
        let mut assembling_text_response = false;

        loop {
            let (next_established, frame) = read_frame_with_exchange_budget(
                exchange_timeout,
                started_at.elapsed(),
                |remaining_timeout| established.read_frame(remaining_timeout),
            )?;
            established = next_established;
            let opcode = frame.opcode();

            if matches!(opcode, 0x9 | 0xa) {
                if control_frame_count == MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE {
                    return Err(
                        WebDriverBiDiLocateNodesExchangeError::ControlFrameLimitExceeded {
                            maximum_control_frames: MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE,
                        },
                    );
                }
                control_frame_count += 1;
            }

            match opcode {
                0x9 => {
                    let masking_key = next_pong_masking_key_before_deadline(
                        next_pong_key,
                        exchange_timeout,
                        started_at.elapsed(),
                    )?;
                    if previous_client_masking_key == masking_key {
                        return Err(WebDriverBiDiLocateNodesExchangeError::PongMaskingKeyReused);
                    }
                    let remaining_timeout =
                        remaining_frame_operation_budget(exchange_timeout, started_at.elapsed())?;
                    established = map_established_frame_result(established.write_pong_frame(
                        frame.payload(),
                        masking_key,
                        remaining_timeout,
                    ))?;
                    previous_client_masking_key = masking_key;
                }
                0xa => {}
                0x1 if !assembling_text_response && frame.fin() => {
                    let result = admit_response_payload(command, frame.payload())?;
                    return Ok((established, result));
                }
                0x1 if !assembling_text_response => {
                    response_fragment_count = 1;
                    append_response_fragment(&mut response_message, frame.payload())?;
                    assembling_text_response = true;
                }
                0x0 if assembling_text_response => {
                    admit_response_fragment(&mut response_fragment_count)?;
                    append_response_fragment(&mut response_message, frame.payload())?;
                    if frame.fin() {
                        let result = admit_response_payload(command, &response_message)?;
                        return Ok((established, result));
                    }
                }
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesExchangeError::UnexpectedResponseFrame {
                            fin: frame.fin(),
                            opcode,
                        },
                    );
                }
            }
        }
    }

    /// Exchange `locateNodes` and bind the exact wire-derived nodes to current browser authority.
    ///
    /// This is the live transport composition boundary for semantic node observation. The bounded
    /// command is exchanged on the already peer-verified WebSocket using [`Self::exchange_locate_nodes`].
    /// Only after exact wire parsing and command correlation succeed does the method revalidate the
    /// reviewed WebDriver BiDi `SemanticObservation` proof and exact current
    /// session/context/origin/document epoch carried together in `authority` through
    /// [`ValidatedWebDriverBiDiLocateNodesResult::bind_current_nodes`]. No raw node identifier can be
    /// substituted between the wire response and authority binding.
    ///
    /// A binding failure consumes this transport result and returns no reusable stream or node
    /// handle, so a navigation or authority change observed after command construction cannot be
    /// converted into stale node authority. Success returns only current [`ObservedNodeHandle`]
    /// values together with the same established peer-verified stream. It still does not authorize
    /// typed input, execute an action, or prove a post-condition.
    pub fn exchange_locate_nodes_and_bind_current_nodes(
        self,
        command: WebDriverBiDiLocateNodesCommand,
        command_masking_key: WebDriverBiDiWebSocketMaskKey,
        next_pong_key: &mut dyn FnMut() -> Option<WebDriverBiDiWebSocketMaskKey>,
        exchange_timeout: Duration,
        authority: (
            ValidatedBrowserProtocolUse,
            BrowserContextOriginEpochDispatchTarget<'_>,
        ),
        authority_registry: &mut BrowserAuthorityRegistry,
    ) -> Result<(Self, Vec<ObservedNodeHandle>), WebDriverBiDiLocateNodesExchangeError> {
        let (validated, target) = authority;
        let (established, result) = self.exchange_locate_nodes(
            command,
            command_masking_key,
            next_pong_key,
            exchange_timeout,
        )?;
        let handles = match result.bind_current_nodes(validated, authority_registry, target) {
            Ok(handles) => handles,
            Err(error) => {
                return Err(WebDriverBiDiLocateNodesExchangeError::LocateNodesResponse(
                    WebDriverBiDiLocateNodesResponseDocumentError::NodeBinding(error),
                ));
            }
        };
        Ok((established, handles))
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error as _, time::Duration};

    use originweave_core::{
        MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES, WebDriverBiDiLocateNodesResponseDocumentError,
        WebDriverBiDiResponseDocumentAdmissionError,
    };

    use crate::{MAX_WEBSOCKET_FRAME_TIMEOUT, WebDriverBiDiWebSocketFrameError};

    use super::{
        MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE,
        MAX_WEBDRIVER_BIDI_RESPONSE_FRAGMENTS_PER_EXCHANGE, WebDriverBiDiLocateNodesExchangeError,
        append_response_fragment, next_pong_masking_key, next_pong_masking_key_before_deadline,
        read_frame_with_exchange_budget, remaining_exchange_budget,
        remaining_frame_operation_budget,
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
    fn exchange_budget_caps_each_frame_operation_without_resetting_total_time() {
        assert_eq!(
            remaining_frame_operation_budget(Duration::from_secs(6), Duration::ZERO).ok(),
            Some(MAX_WEBSOCKET_FRAME_TIMEOUT)
        );
        assert_eq!(
            remaining_frame_operation_budget(Duration::from_secs(6), Duration::from_secs(2)).ok(),
            Some(Duration::from_secs(4))
        );
        assert!(
            remaining_frame_operation_budget(Duration::from_secs(6), Duration::from_secs(6))
                .is_err()
        );
    }

    #[test]
    fn expired_exchange_budget_refuses_frame_read_before_io() {
        use std::cell::Cell;

        let exchange_timeout = Duration::from_millis(500);
        let read_count = Cell::new(0_usize);
        let read_frame = |remaining_timeout| {
            read_count.set(read_count.get() + 1);
            Ok::<Duration, WebDriverBiDiWebSocketFrameError>(remaining_timeout)
        };
        let available = read_frame_with_exchange_budget(
            exchange_timeout,
            Duration::from_millis(100),
            read_frame,
        );
        assert_eq!(available.ok(), Some(Duration::from_millis(400)));
        assert_eq!(read_count.get(), 1);

        let expired =
            read_frame_with_exchange_budget(exchange_timeout, exchange_timeout, read_frame);
        assert_eq!(
            format!("{expired:?}"),
            "Err(ExchangeDeadlineExceeded { exchange_timeout: 500ms })"
        );
        assert_eq!(read_count.get(), 1);

        let frame_error =
            read_frame_with_exchange_budget(exchange_timeout, Duration::from_millis(100), |_| {
                Err::<Duration, WebDriverBiDiWebSocketFrameError>(
                    WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
                        frame_timeout: Duration::ZERO,
                        maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
                    },
                )
            });
        assert!(frame_error.is_err());
    }

    #[test]
    fn response_fragment_buffer_never_exceeds_document_budget() {
        let mut response = vec![0_u8; MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES - 1];
        assert!(append_response_fragment(&mut response, b"x").is_ok());
        assert_eq!(response.len(), MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES);

        assert_eq!(
            format!("{:?}", append_response_fragment(&mut response, b"y")),
            "Err(ResponseDocument(DocumentTooLarge))"
        );
        assert_eq!(response.len(), MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES);
    }

    #[test]
    fn response_fragment_rejects_an_already_oversized_buffer_without_panicking() {
        let mut response = vec![0_u8; MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES + 1];
        assert!(append_response_fragment(&mut response, &[]).is_err());
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
    fn pong_entropy_is_not_drawn_after_exchange_deadline() {
        use std::cell::Cell;

        let expected = crate::WebDriverBiDiWebSocketMaskKey::new([0x11, 0x22, 0x33, 0x44]);
        let draw_count = Cell::new(0_usize);
        let mut next = || {
            draw_count.set(draw_count.get() + 1);
            Some(expected)
        };
        assert_eq!(next(), Some(expected));
        draw_count.set(0);
        let deadline_result = next_pong_masking_key_before_deadline(
            &mut next,
            Duration::from_millis(500),
            Duration::from_millis(500),
        );
        assert_eq!(
            format!("{deadline_result:?}"),
            "Err(ExchangeDeadlineExceeded { exchange_timeout: 500ms })"
        );
        assert_eq!(draw_count.get(), 0);

        let mut available = || Some(expected);
        assert_eq!(
            next_pong_masking_key_before_deadline(
                &mut available,
                Duration::from_millis(500),
                Duration::from_millis(100),
            )
            .ok(),
            Some(expected)
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

        let control_limit = WebDriverBiDiLocateNodesExchangeError::ControlFrameLimitExceeded {
            maximum_control_frames: MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE,
        };
        assert!(control_limit.source().is_none());
        assert!(
            control_limit
                .to_string()
                .contains("maximum 64 interleaved control frames")
        );

        let fragment_limit = WebDriverBiDiLocateNodesExchangeError::ResponseFragmentLimitExceeded {
            maximum_fragments: MAX_WEBDRIVER_BIDI_RESPONSE_FRAGMENTS_PER_EXCHANGE,
        };
        assert!(fragment_limit.source().is_none());
        assert!(
            fragment_limit
                .to_string()
                .contains("maximum 256 response-message data fragments")
        );

        let missing_mask = WebDriverBiDiLocateNodesExchangeError::PongMaskingKeyUnavailable;
        assert!(missing_mask.source().is_none());
        assert!(
            missing_mask
                .to_string()
                .contains("fresh caller-supplied Pong masking key")
        );

        let reused_mask = WebDriverBiDiLocateNodesExchangeError::PongMaskingKeyReused;
        assert!(reused_mask.source().is_none());
        assert!(
            reused_mask
                .to_string()
                .contains("Pong masking key matching the immediately preceding client frame")
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
