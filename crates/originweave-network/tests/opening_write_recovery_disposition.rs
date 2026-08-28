use std::io;

use originweave_network::{
    WebDriverBiDiWebSocketOpeningWriteError,
    WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition,
};

#[test]
fn complete_or_cleanup_failed_opening_write_requires_reconciliation_before_retry() {
    let completed_after_deadline =
        WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded { bytes_written: 128 };
    assert_eq!(
        completed_after_deadline.recovery_disposition(128),
        WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::ReconciliationRequired
    );

    let cleanup_failed = WebDriverBiDiWebSocketOpeningWriteError::WriteTimeoutCleanupFailed {
        bytes_written: 128,
        source: io::Error::from(io::ErrorKind::InvalidInput),
    };
    assert_eq!(
        cleanup_failed.recovery_disposition(128),
        WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::ReconciliationRequired
    );
}

#[test]
fn partial_or_inconsistent_opening_write_failure_stays_fail_closed() {
    let partial_deadline =
        WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded { bytes_written: 64 };
    assert_eq!(
        partial_deadline.recovery_disposition(128),
        WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::RevalidateBeforeNewAttempt
    );

    let partial_timeout = WebDriverBiDiWebSocketOpeningWriteError::WriteTimedOut {
        bytes_written: 64,
        source: io::Error::from(io::ErrorKind::TimedOut),
    };
    assert_eq!(
        partial_timeout.recovery_disposition(128),
        WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::RevalidateBeforeNewAttempt
    );

    let impossible_count =
        WebDriverBiDiWebSocketOpeningWriteError::WriteDeadlineExceeded { bytes_written: 129 };
    assert_eq!(
        impossible_count.recovery_disposition(128),
        WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::ReconciliationRequired
    );
    assert_eq!(
        partial_deadline.recovery_disposition(0),
        WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition::ReconciliationRequired
    );
}
