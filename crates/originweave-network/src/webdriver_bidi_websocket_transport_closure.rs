use std::{error::Error, fmt, time::Duration};

use crate::{WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError};

/// Bounded transport-closure condition observed on one consumed WebDriver BiDi WebSocket.
///
/// This classification proves only what the already session-correlated transport itself exposed.
/// A peer Close frame does not prove browser-process exit or profile cleanup, while peer EOF does
/// not imply that the RFC 6455 closing handshake completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiWebSocketTransportClosureKind {
    /// The peer sent one RFC 6455 Close frame that passed the existing strict frame validator.
    PeerCloseFrame,
    /// The peer ended the TCP byte stream cleanly before any new WebSocket frame byte was read.
    PeerEof,
}

/// Credential-free observation that one established WebDriver BiDi transport ceased carrying data.
///
/// Construction consumes the established WebSocket, so this value cannot be used to regain the
/// underlying connection. It records only a validated peer Close status when one was actually
/// present, or clean TCP EOF before a new frame began. It grants no browser, process, profile,
/// policy, secret, retry, reconnect, or Agent authority and does not perform a reciprocal Close
/// handshake.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketTransportClosureObservation {
    kind: WebDriverBiDiWebSocketTransportClosureKind,
    peer_close_status_code: Option<u16>,
}

impl WebDriverBiDiWebSocketTransportClosureObservation {
    /// Consume one established connection and observe one bounded transport-closure condition.
    ///
    /// A validated peer Close frame and zero-byte clean EOF are the only success cases. Any data
    /// frame, partial-frame EOF, timeout, malformed Close, I/O failure, or integrity failure remains
    /// a typed error and is never normalized into successful teardown evidence.
    pub fn observe(
        established: WebDriverBiDiWebSocketEstablished,
        frame_timeout: Duration,
    ) -> Result<Self, WebDriverBiDiWebSocketTransportClosureError> {
        match established.read_frame(frame_timeout) {
            Ok((established, frame)) => {
                if frame.opcode() != 0x8 {
                    return Err(
                        WebDriverBiDiWebSocketTransportClosureError::UnexpectedFrame {
                            opcode: frame.opcode(),
                        },
                    );
                }
                let peer_close_status_code = frame
                    .payload()
                    .get(..2)
                    .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]));
                drop(established);
                Ok(Self {
                    kind: WebDriverBiDiWebSocketTransportClosureKind::PeerCloseFrame,
                    peer_close_status_code,
                })
            }
            Err(WebDriverBiDiWebSocketFrameError::FrameEnded { bytes_read: 0 }) => Ok(Self {
                kind: WebDriverBiDiWebSocketTransportClosureKind::PeerEof,
                peer_close_status_code: None,
            }),
            Err(source) => Err(WebDriverBiDiWebSocketTransportClosureError::Frame { source }),
        }
    }

    /// Return the exact transport-closure condition that produced this observation.
    #[must_use]
    pub const fn kind(&self) -> WebDriverBiDiWebSocketTransportClosureKind {
        self.kind
    }

    /// Return the validated peer Close status when a Close frame actually carried one.
    #[must_use]
    pub const fn peer_close_status_code(&self) -> Option<u16> {
        self.peer_close_status_code
    }
}

/// Fail-closed errors while converting one established BiDi transport into closure evidence.
#[derive(Debug)]
pub enum WebDriverBiDiWebSocketTransportClosureError {
    /// The peer sent a valid WebSocket frame that was not a Close frame.
    UnexpectedFrame {
        /// Exact validated RFC 6455 opcode observed instead of a Close frame.
        opcode: u8,
    },
    /// The existing bounded WebSocket frame reader failed before closure was proven.
    Frame {
        /// Original typed frame failure retained as the causal source.
        source: WebDriverBiDiWebSocketFrameError,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketTransportClosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedFrame { .. } => formatter
                .write_str("WebDriver BiDi peer sent application traffic instead of closing"),
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
