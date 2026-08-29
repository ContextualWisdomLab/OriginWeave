use core::fmt;
use std::error::Error;

use crate::{
    CaptureDeletionReceipt, CaptureLifecycle, CaptureLifecycleError, CaptureLifecycleState,
    valid_sha256,
};

/// A fail-closed error returned while reconstructing persisted capture lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureLifecycleRestoreError {
    /// The persisted lifecycle fields cannot represent a state produced by the lifecycle API.
    InvalidPersistedState,
    /// An existing lifecycle identity or receipt invariant rejected the persisted input.
    Lifecycle(CaptureLifecycleError),
}

impl fmt::Display for CaptureLifecycleRestoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPersistedState => formatter
                .write_str("persisted capture lifecycle state is internally inconsistent"),
            Self::Lifecycle(error) => error.fmt(formatter),
        }
    }
}

impl Error for CaptureLifecycleRestoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPersistedState => None,
            Self::Lifecycle(error) => Some(error),
        }
    }
}

impl CaptureLifecycle {
    /// Reconstruct one previously persisted lifecycle snapshot without widening authority.
    ///
    /// Restoration accepts only field combinations that can be produced by the public
    /// lifecycle transition API, then deterministically replays those transitions so the
    /// reconstructed value is governed by the same retention, legal-hold, request, receipt,
    /// and trusted-time invariants as a newly created value. The caller remains responsible
    /// for authenticating the persistence record, trusted time, deletion request, deletion
    /// evidence, tenant context, and legal authority before supplying the snapshot here.
    /// This method does not read storage, authenticate an actor, delete bytes, grant access,
    /// or convert malformed persistence into a successful lifecycle state.
    pub fn restore(
        manifest_digest: &str,
        state: CaptureLifecycleState,
        latest_trusted_time_epoch_seconds: u64,
        retention_deadline_epoch_seconds: Option<u64>,
        deletion_request_digest: Option<&str>,
        deletion_receipt: Option<&CaptureDeletionReceipt>,
    ) -> Result<Self, CaptureLifecycleRestoreError> {
        if !valid_sha256(manifest_digest) {
            return Err(CaptureLifecycleRestoreError::Lifecycle(
                CaptureLifecycleError::InvalidManifestDigest,
            ));
        }

        match (
            state,
            retention_deadline_epoch_seconds,
            deletion_request_digest,
            deletion_receipt,
        ) {
            (CaptureLifecycleState::CaptureStarted, None, None, None) => {
                Self::new(manifest_digest, latest_trusted_time_epoch_seconds)
                    .map_err(map_lifecycle_error)
            }
            (CaptureLifecycleState::CaptureCompleted, None, None, None) => {
                let mut lifecycle =
                    Self::new(manifest_digest, latest_trusted_time_epoch_seconds)
                        .map_err(map_lifecycle_error)?;
                lifecycle
                    .complete(latest_trusted_time_epoch_seconds)
                    .map_err(map_lifecycle_error)?;
                Ok(lifecycle)
            }
            (CaptureLifecycleState::Verified, None, None, None) => {
                verified_at(manifest_digest, latest_trusted_time_epoch_seconds)
            }
            (CaptureLifecycleState::Retained, Some(deadline), None, None) => {
                let mut lifecycle =
                    verified_at(manifest_digest, latest_trusted_time_epoch_seconds)?;
                lifecycle
                    .retain_until(deadline, latest_trusted_time_epoch_seconds)
                    .map_err(map_lifecycle_error)?;
                Ok(lifecycle)
            }
            (CaptureLifecycleState::LegalHold, None, None, None) => {
                let mut lifecycle =
                    verified_at(manifest_digest, latest_trusted_time_epoch_seconds)?;
                lifecycle
                    .place_legal_hold(latest_trusted_time_epoch_seconds)
                    .map_err(map_lifecycle_error)?;
                Ok(lifecycle)
            }
            (CaptureLifecycleState::LegalHold, Some(deadline), None, None) => {
                let replay_time = retained_replay_time(deadline, latest_trusted_time_epoch_seconds)?;
                let mut lifecycle = verified_at(manifest_digest, replay_time)?;
                lifecycle
                    .retain_until(deadline, replay_time)
                    .map_err(map_lifecycle_error)?;
                lifecycle
                    .place_legal_hold(latest_trusted_time_epoch_seconds)
                    .map_err(map_lifecycle_error)?;
                Ok(lifecycle)
            }
            (CaptureLifecycleState::DeletionRequested, Some(deadline), Some(request), None) => {
                let replay_time = deletion_replay_time(deadline, latest_trusted_time_epoch_seconds)?;
                let mut lifecycle = verified_at(manifest_digest, replay_time)?;
                lifecycle
                    .retain_until(deadline, replay_time)
                    .map_err(map_lifecycle_error)?;
                lifecycle
                    .request_deletion(request, latest_trusted_time_epoch_seconds)
                    .map_err(map_lifecycle_error)?;
                Ok(lifecycle)
            }
            (
                CaptureLifecycleState::Deleted,
                Some(deadline),
                Some(request),
                Some(receipt),
            ) => {
                let replay_time = deletion_replay_time(deadline, latest_trusted_time_epoch_seconds)?;
                let mut lifecycle = verified_at(manifest_digest, replay_time)?;
                lifecycle
                    .retain_until(deadline, replay_time)
                    .map_err(map_lifecycle_error)?;
                lifecycle
                    .request_deletion(request, latest_trusted_time_epoch_seconds)
                    .map_err(map_lifecycle_error)?;
                lifecycle
                    .confirm_deleted(receipt, latest_trusted_time_epoch_seconds)
                    .map_err(map_lifecycle_error)?;
                Ok(lifecycle)
            }
            _ => Err(CaptureLifecycleRestoreError::InvalidPersistedState),
        }
    }
}

