use std::{error::Error, fmt};

use crate::{MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE, WebDriverBiDiWebSocketFrame};

/// Maximum UTF-8 payload bytes admitted for one assembled WebDriver BiDi WebSocket message.
///
/// The aggregate message bound intentionally matches the existing per-frame data bound so
/// fragmentation cannot be used to bypass the reviewed 1 MiB transport resource budget.
pub const MAX_WEBDRIVER_BIDI_MESSAGE_SIZE: usize = MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE;

/// Semantic kind of one bounded RFC 6455 control frame observed between BiDi message fragments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiWebSocketControlKind {
    /// The peer sent a Close control frame.
    Close,
    /// The peer sent a Ping control frame.
    Ping,
    /// The peer sent a Pong control frame.
    Pong,
}

/// One bounded WebSocket control message retained without exposing application text.
#[derive(Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketControlMessage {
    kind: WebDriverBiDiWebSocketControlKind,
    payload: Vec<u8>,
}

impl fmt::Debug for WebDriverBiDiWebSocketControlMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiWebSocketControlMessage")
            .field("kind", &self.kind)
            .field("payload_bytes", &self.payload.len())
            .finish()
    }
}

impl WebDriverBiDiWebSocketControlMessage {
    /// Return the exact control-frame kind.
    #[must_use]
    pub const fn kind(&self) -> WebDriverBiDiWebSocketControlKind {
        self.kind
    }

    /// Borrow the bounded control payload.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// One complete validated UTF-8 WebDriver BiDi WebSocket text message.
#[derive(Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketTextMessage(String);

impl fmt::Debug for WebDriverBiDiWebSocketTextMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiWebSocketTextMessage")
            .field("payload_bytes", &self.0.len())
            .finish()
    }
}

impl WebDriverBiDiWebSocketTextMessage {
    /// Borrow the complete validated UTF-8 message text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(test)]
    pub(crate) fn from_test_text(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Result of admitting one RFC 6455 frame into the bounded BiDi message assembler.
#[derive(Debug, Eq, PartialEq)]
pub enum WebDriverBiDiWebSocketMessageAssembly {
    /// A fragmented text message is still incomplete.
    Pending,
    /// One complete UTF-8 text message is ready for the later BiDi JSON layer.
    Text(WebDriverBiDiWebSocketTextMessage),
    /// One RFC 6455 control message was observed without disturbing partial text state.
    Control(WebDriverBiDiWebSocketControlMessage),
}

/// Fail-closed semantic failures while assembling WebDriver BiDi WebSocket text messages.
#[derive(Debug, Eq, PartialEq)]
pub enum WebDriverBiDiWebSocketMessageError {
    /// The assembler is terminal after a prior semantic failure or peer Close frame.
    AssemblerPoisoned,
    /// A continuation frame arrived without an active fragmented text message.
    UnexpectedContinuation,
    /// WebDriver BiDi requires text WebSocket messages; binary data is not admitted.
    UnexpectedBinaryMessage,
    /// A new text frame began before the active fragmented text message completed.
    InterruptedFragmentedText,
    /// Aggregate fragmented message bytes exceeded the reviewed resource bound.
    MessageTooLarge {
        /// Aggregate payload bytes that the attempted append would produce.
        payload_bytes: usize,
        /// Maximum aggregate payload bytes admitted by this boundary.
        maximum_bytes: usize,
    },
    /// The complete text message was not valid UTF-8.
    InvalidTextUtf8,
    /// A frame opcode escaped the lower RFC 6455 validation layer unexpectedly.
    UnsupportedFrameOpcode {
        /// Unexpected RFC 6455 opcode.
        opcode: u8,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssemblerPoisoned => formatter.write_str(
                "WebDriver BiDi WebSocket message assembly is terminal after failure or Close",
            ),
            Self::UnexpectedContinuation => formatter
                .write_str("WebDriver BiDi WebSocket continuation arrived without fragmented text"),
            Self::UnexpectedBinaryMessage => formatter.write_str(
                "WebDriver BiDi requires WebSocket text messages; binary message rejected",
            ),
            Self::InterruptedFragmentedText => formatter
                .write_str("WebDriver BiDi fragmented text was interrupted by a new data message"),
            Self::MessageTooLarge {
                payload_bytes,
                maximum_bytes,
            } => write!(
                formatter,
                "WebDriver BiDi WebSocket message has {payload_bytes} bytes; maximum is {maximum_bytes}"
            ),
            Self::InvalidTextUtf8 => {
                formatter.write_str("WebDriver BiDi WebSocket text message is not valid UTF-8")
            }
            Self::UnsupportedFrameOpcode { opcode } => write!(
                formatter,
                "unexpected RFC 6455 opcode escaped frame validation: {opcode:#04x}"
            ),
        }
    }
}

impl Error for WebDriverBiDiWebSocketMessageError {}

/// Stateful bounded assembler for the text-message semantics required by WebDriver BiDi.
///
/// The lower frame layer owns RFC 6455 framing and wire validation. This layer only joins text and
/// continuation payloads, preserves interleaved control frames, rejects binary messages, validates
/// UTF-8 after the complete message exists, and enforces an aggregate 1 MiB message budget. Any
/// semantic protocol failure makes the assembler terminal so callers cannot accidentally recover
/// authority from a corrupted message sequence. A peer Close frame is returned once and likewise
/// makes subsequent message assembly terminal.
pub struct WebDriverBiDiWebSocketMessageAssembler {
    fragmented_text: Option<Vec<u8>>,
    poisoned: bool,
}

impl fmt::Debug for WebDriverBiDiWebSocketMessageAssembler {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiWebSocketMessageAssembler")
            .field(
                "fragmented_payload_bytes",
                &self.fragmented_text.as_ref().map_or(0, Vec::len),
            )
            .field("terminal", &self.poisoned)
            .finish()
    }
}

