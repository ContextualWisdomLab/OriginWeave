use core::fmt;

use super::valid_sha256;

/// Lifecycle state for one manifest-bound capture package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureLifecycleState {
    /// Capture materialization has started but is not complete.
    CaptureStarted,
    /// Capture materialization completed but has not been independently verified.
    CaptureCompleted,
    /// The completed capture passed deterministic verification.
    Verified,
    /// The verified capture is retained until its explicit deadline.
    Retained,
    /// The verified or retained capture is protected by an explicit legal hold.
    LegalHold,
    /// Deletion was requested after the applicable retention and hold gates allowed it.
    DeletionRequested,
    /// The owning persistence boundary confirmed deletion completion.
    Deleted,
}

/// A fail-closed capture-lifecycle transition error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureLifecycleError {
    /// The supplied capture-manifest identity was not a lowercase SHA-256 digest.
    InvalidManifestDigest,
    /// The supplied deletion-evidence identity was not a lowercase SHA-256 digest.
    InvalidDeletionEvidenceDigest,
    /// A deletion receipt was bound to a different capture-manifest identity.
    DeletionReceiptMismatch,
    /// The requested transition is not allowed from the current lifecycle state.
    InvalidTransition,
    /// A caller supplied trusted time older than the latest accepted trusted time.
    TrustedTimeRollback,
    /// A retention deadline was not strictly later than the current trusted time.
    InvalidRetentionDeadline,
    /// Deletion was requested before the active retention deadline expired.
    RetentionNotExpired,
    /// Deletion was requested while an explicit legal hold remained active.
    LegalHoldActive,
}

impl fmt::Display for CaptureLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidManifestDigest => "capture manifest digest must be lowercase sha256",
            Self::InvalidDeletionEvidenceDigest => {
                "capture deletion evidence digest must be lowercase sha256"
            }
            Self::DeletionReceiptMismatch => {
                "capture deletion receipt does not match lifecycle manifest"
            }
            Self::InvalidTransition => {
                "capture lifecycle transition is not allowed from the current state"
            }
            Self::TrustedTimeRollback => "trusted capture lifecycle time moved backwards",
            Self::InvalidRetentionDeadline => {
                "capture retention deadline must be later than the current trusted time"
            }
            Self::RetentionNotExpired => "capture retention deadline has not expired",
            Self::LegalHoldActive => "capture is under legal hold",
        })
    }
}

impl std::error::Error for CaptureLifecycleError {}

/// Immutable identity receipt for persistence-side capture deletion evidence.
///
/// The receipt binds one canonical deletion-evidence digest to the exact capture
/// manifest that the evidence concerns. Construction validates identity shape
/// only. It does not authenticate the evidence producer, prove that storage bytes
/// were deleted, authorize deletion, or grant persistence, tenant, legal, replay,
/// export, or secret authority. The owning trusted persistence boundary must
/// authenticate the referenced evidence before presenting the receipt here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureDeletionReceipt {
    manifest_digest: String,
    evidence_digest: String,
}

impl CaptureDeletionReceipt {
    /// Bind one canonical deletion-evidence identity to one capture manifest.
    pub fn new(
        manifest_digest: &str,
        evidence_digest: &str,
    ) -> Result<Self, CaptureLifecycleError> {
        if !valid_sha256(manifest_digest) {
            return Err(CaptureLifecycleError::InvalidManifestDigest);
        }
        if !valid_sha256(evidence_digest) {
            return Err(CaptureLifecycleError::InvalidDeletionEvidenceDigest);
        }
        Ok(Self {
            manifest_digest: manifest_digest.to_owned(),
            evidence_digest: evidence_digest.to_owned(),
        })
    }

    /// Return the exact capture-manifest digest to which this receipt is bound.
    #[must_use]
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    /// Return the canonical deletion-evidence digest carried by this receipt.
    #[must_use]
    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }
}

/// In-memory policy-neutral lifecycle state for one capture-manifest identity.
///
/// This value object records only a manifest digest, lifecycle state, trusted
/// monotonic transition time, optional retention deadline, and an optional
/// manifest-bound deletion-evidence receipt after terminal confirmation. It does
/// not authenticate an operator or evidence producer, persist artifacts, decide
/// legal entitlement, delete storage objects, or grant capture, replay, export,
/// or secret access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureLifecycle {
    manifest_digest: String,
    state: CaptureLifecycleState,
    latest_trusted_time_epoch_seconds: u64,
    retention_deadline_epoch_seconds: Option<u64>,
    deletion_receipt: Option<CaptureDeletionReceipt>,
}

