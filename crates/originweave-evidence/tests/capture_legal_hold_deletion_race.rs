use std::error::Error;

use originweave_evidence::{
    CaptureDeletionReceipt, CaptureLifecycle, CaptureLifecycleError, CaptureLifecycleState,
};

const MANIFEST_DIGEST: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const FIRST_DELETION_REQUEST_DIGEST: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const SECOND_DELETION_REQUEST_DIGEST: &str =
    "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const FIRST_DELETION_EVIDENCE_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SECOND_DELETION_EVIDENCE_DIGEST: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[test]
fn legal_hold_preempts_pending_deletion_and_invalidates_the_old_request()
-> Result<(), Box<dyn Error>> {
    let mut lifecycle = CaptureLifecycle::new(MANIFEST_DIGEST, 100)?;
    lifecycle.complete(110)?;
    lifecycle.verify(120)?;
    lifecycle.retain_until(200, 130)?;
    lifecycle.request_deletion(FIRST_DELETION_REQUEST_DIGEST, 200)?;

    let stale_receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        FIRST_DELETION_REQUEST_DIGEST,
        FIRST_DELETION_EVIDENCE_DIGEST,
    )?;

    lifecycle.place_legal_hold(201)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::LegalHold);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 201);
    assert_eq!(lifecycle.deletion_request_digest(), None);
    assert!(lifecycle.deletion_receipt().is_none());
    assert_eq!(
        lifecycle.confirm_deleted(&stale_receipt, 202),
        Err(CaptureLifecycleError::InvalidTransition)
    );
    assert_eq!(lifecycle.state(), CaptureLifecycleState::LegalHold);
    assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 201);

    lifecycle.release_legal_hold_to_retained(300, 203)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Retained);
    assert_eq!(
        lifecycle.request_deletion(SECOND_DELETION_REQUEST_DIGEST, 299),
        Err(CaptureLifecycleError::RetentionNotExpired)
    );
    lifecycle.request_deletion(SECOND_DELETION_REQUEST_DIGEST, 300)?;

    let replacement_receipt = CaptureDeletionReceipt::new(
        MANIFEST_DIGEST,
        SECOND_DELETION_REQUEST_DIGEST,
        SECOND_DELETION_EVIDENCE_DIGEST,
    )?;
    lifecycle.confirm_deleted(&replacement_receipt, 301)?;
    assert_eq!(lifecycle.state(), CaptureLifecycleState::Deleted);
    assert_eq!(lifecycle.deletion_receipt(), Some(&replacement_receipt));
    Ok(())
}
