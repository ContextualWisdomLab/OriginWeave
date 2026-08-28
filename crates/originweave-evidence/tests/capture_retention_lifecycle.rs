use std::error::Error;

use originweave_evidence::{
    CaptureDeletionReceipt, CaptureLifecycle, CaptureLifecycleError, CaptureLifecycleState,
};

const MANIFEST_DIGEST: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const DELETION_REQUEST_DIGEST: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const DELETION_EVIDENCE_DIGEST: &str =
    "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

#[test]
fn retained_capture_reaches_deleted_only_after_retention_expiry() -> Result<(), Box<dyn Error>> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::CaptureStarted);
    assert_eq!(lifecycle.manifest_digest(), MANIFEST_DIGEST);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 100);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), None);
    assert_eq!(lifecycle.deletion_request_digest(), None);

    lifecycle.complete(110)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::CaptureCompleted);
    lifecycle.verify(120)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Verified);
    lifecycle.retain_until(200, 130)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Retained);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), Some(200));

    assert_eq!(
        lifecycle.request_deletion(DELETION_REQUEST_DIGEST, 199),
        Err(CaptureLifecycleError::RetentionNotExpired)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Retained);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 130);
    assert_eq!(lifecycle.deletion_request_digest(), None);

    lifecycle.request_deletion(DELETION_REQUEST_DIGEST, 200)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::DeletionRequested);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 200);
    assert_eq!(
        lifecycle.deletion_request_digest(),
        Some(DELETION_REQUEST_DIGEST)
    );
    let receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    lifecycle.confirm_deleted(&receipt, 201)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Deleted);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 201);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), Some(200));
    assert_eq!(lifecycle.deletion_receipt(), Some(&receipt));
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
        lifecycle.request_deletion(DELETION_REQUEST_DIGEST, 1_200),
        Err(CaptureLifecycleError::LegalHoldActive)
    );
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 1_040);
    assert_eq!(lifecycle.deletion_request_digest(), None);

    assert_eq!(
        lifecycle.release_legal_hold_to_retained(1_200, 1_200),
        Err(CaptureLifecycleError::InvalidRetentionDeadline)
    );
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 1_040);
    lifecycle.release_legal_hold_to_retained(1_300, 1_201)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Retained);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 1_201);
    assert_eq!(lifecycle.retention_deadline_epoch_seconds(), Some(1_300));
    assert_eq!(
        lifecycle.request_deletion(DELETION_REQUEST_DIGEST, 1_299),
        Err(CaptureLifecycleError::RetentionNotExpired)
    );
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 1_201);
    lifecycle.request_deletion(DELETION_REQUEST_DIGEST, 1_300)?;
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 1_300);
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
fn rejected_transition_cannot_poison_latest_accepted_trusted_time() -> Result<(), Box<dyn Error>> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;

    assert_eq!(
        lifecycle.place_legal_hold(10_000),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::CaptureStarted);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 100);

    lifecycle.complete(101)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::CaptureCompleted);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 101);
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
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 100);
    assert_eq!(
        lifecycle.complete(99),
        Err(CaptureLifecycleError::TrustedTimeRollback)
    );
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 100);

    lifecycle.complete(102)?;
    lifecycle.verify(103)?;
    assert_eq!(
        lifecycle.retain_until(103, 103),
        Err(CaptureLifecycleError::InvalidRetentionDeadline)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Verified);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 103);
    lifecycle.retain_until(105, 104)?;
    assert_eq!(
        lifecycle.complete(105),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 104);
    lifecycle.place_legal_hold(106)?;
    assert_eq!(
        lifecycle.retain_until(200, 107),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(
        lifecycle.place_legal_hold(108),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    let receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    assert_eq!(
        lifecycle.confirm_deleted(&receipt, 108),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 106);
    assert!(lifecycle.deletion_receipt().is_none());
    Ok(())
}

#[test]
fn every_transition_rejects_trusted_time_rollback() -> Result<(), Box<dyn Error>> {
    let mut verifying = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    assert_eq!(
        verifying.verify(99),
        Err(CaptureLifecycleError::TrustedTimeRollback)
    );

    let mut retaining = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    retaining.complete(101)?;
    retaining.verify(102)?;
    assert_eq!(
        retaining.retain_until(200, 101),
        Err(CaptureLifecycleError::TrustedTimeRollback)
    );

    let mut holding = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    holding.complete(101)?;
    holding.verify(102)?;
    assert_eq!(
        holding.place_legal_hold(101),
        Err(CaptureLifecycleError::TrustedTimeRollback)
    );

    let mut releasing = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    releasing.complete(101)?;
    releasing.verify(102)?;
    releasing.place_legal_hold(103)?;
    assert_eq!(
        releasing.release_legal_hold_to_retained(200, 102),
        Err(CaptureLifecycleError::TrustedTimeRollback)
    );

    let mut requesting = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    requesting.complete(101)?;
    requesting.verify(102)?;
    requesting.retain_until(200, 103)?;
    assert_eq!(
        requesting.request_deletion(DELETION_REQUEST_DIGEST, 102),
        Err(CaptureLifecycleError::TrustedTimeRollback)
    );

    let mut confirming = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    confirming.complete(101)?;
    confirming.verify(102)?;
    confirming.retain_until(200, 103)?;
    confirming.request_deletion(DELETION_REQUEST_DIGEST, 200)?;
    let receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    assert_eq!(
        confirming.confirm_deleted(&receipt, 199),
        Err(CaptureLifecycleError::TrustedTimeRollback)
    );
    assert!(confirming.deletion_receipt().is_none());

    let mut started = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    assert_eq!(
        started.request_deletion(DELETION_REQUEST_DIGEST, 101),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(started.latest_trusted_time_epoch_seconds(), 100);

    let mut not_held = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    assert_eq!(
        not_held.release_legal_hold_to_retained(200, 101),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(not_held.latest_trusted_time_epoch_seconds(), 100);
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
            CaptureLifecycleError::InvalidDeletionRequestDigest,
            "capture deletion request digest must be lowercase sha256",
        ),
        (
            CaptureLifecycleError::InvalidDeletionEvidenceDigest,
            "capture deletion evidence digest must be lowercase sha256",
        ),
        (
            CaptureLifecycleError::DeletionReceiptMismatch,
            "capture deletion receipt does not match lifecycle manifest",
        ),
        (
            CaptureLifecycleError::DeletionReceiptRequestMismatch,
            "capture deletion receipt does not match lifecycle deletion request",
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
