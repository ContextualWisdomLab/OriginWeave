use std::{error::Error, fmt, time::Duration};

use crate::{
    WebDriverBiDiWebSocketControlMessage, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketMessageError,
    WebDriverBiDiWebSocketTextMessage,
    webdriver_bidi_connection::WebDriverBiDiConnectionGeneration,
};

/// One complete WebDriver BiDi text message bound to the exact verified connection that read it.
///
/// The connection generation is minted by the TCP connection owner and cannot be supplied by
/// callers. The wrapper therefore provides correlation provenance without granting browser, page,
/// policy, process, profile, or Agent authority. Debug output never exposes the message payload.
pub struct WebDriverBiDiReceivedTextMessage {
    message: WebDriverBiDiWebSocketTextMessage,
    connection_generation: WebDriverBiDiConnectionGeneration,
}

impl fmt::Debug for WebDriverBiDiReceivedTextMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiReceivedTextMessage")
            .field("payload_bytes", &self.message.as_str().len())
            .field("connection_bound", &true)
            .finish()
    }
}

impl WebDriverBiDiReceivedTextMessage {
    pub(crate) const fn message(&self) -> &WebDriverBiDiWebSocketTextMessage {
        &self.message
    }

    pub(crate) const fn connection_generation(&self) -> WebDriverBiDiConnectionGeneration {
        self.connection_generation
    }
}

/// Stateful message reader that keeps RFC 6455 fragmentation on one verified BiDi connection.
///
/// The reader owns both the live established connection and the message assembler. Every frame
/// admitted into a fragmented message is therefore read from that same non-cloneable connection;
/// callers cannot combine fragments from another socket and still obtain a connection-bound text
/// message. Interleaved control frames are surfaced without discarding partial message state.
pub struct WebDriverBiDiWebSocketMessageReader {
    established: WebDriverBiDiWebSocketEstablished,
    assembler: WebDriverBiDiWebSocketMessageAssembler,
    connection_generation: WebDriverBiDiConnectionGeneration,
}

impl fmt::Debug for WebDriverBiDiWebSocketMessageReader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiWebSocketMessageReader")
            .field("connection_bound", &true)
            .field("assembler", &self.assembler)
            .finish()
    }
}

impl WebDriverBiDiWebSocketMessageReader {
    /// Consume one established WebSocket into a connection-bound message reader.
    #[must_use]
    pub fn new(established: WebDriverBiDiWebSocketEstablished) -> Self {
        let connection_generation = established.transport_evidence().connection_generation();
        Self {
            established,
            assembler: WebDriverBiDiWebSocketMessageAssembler::new(),
            connection_generation,
        }
    }

    /// Read and admit exactly one frame while preserving connection-bound fragmentation state.
    ///
    /// `Pending` and `Control` outcomes return the same reader so the next frame cannot silently
    /// switch sockets. A completed text message carries the private generation of this exact reader;
    /// because a completed text leaves no partial fragments, its established transport is returned
    /// directly for the next protocol stage rather than exposing a way to discard partial state.
    pub fn read_next(
        self,
        frame_timeout: Duration,
    ) -> Result<WebDriverBiDiConnectionMessageRead, WebDriverBiDiConnectionMessageReadError> {
        let Self {
            established,
            mut assembler,
            connection_generation,
        } = self;
        let (established, frame) = established
            .read_frame(frame_timeout)
            .map_err(|source| WebDriverBiDiConnectionMessageReadError::Frame { source })?;
        let assembly = assembler
            .push_frame(frame)
            .map_err(|source| WebDriverBiDiConnectionMessageReadError::Message { source })?;
        Ok(match assembly {
            WebDriverBiDiWebSocketMessageAssembly::Pending => {
                WebDriverBiDiConnectionMessageRead::Pending(Self {
                    established,
                    assembler,
                    connection_generation,
                })
            }
            WebDriverBiDiWebSocketMessageAssembly::Text(message) => {
                WebDriverBiDiConnectionMessageRead::Text {
                    established,
                    message: WebDriverBiDiReceivedTextMessage {
                        message,
                        connection_generation,
                    },
                }
            }
            WebDriverBiDiWebSocketMessageAssembly::Control(message) => {
                WebDriverBiDiConnectionMessageRead::Control {
                    reader: Self {
                        established,
                        assembler,
                        connection_generation,
                    },
                    message,
                }
            }
        })
    }
}

/// Outcome from one connection-bound WebDriver BiDi message-reader step.
pub enum WebDriverBiDiConnectionMessageRead {
    /// A fragmented text message remains incomplete; continue with this same reader.
    Pending(WebDriverBiDiWebSocketMessageReader),
    /// One complete text message was assembled entirely on this verified connection.
    Text {
        /// Established transport returned after the complete message left no partial fragments.
        established: WebDriverBiDiWebSocketEstablished,
        /// Complete text message carrying non-forgeable connection provenance.
        message: WebDriverBiDiReceivedTextMessage,
    },
    /// An interleaved RFC 6455 control message was observed without discarding partial text state.
    Control {
        /// Reader retaining the same established connection and partial-message state.
        reader: WebDriverBiDiWebSocketMessageReader,
        /// Validated control message observed on this exact connection.
        message: WebDriverBiDiWebSocketControlMessage,
    },
}

impl fmt::Debug for WebDriverBiDiConnectionMessageRead {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending(_) => formatter.write_str("Pending(connection-bound reader)"),
            Self::Text { message, .. } => formatter.debug_tuple("Text").field(message).finish(),
            Self::Control { message, .. } => {
                formatter.debug_tuple("Control").field(message).finish()
            }
        }
    }
}

/// Fail-closed frame or message error while reading one connection-bound BiDi message step.
#[derive(Debug)]
pub enum WebDriverBiDiConnectionMessageReadError {
    /// The exact established connection failed while reading or validating one RFC 6455 frame.
    Frame {
        /// Underlying bounded frame failure.
        source: WebDriverBiDiWebSocketFrameError,
    },
    /// The connection-local message assembler rejected the frame sequence.
    Message {
        /// Underlying bounded message-assembly failure.
        source: WebDriverBiDiWebSocketMessageError,
    },
}

impl fmt::Display for WebDriverBiDiConnectionMessageReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Frame { .. } => {
                formatter.write_str("connection-bound WebDriver BiDi WebSocket frame read failed")
            }
            Self::Message { .. } => formatter
                .write_str("connection-bound WebDriver BiDi WebSocket message assembly failed"),
        }
    }
}

impl Error for WebDriverBiDiConnectionMessageReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frame { source } => Some(source),
            Self::Message { source } => Some(source),
        }
    }
}
