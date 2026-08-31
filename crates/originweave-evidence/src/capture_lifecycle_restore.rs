impl CaptureLifecycleState {
    /// Return the exact storage-neutral token for this lifecycle state.
    ///
    /// These tokens are the only accepted persisted textual representation. They are
    /// intentionally independent of Rust variant names so persistence adapters do not
    /// rely on `Debug` output or case conversion conventions that can drift across
    /// releases.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CaptureStarted => "capture_started",
            Self::CaptureCompleted => "capture_completed",
            Self::Verified => "verified",
            Self::Retained => "retained",
            Self::LegalHold => "legal_hold",
            Self::DeletionRequested => "deletion_requested",
            Self::Deleted => "deleted",
        }
    }

    /// Parse one exact storage-neutral lifecycle token without accepting aliases.
    ///
    /// Unknown values fail closed as `InvalidRestoredState`; callers must not infer a
    /// lifecycle transition from differently cased, hyphenated, padded, or future
    /// values. This parser grants no persistence, tenant, legal, deletion, or replay
    /// authority.
    pub fn parse(value: &str) -> Result<Self, CaptureLifecycleError> {
        match value {
            "capture_started" => Ok(Self::CaptureStarted),
            "capture_completed" => Ok(Self::CaptureCompleted),
            "verified" => Ok(Self::Verified),
            "retained" => Ok(Self::Retained),
            "legal_hold" => Ok(Self::LegalHold),
            "deletion_requested" => Ok(Self::DeletionRequested),
            "deleted" => Ok(Self::Deleted),
            _ => Err(CaptureLifecycleError::InvalidRestoredState),
        }
    }
}

impl CaptureLifecycle {
    /// Reconstruct one previously persisted lifecycle snapshot without widening authority.
    ///
    /// Restoration accepts only field combinations that can be produced by the public
    /// lifecycle transition API. The snapshot is validated directly against the same manifest,
    /// retention, legal-hold, request, receipt, and trusted-time invariants before any state is
    /// reconstructed. The caller remains responsible for authenticating the persistence record,
    /// trusted time, deletion request, deletion evidence, tenant context, and legal authority
    /// before supplying the snapshot here. This method does not read storage, authenticate an
    /// actor, delete bytes, grant access, or convert malformed persistence into a successful
    /// lifecycle state.
    pub fn restore(
        manifest_digest: &str,
        state: CaptureLifecycleState,
        latest_trusted_time_epoch_seconds: u64,
        retention_deadline_epoch_seconds: Option<u64>,
        deletion_request_digest: Option<&str>,
        deletion_receipt: Option<&CaptureDeletionReceipt>,
    ) -> Result<Self, CaptureLifecycleError> {
        if !valid_sha256(manifest_digest) {
            return Err(CaptureLifecycleError::InvalidManifestDigest);
        }

        let (retention_deadline_epoch_seconds, deletion_request_digest, deletion_receipt) = match (
            state,
            retention_deadline_epoch_seconds,
            deletion_request_digest,
            deletion_receipt,
        ) {
            (
                CaptureLifecycleState::CaptureStarted
                | CaptureLifecycleState::CaptureCompleted
                | CaptureLifecycleState::Verified,
                None,
                None,
                None,
            ) => (None, None, None),
            (CaptureLifecycleState::Retained, Some(deadline), None, None) => {
                if deadline <= latest_trusted_time_epoch_seconds {
                    return Err(CaptureLifecycleError::InvalidRestoredState);
                }
                (Some(deadline), None, None)
            }
            (CaptureLifecycleState::LegalHold, None, None, None) => (None, None, None),
            (CaptureLifecycleState::LegalHold, Some(deadline), None, None) => {
                if deadline == 0 {
                    return Err(CaptureLifecycleError::InvalidRestoredState);
                }
                (Some(deadline), None, None)
            }
            (
                CaptureLifecycleState::DeletionRequested | CaptureLifecycleState::Deleted,
                Some(deadline),
                Some(request),
                receipt,
            ) => {
                if deadline == 0 || deadline > latest_trusted_time_epoch_seconds {
                    return Err(CaptureLifecycleError::InvalidRestoredState);
                }
                if !valid_sha256(request) {
                    return Err(CaptureLifecycleError::InvalidDeletionRequestDigest);
                }

                let receipt = match (state, receipt) {
                    (CaptureLifecycleState::DeletionRequested, None) => None,
                    (CaptureLifecycleState::Deleted, Some(receipt)) => {
                        if receipt.manifest_digest() != manifest_digest {
                            return Err(CaptureLifecycleError::DeletionReceiptMismatch);
                        }
                        if receipt.deletion_request_digest() != request {
                            return Err(CaptureLifecycleError::DeletionReceiptRequestMismatch);
                        }
                        Some(receipt.clone())
                    }
                    _ => return Err(CaptureLifecycleError::InvalidRestoredState),
                };

                (Some(deadline), Some(request.to_owned()), receipt)
            }
            _ => return Err(CaptureLifecycleError::InvalidRestoredState),
        };

        Ok(Self {
            manifest_digest: manifest_digest.to_owned(),
            state,
            latest_trusted_time_epoch_seconds,
            retention_deadline_epoch_seconds,
            deletion_request_digest,
            deletion_receipt,
        })
    }
}
