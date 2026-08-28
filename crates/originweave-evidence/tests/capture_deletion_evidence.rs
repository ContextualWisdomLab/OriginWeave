use std::error::Error;

use originweave_evidence::{
    CaptureDeletionReceipt, CaptureLifecycle, CaptureLifecycleError, CaptureLifecycleState,
};

const MANIFEST_DIGEST: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const OTHER_MANIFEST_DIGEST: &str =
    "sha256:1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const CURRENT_DELETION_REQUEST_DIGEST: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const STALE_DELETION_REQUEST_DIGEST: &str =
    "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const DELETION_EVIDENCE_DIGEST: &str =
    "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

fn deletion_requested_lifecycle() -> Result<CaptureLifecycle, CaptureLifecycleError> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    lifecycle.complete(110)?;
    lifecycle.verify(120)?;
    lifecycle.retain_until(200, 130)?;
    lifecycle.request_deletion(200)?;
    Ok(lifecycle)
}

#[test]
fn deletion_confirmation_requires_manifest_bound_evidence_receipt() -> Result<(), Box<dyn Error>> {
    let mut lifecycle = deletion_requested_lifecycle()?;
    let foreign_receipt =
        CaptureDeletionReceipt::new(OTHER_MANIFEST_DIGEST, DELETION_EVIDENCE_DIGEST)?;

    assert_eq!(
        lifecycle.confirm_deleted(&foreign_receipt, 201),
        Err(CaptureLifecycleError::DeletionReceiptMismatch)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::DeletionRequested);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 200);
    assert!(lifecycle.deletion_receipt().is_none());

    let receipt = CaptureDeletionReceipt::new(MANIFEST_DIGEST, DELETION_EVIDENCE_DIGEST)?;
    lifecycle.confirm_deleted(&receipt, 201)?;

    assert_eq!(lifecycle.state(), CaptureLifecycleState::Deleted);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 201);
    assert_eq!(lifecycle.deletion_receipt(), Some(&receipt));
    assert_eq!(receipt.manifest_digest(), MANIFEST_DIGEST);
    assert_eq!(receipt.evidence_digest(), DELETION_EVIDENCE_DIGEST);
    Ok(())
}

#[test]
fn deletion_receipt_rejects_unbound_or_noncanonical_evidence_identity() {
    assert_eq!(
        CaptureDeletionReceipt::new(MANIFEST_DIGEST, "sha256:not-a-digest"),
        Err(CaptureLifecycleError::InvalidDeletionEvidenceDigest)
    );
    assert_eq!(
        CaptureDeletionReceipt::new("sha256:not-a-digest", DELETION_EVIDENCE_DIGEST),
        Err(CaptureLifecycleError::InvalidManifestDigest)
    );
}

#[test]
fn deletion_receipt_cannot_be_replayed_across_request_identity() -> Result<(), Box<dyn Error>> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    lifecycle.complete(110)?;
    lifecycle.verify(120)?;
    lifecycle.retain_until(200, 130)?;
    lifecycle.request_deletion(CURRENT_DELETION_REQUEST_DIGEST, 200)?;

    let stale_receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        STALE_DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    assert_eq!(
        lifecycle.confirm_deleted(&stale_receipt, 201),
        Err(CaptureLifecycleError::DeletionReceiptRequestMismatch)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::DeletionRequested);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 200);
    assert_eq!(
        lifecycle.deletion_request_digest(),
        Some(CURRENT_DELETION_REQUEST_DIGEST)
    );
    assert!(lifecycle.deletion_receipt().is_none());

    let current_receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        CURRENT_DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;
    lifecycle.confirm_deleted(&current_receipt, 201)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Deleted);
    assert_eq!(
        current_receipt.deletion_request_digest(),
        CURRENT_DELETION_REQUEST_DIGEST
    );
    assert_eq!(lifecycle.deletion_receipt(), Some(&current_receipt));
    Ok(())
}
