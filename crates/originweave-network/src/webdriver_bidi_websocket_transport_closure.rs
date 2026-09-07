use std::{error::Error, fmt, time::Duration};

use crate::{
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey, webdriver_bidi_connection::WebDriverBiDiConnectionGeneration,
};

/// Bounded transport-closure condition observed on one consumed WebDriver BiDi WebSocket.
///
/// Every successful variant proves clean TCP EOF on the exact established connection. A preceding
/// RFC 6455 Close exchange is retained separately from EOF-only cessation so a consumer cannot
/// confuse entering CLOSING with the transport actually becoming closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiWebSocketTransportClosureKind {
    /// The peer ended the TCP byte stream cleanly before any new WebSocket frame byte was read.
    PeerEof,
    /// A validated peer Close was answered with a masked client Close before clean TCP EOF.
    PeerCloseThenEof,
}

/// Credential-free observation that one established WebDriver BiDi transport actually closed.
///
/// Construction consumes the established WebSocket, so this value cannot be used to regain the
/// underlying connection. It retains the private process-local generation of that exact connection
/// so a later teardown consumer can reject closure observed on another socket even when session and
/// command identifiers are reused. Peer Close enters CLOSING only: OriginWeave answers it with a
/// caller-keyed masked Close and emits this final observation only after subsequent clean TCP EOF.
/// Pre-Close Ping is answered with an equally caller-keyed masked Pong carrying identical payload.
/// No masking entropy, browser authority, process state, profile cleanup, policy, secret, reconnect,
/// retry, or Agent authority is invented by this transport adapter.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketTransportClosureObservation {
    kind: WebDriverBiDiWebSocketTransportClosureKind,
    peer_close_status_code: Option<u16>,
    connection_generation: WebDriverBiDiConnectionGeneration,
}

impl WebDriverBiDiWebSocketTransportClosureObservation {
    /// Consume one established connection and prove bounded transport closure.
    ///
    /// The caller supplies independent fresh masking keys for the only pre-Close Ping response and
    /// the required Close response. At most one pre-Close Ping or unsolicited Pong is admitted;
    /// additional control traffic, application data, timeout, partial EOF, malformed frames, write
    /// failures, or any post-Close frame fail closed. Peer Close alone is never success: after the
    /// masked response this boundary requires zero-byte TCP EOF within the same bounded frame
    /// deadline before producing closure evidence.
    pub fn observe(
        established: WebDriverBiDiWebSocketEstablished,
        pong_masking_key: WebDriverBiDiWebSocketMaskKey,
        close_masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<Self, WebDriverBiDiWebSocketTransportClosureError> {
        let connection_generation = established.transport_evidence().connection_generation();
        Self::observe_pre_close(
            established,
            pong_masking_key,
            close_masking_key,
            frame_timeout,
            true,
            connection_generation,
        )
    }

    fn observe_pre_close(
        established: WebDriverBiDiWebSocketEstablished,
        pong_masking_key: WebDriverBiDiWebSocketMaskKey,
        close_masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
        allow_pre_close_control: bool,
        connection_generation: WebDriverBiDiConnectionGeneration,
    ) -> Result<Self, WebDriverBiDiWebSocketTransportClosureError> {
        match established.read_frame(frame_timeout) {
            Ok((established, frame)) if frame.opcode() == 0xa && allow_pre_close_control => {
                Self::observe_pre_close(
                    established,
                    pong_masking_key,
                    close_masking_key,
                    frame_timeout,
                    false,
                    connection_generation,
                )
            }
            Ok((established, frame)) if frame.opcode() == 0x9 && allow_pre_close_control => {
                let established = established
                    .write_pong_frame(frame.payload(), pong_masking_key, frame_timeout)
                    .map_err(
                        |source| WebDriverBiDiWebSocketTransportClosureError::Frame { source },
                    )?;
                Self::observe_pre_close(
                    established,
                    pong_masking_key,
                    close_masking_key,
                    frame_timeout,
                    false,
                    connection_generation,
                )
            }
            Ok((established, frame)) if frame.opcode() == 0x8 => {
                let peer_close_status_code = frame
                    .payload()
                    .get(..2)
                    .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]));
                let established = established
                    .write_close_frame(peer_close_status_code, close_masking_key, frame_timeout)
                    .map_err(
                        |source| WebDriverBiDiWebSocketTransportClosureError::Frame { source },
                    )?;
                Self::observe_eof_after_close(
                    established,
                    frame_timeout,
                    peer_close_status_code,
                    connection_generation,
                )
            }
            Ok((_established, frame)) => Err(
                WebDriverBiDiWebSocketTransportClosureError::UnexpectedFrame {
                    opcode: frame.opcode(),
                },
            ),
            Err(WebDriverBiDiWebSocketFrameError::FrameEnded { bytes_read: 0 }) => Ok(Self {
                kind: WebDriverBiDiWebSocketTransportClosureKind::PeerEof,
                peer_close_status_code: None,
                connection_generation,
            }),
            Err(source) => Err(WebDriverBiDiWebSocketTransportClosureError::Frame { source }),
        }
    }

    fn observe_eof_after_close(
        established: WebDriverBiDiWebSocketEstablished,
        frame_timeout: Duration,
        peer_close_status_code: Option<u16>,
        connection_generation: WebDriverBiDiConnectionGeneration,
    ) -> Result<Self, WebDriverBiDiWebSocketTransportClosureError> {
        match established.read_frame(frame_timeout) {
            Err(WebDriverBiDiWebSocketFrameError::FrameEnded { bytes_read: 0 }) => Ok(Self {
                kind: WebDriverBiDiWebSocketTransportClosureKind::PeerCloseThenEof,
                peer_close_status_code,
                connection_generation,
            }),
            Ok((_established, frame)) => Err(
                WebDriverBiDiWebSocketTransportClosureError::UnexpectedFrame {
                    opcode: frame.opcode(),
                },
            ),
            Err(source) => Err(WebDriverBiDiWebSocketTransportClosureError::Frame { source }),
        }
    }

    /// Return the exact transport-closure condition that produced this observation.
    #[must_use]
    pub const fn kind(&self) -> WebDriverBiDiWebSocketTransportClosureKind {
        self.kind
    }

    /// Return the validated peer Close status when the completed closing exchange carried one.
    #[must_use]
    pub const fn peer_close_status_code(&self) -> Option<u16> {
        self.peer_close_status_code
    }

    pub(crate) const fn connection_generation(&self) -> WebDriverBiDiConnectionGeneration {
        self.connection_generation
    }
}

/// Fail-closed errors while converting one established BiDi transport into closure evidence.
#[derive(Debug)]
pub enum WebDriverBiDiWebSocketTransportClosureError {
    /// The peer sent a valid WebSocket frame outside the bounded closing state machine.
    UnexpectedFrame {
        /// Exact validated RFC 6455 opcode observed instead of admissible bounded closing traffic.
        opcode: u8,
    },
    /// The existing bounded WebSocket frame reader or writer failed before closure was proven.
    Frame {
        /// Original typed frame failure retained as the causal source.
        source: WebDriverBiDiWebSocketFrameError,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketTransportClosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedFrame { .. } => formatter
                .write_str("WebDriver BiDi peer sent non-closure traffic instead of closing"),
            Self::Frame { .. } => {
                formatter.write_str("WebDriver BiDi transport closure could not be observed safely")
            }
        }
    }
}

impl Error for WebDriverBiDiWebSocketTransportClosureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedFrame { .. } => None,
            Self::Frame { source } => Some(source),
        }
    }
}
