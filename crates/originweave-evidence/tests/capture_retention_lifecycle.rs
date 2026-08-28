use std::error::Error;

use originweave_evidence::{CaptureLifecycle, CaptureLifecycleError, CaptureLifecycleState};

const MANIFEST_DIGEST: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn retained_capture_reaches_deleted_only_after_retention_expiry() -> Result<(), Box<dyn Error>> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::CaptureStarted);
    assert_eq!(lifecycle.manifest_digest(), MANIFEST_DIGEST);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 100);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), None);

    lifecycle.complete(110)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::CaptureCompleted);
    lifecycle.verify(120)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Verified);
    lifecycle.retain_until(200, 130)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Retained);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), Some(200));

    assert_eq!(
        lifecycle.request_deletion(199),
        Err(CaptureLifecycleError::RetentionNotExpired)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Retained);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 199);

    lifecycle.request_deletion(200)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::DeletionRequested);
    lifecycle.confirm_deleted(201)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Deleted);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), Some(200));
    Ok(())
}

#[test]
fn legal_hold_blocks_deletion_until_explicit_release() -> Result<(), Box<dyn Error>> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 1_000)?;
    lifecycle.complete(1_010)?;
    lifecycle.verify(1_020)?;
    lifecycle.retain_until(1_100, 1_030)?;
    lifecycle.place_legal_hold(1_040)?;

    assert_eq!(lifecycle.state(), CaptureLifecycleState::LegalHold);
    assert_eq!(
        lifecycle.request_deletion(1_200),
        Err(CaptureLifecycleError::LegalHoldActive)
    );
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 1_200);

    assert_eq!(
        lifecycle.release_legal_hold_to_retained(1_200, 1_200),
        Err(CaptureLifecycleError::InvalidRetentionDeadline)
    );
    lifecycle.release_legal_hold_to_retained(1_300, 1_201)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Retained);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), Some(1_300));
    assert_eq!(
        lifecycle.request_deletion(1_299),
        Err(CaptureLifecycleError::RetentionNotExpired)
    );
    lifecycle.request_deletion(1_300)?;
    Ok(())
}

#[test]
fn verified_capture_can_enter_hold_without_inventing_retention() -> Result<(), Box<dyn Error>> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 10)?;
    lifecycle.complete(11)?;
    lifecycle.verify(12)?;
    lifecycle.place_legal_hold(13)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::LegalHold);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), None);
    lifecycle.release_legal_hold_to_retained(30, 14)?;
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), Some(30));
    Ok(())
}

#[test]
fn lifecycle_fails_closed_on_bad_identity_order_and_time() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        CaptureLifecycle::new("sha256:not-a-digest", 10),
        Err(CaptureLifecycleError::InvalidManifestDigest)
    );

    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    assert_eq!(
        lifecycle.place_legal_hold(101),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(
        lifecycle.verify(101),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::CaptureStarted);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 101);
    assert_eq!(
        lifecycle.complete(100),
        Err(CaptureLifecycleError::TrustedTimeRollback)
    );
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 101);

    lifecycle.complete(102)?;
    lifecycle.verify(103)?;
    assert_eq!(
        lifecycle.retain_until(103, 103),
        Err(CaptureLifecycleError::InvalidRetentionDeadline)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Verified);
    lifecycle.retain_until(105, 104)?;
    assert_eq!(
        lifecycle.complete(105),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    lifecycle.place_legal_hold(106)?;
    assert_eq!(
        lifecycle.confirm_deleted(107),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    Ok(())
}

#[test]
fn lifecycle_errors_are_standard_credential_safe_errors() {
    let cases = [
        (
            CaptureLifecycleError::InvalidManifestDigest,
            "capture manifest digest must be lowercase sha256",
        ),
        (
            CaptureLifecycleError::InvalidTransition,
            "capture lifecycle transition is not allowed from the current state",
        ),
        (
            CaptureLifecycleError::TrustedTimeRollback,
            "trusted capture lifecycle time moved backwards",
        ),
        (
            CaptureLifecycleError::InvalidRetentionDeadline,
            "capture retention deadline must be later than the current trusted time",
        ),
        (
            CaptureLifecycleError::RetentionNotExpired,
            "capture retention deadline has not expired",
        ),
        (
            CaptureLifecycleError::LegalHoldActive,
            "capture is under legal hold",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(Error::source(&error).is_none());
    }
}
