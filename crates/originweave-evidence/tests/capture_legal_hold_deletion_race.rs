use std::error::Error;

use originweave_evidence::{
    CaptureDeletionReceipt, CaptureLifecycle, CaptureLifecycleError, CaptureLifecycleState,
};

const MANIFEST_DIGEST: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const DELETION_REQUEST_DIGEST: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const DELETION_EVIDENCE_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn legal_hold_cannot_revoke_pending_deletion_without_cancellation_evidence()
-> Result<(), Box<dyn Error>> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    lifecycle.complete(110)?;
    lifecycle.verify(120)?;
    lifecycle.retain_until(200, 130)?;
    lifecycle.request_deletion(DELETION_REQUEST_DIGEST, 200)?;

    let receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        DELETION_REQUEST_DIGEST,
        DELETION_EVIDENCE_DIGEST,
    )?;

    assert_eq!(
        lifecycle.place_legal_hold(201),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::DeletionRequested);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 200);
    assert_eq!(
        lifecycle.deletion_request_digest(),
        Some(DELETION_REQUEST_DIGEST)
    );
    assert!(lifecycle.deletion_receipt().is_none());

    lifecycle.confirm_deleted(&receipt, 202)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Deleted);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 202);
    assert_eq!(lifecycle.deletion_receipt(), Some(&receipt));
    Ok(())
}