impl WebDriverBiDiWebSocketMessageAssembler {
    /// Create one empty assembler with no inherited message state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fragmented_text: None,
            poisoned: false,
        }
    }

    /// Admit one already-validated RFC 6455 frame into the BiDi text-message state machine.
    pub fn push_frame(
        &mut self,
        frame: WebDriverBiDiWebSocketFrame,
    ) -> Result<WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketMessageError> {
        self.push_parts(frame.fin(), frame.opcode(), frame.payload())
    }

    fn push_parts(
        &mut self,
        fin: bool,
        opcode: u8,
        payload: &[u8],
    ) -> Result<WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketMessageError> {
        if self.poisoned {
            return Err(WebDriverBiDiWebSocketMessageError::AssemblerPoisoned);
        }
        match opcode {
            0x0 => self.push_continuation(fin, payload),
            0x1 => self.push_text(fin, payload),
            0x2 => self.reject(WebDriverBiDiWebSocketMessageError::UnexpectedBinaryMessage),
            0x8 => {
                self.fragmented_text = None;
                self.poisoned = true;
                Ok(WebDriverBiDiWebSocketMessageAssembly::Control(
                    WebDriverBiDiWebSocketControlMessage {
                        kind: WebDriverBiDiWebSocketControlKind::Close,
                        payload: payload.to_vec(),
                    },
                ))
            }
            0x9 => Ok(Self::control(
                WebDriverBiDiWebSocketControlKind::Ping,
                payload,
            )),
            0xa => Ok(Self::control(
                WebDriverBiDiWebSocketControlKind::Pong,
                payload,
            )),
            _ => self.reject(WebDriverBiDiWebSocketMessageError::UnsupportedFrameOpcode { opcode }),
        }
    }

    fn push_text(
        &mut self,
        fin: bool,
        payload: &[u8],
    ) -> Result<WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketMessageError> {
        if self.fragmented_text.is_some() {
            return self.reject(WebDriverBiDiWebSocketMessageError::InterruptedFragmentedText);
        }
        if payload.len() > MAX_WEBDRIVER_BIDI_MESSAGE_SIZE {
            return self.reject(WebDriverBiDiWebSocketMessageError::MessageTooLarge {
                payload_bytes: payload.len(),
                maximum_bytes: MAX_WEBDRIVER_BIDI_MESSAGE_SIZE,
            });
        }
        if fin {
            return Self::complete_text(payload.to_vec())
                .map(WebDriverBiDiWebSocketMessageAssembly::Text)
                .inspect_err(|_| {
                    self.fragmented_text = None;
                    self.poisoned = true;
                });
        }
        self.fragmented_text = Some(payload.to_vec());
        Ok(WebDriverBiDiWebSocketMessageAssembly::Pending)
    }

    fn push_continuation(
        &mut self,
        fin: bool,
        payload: &[u8],
    ) -> Result<WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketMessageError> {
        let Some(mut buffer) = self.fragmented_text.take() else {
            return self.reject(WebDriverBiDiWebSocketMessageError::UnexpectedContinuation);
        };
        let current_len = buffer.len();
        if payload.len() > MAX_WEBDRIVER_BIDI_MESSAGE_SIZE - current_len {
            return self.reject(WebDriverBiDiWebSocketMessageError::MessageTooLarge {
                payload_bytes: current_len.saturating_add(payload.len()),
                maximum_bytes: MAX_WEBDRIVER_BIDI_MESSAGE_SIZE,
            });
        }
        buffer.extend_from_slice(payload);
        if !fin {
            self.fragmented_text = Some(buffer);
            return Ok(WebDriverBiDiWebSocketMessageAssembly::Pending);
        }
        Self::complete_text(buffer)
            .map(WebDriverBiDiWebSocketMessageAssembly::Text)
            .inspect_err(|_| {
                self.poisoned = true;
            })
    }

    fn complete_text(
        payload: Vec<u8>,
    ) -> Result<WebDriverBiDiWebSocketTextMessage, WebDriverBiDiWebSocketMessageError> {
        String::from_utf8(payload)
            .map(WebDriverBiDiWebSocketTextMessage)
            .map_err(|_| WebDriverBiDiWebSocketMessageError::InvalidTextUtf8)
    }

    fn control(
        kind: WebDriverBiDiWebSocketControlKind,
        payload: &[u8],
    ) -> WebDriverBiDiWebSocketMessageAssembly {
        WebDriverBiDiWebSocketMessageAssembly::Control(WebDriverBiDiWebSocketControlMessage {
            kind,
            payload: payload.to_vec(),
        })
    }

    fn reject<T>(
        &mut self,
        error: WebDriverBiDiWebSocketMessageError,
    ) -> Result<T, WebDriverBiDiWebSocketMessageError> {
        self.fragmented_text = None;
        self.poisoned = true;
        Err(error)
    }
}

