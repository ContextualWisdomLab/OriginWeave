use crate::webdriver_bidi_websocket_handshake::WebDriverBiDiWebSocketOpeningWriteError;

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
mod tests {
    use std::{io, time::Duration};

    use super::WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::{
        ReconciliationRequired, RevalidateBeforeNewAttempt,
    };
    use crate::WebDriverBiDiWebSocketOpeningWriteError;

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
