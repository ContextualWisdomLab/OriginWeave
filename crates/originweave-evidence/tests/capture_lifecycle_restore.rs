use std::error::Error;

use originweave_evidence::{
    CaptureDeletionReceipt, CaptureLifecycle, CaptureLifecycleError, CaptureLifecycleState,
};

const MANIFEST_DIGEST: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const OTHER_MANIFEST_DIGEST: &str =
    "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const DELETION_REQUEST_DIGEST: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const OTHER_DELETION_REQUEST_DIGEST: &str =
    "sha256:3333333333333333333333333333333333333333333333333333333333333333";
const DELETION_EVIDENCE_DIGEST: &str =
    "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

#[test]
fn restored_lifecycle_states_preserve_only_valid_persisted_shapes() -> Result<(), Box<dyn Error>> {
    let started = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::CaptureStarted,
        100,
        None,
        None,
        None,
    )?;
    assert_eq!(started.state(), CaptureLifecycleState::CaptureStarted);

    let completed = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::CaptureCompleted,
        110,
        None,
        None,
        None,
    )?;
    assert_eq!(completed.state(), CaptureLifecycleState::CaptureCompleted);

    let verified = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::Verified,
        120,
        None,
        None,
        None,
    )?;
    assert_eq!(verified.state(), CaptureLifecycleState::Verified);

    let retained = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::Retained,
        130,
        Some(200),
        None,
        None,
    )?;
    assert_eq!(retained.state(), CaptureLifecycleState::Retained);
    assert_eq!(retained.retention_deadline_epoch_seconds(), Some(200));

    let held_without_retention = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::LegalHold,
        140,
        None,
        None,
        None,
    )?;
    assert_eq!(held_without_retention.state(), CaptureLifecycleState::LegalHold);

    let held_after_retention = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::LegalHold,
        250,
        Some(200),
        None,
        None,
    )?;
    assert_eq!(held_after_retention.state(), CaptureLifecycleState::LegalHold);
    assert_eq!(
        held_after_retention.retention_deadline_epoch_seconds(),
        Some(200)
    );

    let deletion_requested = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::DeletionRequested,
        200,
        Some(200),
        Some(DELETION_REQUEST_DIGEST),
        None,
    )?;
    assert_eq!(
        deletion_requested.deletion_request_digest(),
        Some(DELETION_REQUEST_DIGEST)
    );

    let receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    let deleted = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::Deleted,
        201,
        Some(200),
        Some(DELETION_REQUEST_DIGEST),
        Some(&receipt),
    )?;
    assert_eq!(deleted.state(), CaptureLifecycleState::Deleted);
    assert_eq!(deleted.deletion_receipt(), Some(&receipt));
    Ok(())
}

#[test]
fn restored_retained_and_pending_deletion_states_can_continue_safely() -> Result<(), Box<dyn Error>> {
    let mut retained = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::Retained,
        130,
        Some(200),
        None,
        None,
    )?;
    retained.request_deletion(DELETION_REQUEST_DIGEST, 200)?;
    let receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    retained.confirm_deleted(&receipt, 201)?;
    assert_eq!(retained.state(), CaptureLifecycleState::Deleted);

    let mut pending = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::DeletionRequested,
        200,
        Some(200),
        Some(DELETION_REQUEST_DIGEST),
        None,
    )?;
    pending.confirm_deleted(&receipt, 201)?;
    assert_eq!(pending.state(), CaptureLifecycleState::Deleted);
    Ok(())
}

#[test]
fn restore_rejects_corrupt_persisted_lifecycle_shapes() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        CaptureLifecycle::restore(
            "sha256:not-a-digest",
            CaptureLifecycleState::CaptureStarted,
            100,
            None,
            None,
            None,
        ),
        Err(CaptureLifecycleError::InvalidManifestDigest)
    );
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::Retained,
            130,
            None,
            None,
            None,
        ),
        Err(CaptureLifecycleError::InvalidRestoredState)
    );
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::Retained,
            200,
            Some(200),
            None,
            None,
        ),
        Err(CaptureLifecycleError::InvalidRestoredState)
    );
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::CaptureCompleted,
            110,
            Some(200),
            None,
            None,
        ),
        Err(CaptureLifecycleError::InvalidRestoredState)
    );
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::LegalHold,
            210,
            Some(200),
            Some(DELETION_REQUEST_DIGEST),
            None,
        ),
        Err(CaptureLifecycleError::InvalidRestoredState)
    );
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::DeletionRequested,
            200,
            Some(200),
            Some("sha256:not-a-digest"),
            None,
        ),
        Err(CaptureLifecycleError::InvalidDeletionRequestDigest)
    );
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::DeletionRequested,
            199,
            Some(200),
            Some(DELETION_REQUEST_DIGEST),
            None,
        ),
        Err(CaptureLifecycleError::InvalidRestoredState)
    );

    let receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::Deleted,
            201,
            Some(200),
            Some(DELETION_REQUEST_DIGEST),
            None,
        ),
        Err(CaptureLifecycleError::InvalidRestoredState)
    );
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::DeletionRequested,
            200,
            Some(200),
            Some(DELETION_REQUEST_DIGEST),
            Some(&receipt),
        ),
        Err(CaptureLifecycleError::InvalidRestoredState)
    );

    let wrong_manifest_receipt = CaptureDeletionReceipt::new(
        OTHER_MANIFEST_DIGEST,
        DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::Deleted,
            201,
            Some(200),
            Some(DELETION_REQUEST_DIGEST),
            Some(&wrong_manifest_receipt),
        ),
        Err(CaptureLifecycleError::DeletionReceiptMismatch)
    );

    let wrong_request_receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        OTHER_DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    assert_eq!(
        CaptureLifecycle::restore(
            MANIFEST_DIGEST,
            CaptureLifecycleState::Deleted,
            201,
            Some(200),
            Some(DELETION_REQUEST_DIGEST),
            Some(&wrong_request_receipt),
        ),
        Err(CaptureLifecycleError::DeletionReceiptRequestMismatch)
    );

    assert_eq!(
        CaptureLifecycleError::InvalidRestoredState.to_string(),
        "persisted capture lifecycle state is internally inconsistent"
    );
    Ok(())
}