impl CaptureLifecycle {
    /// Start lifecycle tracking for one validated capture-manifest digest.
    pub fn new(
        manifest_digest: &str,
        trusted_time_epoch_seconds: u64,
    ) -> Result<Self, CaptureLifecycleError> {
        if !valid_sha256(manifest_digest) {
            return Err(CaptureLifecycleError::InvalidManifestDigest);
        }
        Ok(Self {
            manifest_digest: manifest_digest.to_owned(),
            state: CaptureLifecycleState::CaptureStarted,
            latest_trusted_time_epoch_seconds: trusted_time_epoch_seconds,
            retention_deadline_epoch_seconds: None,
            deletion_receipt: None,
        })
    }

    /// Return the immutable capture-manifest digest bound to this lifecycle.
    #[must_use]
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    /// Return the current capture lifecycle state.
    #[must_use]
    pub const fn state(&self) -> CaptureLifecycleState {
        self.state
    }

    /// Return the latest accepted trusted transition time in Unix epoch seconds.
    #[must_use]
    pub const fn latest_trusted_time_epoch_seconds(&self) -> u64 {
        self.latest_trusted_time_epoch_seconds
    }

    /// Return the active retention deadline, when one has been established.
    #[must_use]
    pub const fn retention_deadline_epoch_seconds(&self) -> Option<u64> {
        self.retention_deadline_epoch_seconds
    }

    /// Return the deletion-evidence receipt retained after terminal confirmation.
    ///
    /// This receipt is identity evidence only and does not itself prove that the
    /// referenced persistence operation was authentic or complete.
    #[must_use]
    pub const fn deletion_receipt(&self) -> Option<&CaptureDeletionReceipt> {
        self.deletion_receipt.as_ref()
    }

    /// Mark a started capture complete.
    pub fn complete(
        &mut self,
        trusted_time_epoch_seconds: u64,
    ) -> Result<(), CaptureLifecycleError> {
        self.require_trusted_time_not_rollback(trusted_time_epoch_seconds)?;
        self.require_state(CaptureLifecycleState::CaptureStarted)?;
        self.state = CaptureLifecycleState::CaptureCompleted;
        self.accept_trusted_time(trusted_time_epoch_seconds);
        Ok(())
    }

    /// Mark a completed capture independently verified.
    pub fn verify(&mut self, trusted_time_epoch_seconds: u64) -> Result<(), CaptureLifecycleError> {
        self.require_trusted_time_not_rollback(trusted_time_epoch_seconds)?;
        self.require_state(CaptureLifecycleState::CaptureCompleted)?;
        self.state = CaptureLifecycleState::Verified;
        self.accept_trusted_time(trusted_time_epoch_seconds);
        Ok(())
    }

    /// Retain a verified capture until a future trusted-time deadline.
    pub fn retain_until(
        &mut self,
        retention_deadline_epoch_seconds: u64,
        trusted_time_epoch_seconds: u64,
    ) -> Result<(), CaptureLifecycleError> {
        self.require_trusted_time_not_rollback(trusted_time_epoch_seconds)?;
        self.require_state(CaptureLifecycleState::Verified)?;
        Self::require_future_deadline(
            retention_deadline_epoch_seconds,
            trusted_time_epoch_seconds,
        )?;
        self.retention_deadline_epoch_seconds = Some(retention_deadline_epoch_seconds);
        self.state = CaptureLifecycleState::Retained;
        self.accept_trusted_time(trusted_time_epoch_seconds);
        Ok(())
    }

    /// Place a verified or retained capture under an explicit legal hold.
    pub fn place_legal_hold(
        &mut self,
        trusted_time_epoch_seconds: u64,
    ) -> Result<(), CaptureLifecycleError> {
        self.require_trusted_time_not_rollback(trusted_time_epoch_seconds)?;
        match self.state {
            CaptureLifecycleState::Verified | CaptureLifecycleState::Retained => {
                self.state = CaptureLifecycleState::LegalHold;
                self.accept_trusted_time(trusted_time_epoch_seconds);
                Ok(())
            }
            _ => Err(CaptureLifecycleError::InvalidTransition),
        }
    }

