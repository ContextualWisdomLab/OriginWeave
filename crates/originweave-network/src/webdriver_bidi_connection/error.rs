use std::{fmt, io, net::SocketAddr, time::Duration};

use originweave_core::WebDriverBiDiSocketPeerVerificationError;

use crate::connection::{MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS};

/// Deterministic failures while establishing one bounded WebDriver BiDi TCP transport.
#[derive(Debug)]
pub enum WebDriverBiDiTcpConnectionError {
    /// The requested timeout was zero or exceeded [`MAX_CONNECT_TIMEOUT`].
    InvalidConnectTimeout {
        /// The rejected timeout.
        connect_timeout: Duration,
        /// The largest accepted per-attempt timeout.
        maximum_timeout: Duration,
    },
    /// The requested attempt count was outside `1..=MAX_CONNECTION_ATTEMPTS`.
    InvalidAttemptCount {
        /// The rejected attempt count.
        attempt_count: u8,
        /// The largest accepted attempt count.
        maximum_attempts: u8,
    },
    /// The final bounded connection attempt timed out.
    ConnectionTimedOut {
        /// Exact approved socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// Number of attempts completed before failure.
        attempt_count: u8,
        /// Per-attempt timeout used by the plan.
        connect_timeout: Duration,
        /// Final operating-system timeout error.
        source: io::Error,
    },
    /// The final bounded connection attempt failed without a timeout.
    ConnectionFailed {
        /// Exact approved socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// Number of attempts completed before failure.
        attempt_count: u8,
        /// Final operating-system connection error.
        source: io::Error,
    },
    /// The established stream did not reveal an operating-system peer address.
    PeerInspectionFailed {
        /// Exact approved socket address submitted to the operating system.
        socket_address: SocketAddr,
        /// One-based attempt that established the stream.
        attempt_number: u8,
        /// Operating-system peer-inspection error.
        source: io::Error,
    },
    /// The established stream reported a peer other than the exact approved BiDi target.
    PeerMismatch {
        /// One-based attempt that established the stream.
        attempt_number: u8,
        /// Typed core peer-verification failure preserving expected and actual socket addresses.
        source: WebDriverBiDiSocketPeerVerificationError,
    },
}

impl WebDriverBiDiTcpConnectionError {
    /// Return the number of transport attempts associated with this failure, when applicable.
    #[must_use]
    pub const fn attempt_count(&self) -> Option<u8> {
        match self {
            Self::ConnectionTimedOut { attempt_count, .. }
            | Self::ConnectionFailed { attempt_count, .. } => Some(*attempt_count),
            Self::PeerInspectionFailed { attempt_number, .. }
            | Self::PeerMismatch { attempt_number, .. } => Some(*attempt_number),
            Self::InvalidConnectTimeout { .. } | Self::InvalidAttemptCount { .. } => None,
        }
    }
}

impl fmt::Display for WebDriverBiDiTcpConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConnectTimeout {
                connect_timeout,
                maximum_timeout,
            } => write!(
                formatter,
                "WebDriver BiDi connect timeout {connect_timeout:?} is outside 1ns..={maximum_timeout:?}",
            ),
            Self::InvalidAttemptCount {
                attempt_count,
                maximum_attempts,
            } => write!(
                formatter,
                "WebDriver BiDi connection attempt count {attempt_count} is outside 1..={maximum_attempts}",
            ),
            Self::ConnectionTimedOut {
                socket_address,
                attempt_count,
                connect_timeout,
                ..
            } => write!(
                formatter,
                "WebDriver BiDi TCP connection to {socket_address} timed out after {attempt_count} attempts with per-attempt timeout {connect_timeout:?}",
            ),
            Self::ConnectionFailed {
                socket_address,
                attempt_count,
                ..
            } => write!(
                formatter,
                "WebDriver BiDi TCP connection to {socket_address} failed after {attempt_count} attempts",
            ),
            Self::PeerInspectionFailed {
                socket_address,
                attempt_number,
                ..
            } => write!(
                formatter,
                "WebDriver BiDi TCP peer inspection failed for {socket_address} on attempt {attempt_number}",
            ),
            Self::PeerMismatch { attempt_number, .. } => write!(
                formatter,
                "WebDriver BiDi TCP peer did not match the approved target on attempt {attempt_number}",
            ),
        }
    }
}

impl std::error::Error for WebDriverBiDiTcpConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ConnectionTimedOut { source, .. }
            | Self::ConnectionFailed { source, .. }
            | Self::PeerInspectionFailed { source, .. } => Some(source),
            Self::PeerMismatch { source, .. } => Some(source),
            Self::InvalidConnectTimeout { .. } | Self::InvalidAttemptCount { .. } => None,
        }
    }
}
