//! Direct-only policy-bound TCP connection authority for OriginWeave.
//!
//! The crate consumes validated connection plans, opens exact socket addresses
//! without hostname resolution or proxy inheritance, verifies operating-system
//! peers before exposing transport I/O, and emits credential-free evidence.
//! It also bridges a session-correlated WebDriver BiDi loopback target from
//! `originweave-core` into one bounded exact TCP connection, binds an RFC 6455
//! opening request to that verified plain stream, and can write that exact request
//! under one bounded deadline, validate its bounded RFC 6455 opening response,
//! carry one bounded frame at a time, and bind one exact `locateNodes` command to
//! its bounded correlated response without granting browser, WebSocket, TLS,
//! policy, or Agent authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connection;
mod webdriver_bidi_connection;
mod webdriver_bidi_locate_nodes_exchange;
#[cfg(test)]
mod webdriver_bidi_locate_nodes_exchange_transport_failure_tests;
#[cfg(test)]
mod webdriver_bidi_locate_nodes_exchange_unit_coverage_tests;
mod webdriver_bidi_websocket_control;
#[cfg(test)]
#[allow(clippy::expect_used)]
mod webdriver_bidi_websocket_coverage_tests;
#[cfg(test)]
#[allow(clippy::expect_used)]
mod webdriver_bidi_websocket_debug_tests;
#[path = "webdriver_bidi_websocket_validated.rs"]
mod webdriver_bidi_websocket_handshake;
#[path = "webdriver_bidi_websocket_raw_redacted.rs"]
mod webdriver_bidi_websocket_handshake_raw;
mod webdriver_bidi_websocket_mask_key;

pub use connection::{
    ConnectionPlan, DirectTcpConnection, MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS,
    NetworkError, SocketConnectionEvidence,
};
pub use webdriver_bidi_connection::{
    WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError,
    WebDriverBiDiTcpConnectionEvidence, WebDriverBiDiTcpConnectionPlan,
};
pub use webdriver_bidi_locate_nodes_exchange::{
    MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE,
    MAX_WEBDRIVER_BIDI_RESPONSE_FRAGMENTS_PER_EXCHANGE, WebDriverBiDiLocateNodesExchangeError,
};
pub use webdriver_bidi_websocket_handshake::{
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketOpeningRequestSent,
};
pub use webdriver_bidi_websocket_handshake_raw::{
    MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE, MAX_WEBSOCKET_FRAME_TIMEOUT,
    MAX_WEBSOCKET_OPENING_RESPONSE_SIZE, MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT,
    MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketFrame, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketHandshakeError, WebDriverBiDiWebSocketHandshakeResponseError,
    WebDriverBiDiWebSocketOpeningWriteError,
};
pub use webdriver_bidi_websocket_mask_key::WebDriverBiDiWebSocketMaskKey;

/// Required recovery posture after a failed WebDriver BiDi WebSocket opening-request write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition {
    /// No complete opening request is known to have been submitted.
    ///
    /// This is not automatic retry permission. The caller must obtain fresh authority, route,
    /// connection, and deadline validation before starting another opening request.
    RevalidateBeforeNewAttempt,
    /// The peer may already have received the complete opening request, or byte accounting is
    /// inconsistent with the exact serialized request length.
    ///
    /// Blind redispatch is forbidden until the caller reconciles the potentially completed
    /// external side effect.
    ReconciliationRequired,
}

impl WebDriverBiDiWebSocketOpeningWriteError {
    /// Classify the fail-closed recovery posture for this failed opening-request write.
    ///
    /// `request_byte_count` must be the exact serialized length of the request whose write produced
    /// this error. A zero request length, complete-or-greater byte count, or timeout-cleanup failure
    /// is treated as ambiguous external completion and therefore requires reconciliation.
    #[must_use]
    pub fn recovery_disposition(
        &self,
        request_byte_count: usize,
    ) -> WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition {
        use WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::{
            ReconciliationRequired, RevalidateBeforeNewAttempt,
        };

        if request_byte_count == 0 {
            return ReconciliationRequired;
        }

        match self {
            Self::InvalidWriteTimeout { .. } => RevalidateBeforeNewAttempt,
            Self::WriteTimeoutCleanupFailed { .. } => ReconciliationRequired,
            Self::WriteDeadlineExceeded { bytes_written }
            | Self::WriteTimeoutConfigurationFailed { bytes_written, .. }
            | Self::WriteTimedOut { bytes_written, .. }
            | Self::WriteZero { bytes_written }
            | Self::WriteFailed { bytes_written, .. } => {
                if *bytes_written >= request_byte_count {
                    ReconciliationRequired
                } else {
                    RevalidateBeforeNewAttempt
                }
            }
        }
    }
}

#[cfg(test)]
mod opening_write_recovery_tests {
    use std::{io, time::Duration};

    use super::{
        WebDriverBiDiWebSocketOpeningWriteError,
        WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::{
            ReconciliationRequired, RevalidateBeforeNewAttempt,
        },
    };

    #[test]
    fn ambiguous_or_complete_opening_writes_require_reconciliation() {
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded { bytes_written: 16 }
                .recovery_disposition(16),
            ReconciliationRequired
        );
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::WriteFailed {
                bytes_written: 17,
                source: io::Error::other("write completion accounting exceeded request length"),
            }
            .recovery_disposition(16),
            ReconciliationRequired
        );
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutCleanupFailed {
                bytes_written: 16,
                source: io::Error::other("write timeout cleanup failed after request completion"),
            }
            .recovery_disposition(16),
            ReconciliationRequired
        );
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::InvalidWriteTimeout {
                write_timeout: Duration::ZERO,
                maximum_timeout: Duration::from_secs(5),
            }
            .recovery_disposition(0),
            ReconciliationRequired
        );
    }

    #[test]
    fn incomplete_opening_writes_require_fresh_revalidation_before_another_attempt() {
        let request_byte_count = 16;

        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::InvalidWriteTimeout {
                write_timeout: Duration::ZERO,
                maximum_timeout: Duration::from_secs(5),
            }
            .recovery_disposition(request_byte_count),
            RevalidateBeforeNewAttempt
        );
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded { bytes_written: 3 }
                .recovery_disposition(request_byte_count),
            RevalidateBeforeNewAttempt
        );
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutConfigurationFailed {
                bytes_written: 3,
                source: io::Error::other("timeout configuration failed"),
            }
            .recovery_disposition(request_byte_count),
            RevalidateBeforeNewAttempt
        );
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::WriteTimedOut {
                bytes_written: 3,
                source: io::Error::from(io::ErrorKind::TimedOut),
            }
            .recovery_disposition(request_byte_count),
            RevalidateBeforeNewAttempt
        );
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::WriteZero { bytes_written: 3 }
                .recovery_disposition(request_byte_count),
            RevalidateBeforeNewAttempt
        );
        assert_eq!(
            WebDriverBiDiWebSocketOpeningWriteError::WriteFailed {
                bytes_written: 3,
                source: io::Error::other("write failed before request completion"),
            }
            .recovery_disposition(request_byte_count),
            RevalidateBeforeNewAttempt
        );
    }
}
