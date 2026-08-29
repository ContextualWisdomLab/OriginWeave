use std::error::Error;

use originweave_evidence::{
    CaptureDeletionReceipt, CaptureLifecycle, CaptureLifecycleRestoreError, CaptureLifecycleState,
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
fn restore_accepts_valid_persisted_states() -> Result<(), Box<dyn Error>> {
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
    assert_eq!(
        held_without_retention.state(),
        CaptureLifecycleState::LegalHold
    );

    let held_after_retention = CaptureLifecycle::restore(
        MANIFEST_DIGEST,
        CaptureLifecycleState::LegalHold,
        250,
        Some(200),
        None,
        None,
    )?;
    assert_eq!(
        held_after_retention.state(),
        CaptureLifecycleState::LegalHold
    );
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
fn restored_states_can_continue_safely() -> Result<(), Box<dyn Error>> {
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
fn restore_rejects_corrupt_persisted_states() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        CaptureLifecycle::restore(
            "sha256:not-a-digest",
            CaptureLifecycleState::CaptureStarted,
            100,
            None,
            None,
            None,
        ),
        Err(CaptureLifecycleRestoreError::InvalidManifestDigest)
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
        Err(CaptureLifecycleRestoreError::InvalidPersistedState)
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
        Err(CaptureLifecycleRestoreError::InvalidPersistedState)
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
        Err(CaptureLifecycleRestoreError::InvalidPersistedState)
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
        Err(CaptureLifecycleRestoreError::InvalidPersistedState)
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
        Err(CaptureLifecycleRestoreError::InvalidDeletionRequestDigest)
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
        Err(CaptureLifecycleRestoreError::InvalidPersistedState)
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
        Err(CaptureLifecycleRestoreError::InvalidPersistedState)
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
        Err(CaptureLifecycleRestoreError::InvalidPersistedState)
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
        Err(CaptureLifecycleRestoreError::DeletionReceiptMismatch)
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
        Err(CaptureLifecycleRestoreError::DeletionReceiptRequestMismatch)
    );

    assert_eq!(
        CaptureLifecycleRestoreError::InvalidPersistedState.to_string(),
        "persisted capture lifecycle state is internally inconsistent"
    );
    Ok(())
}