fn verified_at(
    manifest_digest: &str,
    trusted_time_epoch_seconds: u64,
) -> Result<CaptureLifecycle, CaptureLifecycleRestoreError> {
    let mut lifecycle = CaptureLifecycle::new(manifest_digest, trusted_time_epoch_seconds)
        .map_err(map_lifecycle_error)?;
    lifecycle
        .complete(trusted_time_epoch_seconds)
        .map_err(map_lifecycle_error)?;
    lifecycle
        .verify(trusted_time_epoch_seconds)
        .map_err(map_lifecycle_error)?;
    Ok(lifecycle)
}

fn retained_replay_time(
    retention_deadline_epoch_seconds: u64,
    latest_trusted_time_epoch_seconds: u64,
) -> Result<u64, CaptureLifecycleRestoreError> {
    if retention_deadline_epoch_seconds == 0 {
        return Err(CaptureLifecycleRestoreError::InvalidPersistedState);
    }
    Ok(latest_trusted_time_epoch_seconds.min(retention_deadline_epoch_seconds - 1))
}

fn deletion_replay_time(
    retention_deadline_epoch_seconds: u64,
    latest_trusted_time_epoch_seconds: u64,
) -> Result<u64, CaptureLifecycleRestoreError> {
    if retention_deadline_epoch_seconds == 0
        || retention_deadline_epoch_seconds > latest_trusted_time_epoch_seconds
    {
        return Err(CaptureLifecycleRestoreError::InvalidPersistedState);
    }
    Ok(retention_deadline_epoch_seconds - 1)
}

fn map_lifecycle_error(error: CaptureLifecycleError) -> CaptureLifecycleRestoreError {
    match error {
        CaptureLifecycleError::InvalidManifestDigest
        | CaptureLifecycleError::InvalidDeletionRequestDigest
        | CaptureLifecycleError::InvalidDeletionEvidenceDigest
        | CaptureLifecycleError::DeletionReceiptMismatch
        | CaptureLifecycleError::DeletionReceiptRequestMismatch => {
            CaptureLifecycleRestoreError::Lifecycle(error)
        }
        CaptureLifecycleError::InvalidTransition
        | CaptureLifecycleError::TrustedTimeRollback
        | CaptureLifecycleError::InvalidRetentionDeadline
        | CaptureLifecycleError::RetentionNotExpired
        | CaptureLifecycleError::LegalHoldActive => CaptureLifecycleRestoreError::InvalidPersistedState,
    }
}