    /// Release a legal hold into a newly bounded future retention period.
    ///
    /// A legal hold is never released directly into deletion eligibility. The
    /// caller must supply a new future retention deadline before deletion can
    /// later be requested.
    pub fn release_legal_hold_to_retained(
        &mut self,
        retention_deadline_epoch_seconds: u64,
        trusted_time_epoch_seconds: u64,
    ) -> Result<(), CaptureLifecycleError> {
        self.require_trusted_time_not_rollback(trusted_time_epoch_seconds)?;
        self.require_state(CaptureLifecycleState::LegalHold)?;
        Self::require_future_deadline(
            retention_deadline_epoch_seconds,
            trusted_time_epoch_seconds,
        )?;
        self.retention_deadline_epoch_seconds = Some(retention_deadline_epoch_seconds);
        self.state = CaptureLifecycleState::Retained;
        self.accept_trusted_time(trusted_time_epoch_seconds);
        Ok(())
    }

    /// Request deletion after the active retention period has expired.
    pub fn request_deletion(
        &mut self,
        trusted_time_epoch_seconds: u64,
    ) -> Result<(), CaptureLifecycleError> {
        self.require_trusted_time_not_rollback(trusted_time_epoch_seconds)?;
        if self.state == CaptureLifecycleState::LegalHold {
            return Err(CaptureLifecycleError::LegalHoldActive);
        }
        self.require_state(CaptureLifecycleState::Retained)?;
        self.retention_deadline_epoch_seconds
            .ok_or(CaptureLifecycleError::InvalidTransition)
            .and_then(|deadline| {
                if trusted_time_epoch_seconds < deadline {
                    Err(CaptureLifecycleError::RetentionNotExpired)
                } else {
                    self.state = CaptureLifecycleState::DeletionRequested;
                    self.accept_trusted_time(trusted_time_epoch_seconds);
                    Ok(())
                }
            })
    }

    /// Confirm deletion with manifest-bound evidence from the owning persistence boundary.
    ///
    /// The receipt must bind the exact lifecycle manifest before lifecycle or
    /// trusted-time state is consulted. Successful confirmation retains the exact
    /// receipt so terminal `Deleted` state cannot exist without a deletion-evidence
    /// identity. The receipt itself remains non-authenticating; callers must verify
    /// the evidence at the trusted persistence boundary before invoking this method.
    pub fn confirm_deleted(
        &mut self,
        receipt: &CaptureDeletionReceipt,
        trusted_time_epoch_seconds: u64,
    ) -> Result<(), CaptureLifecycleError> {
        if receipt.manifest_digest() != self.manifest_digest {
            return Err(CaptureLifecycleError::DeletionReceiptMismatch);
        }
        self.require_trusted_time_not_rollback(trusted_time_epoch_seconds)?;
        self.require_state(CaptureLifecycleState::DeletionRequested)?;
        self.deletion_receipt = Some(receipt.clone());
        self.state = CaptureLifecycleState::Deleted;
        self.accept_trusted_time(trusted_time_epoch_seconds);
        Ok(())
    }

    fn require_trusted_time_not_rollback(
        &self,
        trusted_time_epoch_seconds: u64,
    ) -> Result<(), CaptureLifecycleError> {
        if trusted_time_epoch_seconds < self.latest_trusted_time_epoch_seconds {
            return Err(CaptureLifecycleError::TrustedTimeRollback);
        }
        Ok(())
    }

    fn accept_trusted_time(&mut self, trusted_time_epoch_seconds: u64) {
        self.latest_trusted_time_epoch_seconds = trusted_time_epoch_seconds;
    }

    fn require_state(
        &self,
        required_state: CaptureLifecycleState,
    ) -> Result<(), CaptureLifecycleError> {
        if self.state != required_state {
            return Err(CaptureLifecycleError::InvalidTransition);
        }
        Ok(())
    }

    fn require_future_deadline(
        retention_deadline_epoch_seconds: u64,
        trusted_time_epoch_seconds: u64,
    ) -> Result<(), CaptureLifecycleError> {
        if retention_deadline_epoch_seconds <= trusted_time_epoch_seconds {
            return Err(CaptureLifecycleError::InvalidRetentionDeadline);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CaptureLifecycle, CaptureLifecycleError, CaptureLifecycleState};

    #[test]
    fn corrupted_retained_state_without_deadline_fails_closed() {
        let mut lifecycle = CaptureLifecycle {
            manifest_digest:
                "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned(),
            state: CaptureLifecycleState::Retained,
            latest_trusted_time_epoch_seconds: 100,
            retention_deadline_epoch_seconds: None,
            deletion_receipt: None,
        };

        assert_eq!(
            lifecycle.request_deletion(101),
            Err(CaptureLifecycleError::InvalidTransition)
        );
        assert_eq!(lifecycle.state(), CaptureLifecycleState::Retained);
        assert_eq!(lifecycle.latest_trusted_time_epoch_seconds(), 100);
        assert!(lifecycle.deletion_receipt().is_none());
    }
}