impl Default for WebDriverBiDiWebSocketMessageAssembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_message(value: &str) -> WebDriverBiDiWebSocketTextMessage {
        WebDriverBiDiWebSocketTextMessage(value.to_owned())
    }

    fn control_message(
        kind: WebDriverBiDiWebSocketControlKind,
        payload: &[u8],
    ) -> WebDriverBiDiWebSocketControlMessage {
        WebDriverBiDiWebSocketControlMessage {
            kind,
            payload: payload.to_vec(),
        }
    }

    #[test]
    fn complete_text_and_debug_are_payload_redacted() {
        let mut assembler = WebDriverBiDiWebSocketMessageAssembler::default();
        assert_eq!(
            assembler.push_parts(true, 0x1, b"secret-text"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Text(text_message(
                "secret-text"
            )))
        );
        let message = text_message("secret-text");
        assert_eq!(message.as_str(), "secret-text");
        let message_debug = format!("{message:?}");
        assert!(message_debug.contains("payload_bytes: 11"));
        assert!(!message_debug.contains("secret-text"));
        let assembler_debug = format!("{assembler:?}");
        assert!(assembler_debug.contains("fragmented_payload_bytes: 0"));
        assert!(assembler_debug.contains("terminal: false"));
    }

    #[test]
    fn fragments_reassemble_only_after_final_continuation() {
        let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
        assert_eq!(
            assembler.push_parts(false, 0x1, b"A\xe2"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Pending)
        );
        assert_eq!(
            assembler.push_parts(false, 0x0, b"\x82"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Pending)
        );
        assert_eq!(
            assembler.push_parts(true, 0x0, b"\xacB"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Text(text_message(
                "A€B"
            )))
        );
    }

    #[test]
    fn ping_and_pong_preserve_fragmented_text_state() {
        let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
        assert_eq!(
            assembler.push_parts(false, 0x1, b"left-"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Pending)
        );
        assert_eq!(
            assembler.push_parts(true, 0x9, b"ping-data"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Control(
                control_message(WebDriverBiDiWebSocketControlKind::Ping, b"ping-data")
            ))
        );
        let ping = control_message(WebDriverBiDiWebSocketControlKind::Ping, b"ping-data");
        assert_eq!(ping.kind(), WebDriverBiDiWebSocketControlKind::Ping);
        assert_eq!(ping.payload(), b"ping-data");
        let ping_debug = format!("{ping:?}");
        assert!(ping_debug.contains("payload_bytes: 9"));
        assert!(!ping_debug.contains("ping-data"));

        assert_eq!(
            assembler.push_parts(true, 0xa, b"pong"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Control(
                control_message(WebDriverBiDiWebSocketControlKind::Pong, b"pong")
            ))
        );
        let pong = control_message(WebDriverBiDiWebSocketControlKind::Pong, b"pong");
        assert_eq!(pong.kind(), WebDriverBiDiWebSocketControlKind::Pong);
        assert_eq!(pong.payload(), b"pong");
        assert_eq!(
            assembler.push_parts(true, 0x0, b"right"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Text(text_message(
                "left-right"
            )))
        );
    }

    #[test]
    fn close_is_returned_once_and_makes_assembler_terminal() {
        let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
        assert_eq!(
            assembler.push_parts(false, 0x1, b"discarded"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Pending)
        );
        assert_eq!(
            assembler.push_parts(true, 0x8, b"bye"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Control(
                control_message(WebDriverBiDiWebSocketControlKind::Close, b"bye")
            ))
        );
        let close = control_message(WebDriverBiDiWebSocketControlKind::Close, b"bye");
        assert_eq!(close.kind(), WebDriverBiDiWebSocketControlKind::Close);
        assert_eq!(close.payload(), b"bye");
        assert_eq!(
            assembler.push_parts(true, 0x1, b"later"),
            Err(WebDriverBiDiWebSocketMessageError::AssemblerPoisoned)
        );
    }

    #[test]
    fn semantic_data_sequence_errors_fail_closed() {
        let cases = [
            (
                0x0,
                WebDriverBiDiWebSocketMessageError::UnexpectedContinuation,
            ),
            (
                0x2,
                WebDriverBiDiWebSocketMessageError::UnexpectedBinaryMessage,
            ),
            (
                0x3,
                WebDriverBiDiWebSocketMessageError::UnsupportedFrameOpcode { opcode: 0x3 },
            ),
        ];
        for (opcode, expected) in cases {
            let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
            assert_eq!(assembler.push_parts(true, opcode, b"x"), Err(expected));
            assert_eq!(
                assembler.push_parts(true, 0x1, b"later"),
                Err(WebDriverBiDiWebSocketMessageError::AssemblerPoisoned)
            );
        }

        let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
        assert_eq!(
            assembler.push_parts(false, 0x1, b"partial"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Pending)
        );
        assert_eq!(
            assembler.push_parts(true, 0x1, b"new-message"),
            Err(WebDriverBiDiWebSocketMessageError::InterruptedFragmentedText)
        );
    }

    #[test]
    fn aggregate_message_bound_rejects_fragmentation_bypass() {
        let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
        let maximum = vec![b'x'; MAX_WEBDRIVER_BIDI_MESSAGE_SIZE];
        assert_eq!(
            assembler.push_parts(false, 0x1, &maximum),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Pending)
        );
        assert_eq!(
            assembler.push_parts(true, 0x0, b"y"),
            Err(WebDriverBiDiWebSocketMessageError::MessageTooLarge {
                payload_bytes: MAX_WEBDRIVER_BIDI_MESSAGE_SIZE + 1,
                maximum_bytes: MAX_WEBDRIVER_BIDI_MESSAGE_SIZE,
            })
        );

        let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
        let oversized = vec![b'x'; MAX_WEBDRIVER_BIDI_MESSAGE_SIZE + 1];
        assert_eq!(
            assembler.push_parts(false, 0x1, &oversized),
            Err(WebDriverBiDiWebSocketMessageError::MessageTooLarge {
                payload_bytes: MAX_WEBDRIVER_BIDI_MESSAGE_SIZE + 1,
                maximum_bytes: MAX_WEBDRIVER_BIDI_MESSAGE_SIZE,
            })
        );
    }

    #[test]
    fn utf8_validation_waits_for_complete_message_and_then_fails_closed() {
        let mut assembler = WebDriverBiDiWebSocketMessageAssembler::new();
        assert_eq!(
            assembler.push_parts(true, 0x1, b"\xff"),
            Err(WebDriverBiDiWebSocketMessageError::InvalidTextUtf8)
        );
        assert_eq!(
            assembler.push_parts(true, 0x1, b"later"),
            Err(WebDriverBiDiWebSocketMessageError::AssemblerPoisoned)
        );

        let mut fragmented = WebDriverBiDiWebSocketMessageAssembler::new();
        assert_eq!(
            fragmented.push_parts(false, 0x1, b"\xe2"),
            Ok(WebDriverBiDiWebSocketMessageAssembly::Pending)
        );
        assert_eq!(
            fragmented.push_parts(true, 0x0, b"x"),
            Err(WebDriverBiDiWebSocketMessageError::InvalidTextUtf8)
        );
    }

    #[test]
    fn public_error_contract_is_stable_and_source_free() {
        let cases = [
            (
                WebDriverBiDiWebSocketMessageError::AssemblerPoisoned,
                "WebDriver BiDi WebSocket message assembly is terminal after failure or Close",
            ),
            (
                WebDriverBiDiWebSocketMessageError::UnexpectedContinuation,
                "WebDriver BiDi WebSocket continuation arrived without fragmented text",
            ),
            (
                WebDriverBiDiWebSocketMessageError::UnexpectedBinaryMessage,
                "WebDriver BiDi requires WebSocket text messages; binary message rejected",
            ),
            (
                WebDriverBiDiWebSocketMessageError::InterruptedFragmentedText,
                "WebDriver BiDi fragmented text was interrupted by a new data message",
            ),
            (
                WebDriverBiDiWebSocketMessageError::InvalidTextUtf8,
                "WebDriver BiDi WebSocket text message is not valid UTF-8",
            ),
            (
                WebDriverBiDiWebSocketMessageError::UnsupportedFrameOpcode { opcode: 0x3 },
                "unexpected RFC 6455 opcode escaped frame validation: 0x03",
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
            assert!(error.source().is_none());
        }
        let too_large = WebDriverBiDiWebSocketMessageError::MessageTooLarge {
            payload_bytes: 10,
            maximum_bytes: 9,
        };
        assert_eq!(
            too_large.to_string(),
            "WebDriver BiDi WebSocket message has 10 bytes; maximum is 9"
        );
        assert!(too_large.source().is_none());
    }
}
