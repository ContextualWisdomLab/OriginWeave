//! Credential-free lifecycle evidence for opaque sensitive-value handles.
//!
//! A trusted broker can use this value object to record when a handle was
//! issued, when it expires, how many uses it permits, how many resolutions were
//! observed, and when it was revoked. The lifecycle retains the complete
//! credential-free sensitive-access receipt that authorized opaque-handle use,
//! while intentionally excluding the opaque handle token and protected value.

use crate::sensitive_access::{
    SensitiveAccessEvidence, SensitiveAccessOutcome, SensitiveEvidenceError,
};

/// Unvalidated metadata describing one opaque sensitive-value handle lifecycle.
///
/// The embedded access receipt binds the lifecycle to the tenant, actor, task,
/// field set, purpose, destination, classification, policy version, and exact
/// opaque-handle authorization without carrying protected values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveHandleLifecycleEvidenceInput {
    /// Credential-free access receipt that authorized this opaque handle.
    pub access_evidence: SensitiveAccessEvidence,
    /// Trusted Unix epoch second when the handle was issued.
    pub issued_epoch_seconds: u64,
    /// Trusted Unix epoch second after which the handle is no longer valid.
    pub expires_epoch_seconds: u64,
    /// Maximum number of broker resolutions authorized for the handle.
    pub maximum_uses: u32,
    /// Number of broker resolutions already observed for the handle.
    pub resolution_count: u32,
    /// Trusted Unix epoch second when the handle was revoked, when applicable.
    pub revoked_epoch_seconds: Option<u64>,
}

/// Immutable credential-free evidence about one opaque handle lifecycle.
///
/// The value retains the exact credential-free sensitive-access receipt that
/// authorized opaque-handle use, but deliberately excludes both the opaque
/// handle token and the secret or protected value that the broker can resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveHandleLifecycleEvidence {
    access_evidence: SensitiveAccessEvidence,
    issued_epoch_seconds: u64,
    expires_epoch_seconds: u64,
    maximum_uses: u32,
    resolution_count: u32,
    revoked_epoch_seconds: Option<u64>,
}

impl TryFrom<SensitiveHandleLifecycleEvidenceInput> for SensitiveHandleLifecycleEvidence {
    type Error = SensitiveEvidenceError;

    fn try_from(input: SensitiveHandleLifecycleEvidenceInput) -> Result<Self, Self::Error> {
        if input.access_evidence.outcome() != SensitiveAccessOutcome::OpaqueHandleOnly
            || input.issued_epoch_seconds == 0
            || input.issued_epoch_seconds < input.access_evidence.decision_epoch_seconds()
            || input.expires_epoch_seconds <= input.issued_epoch_seconds
            || input.maximum_uses == 0
            || input.resolution_count > input.maximum_uses
            || input.revoked_epoch_seconds.is_some_and(|revoked| {
                revoked < input.issued_epoch_seconds || revoked >= input.expires_epoch_seconds
            })
        {
            return Err(SensitiveEvidenceError::InvalidLifecycle);
        }

        Ok(Self {
            access_evidence: input.access_evidence,
            issued_epoch_seconds: input.issued_epoch_seconds,
            expires_epoch_seconds: input.expires_epoch_seconds,
            maximum_uses: input.maximum_uses,
            resolution_count: input.resolution_count,
            revoked_epoch_seconds: input.revoked_epoch_seconds,
        })
    }
}

impl SensitiveHandleLifecycleEvidence {
    /// Return the credential-free access receipt that authorized this opaque handle.
    #[must_use]
    pub const fn access_evidence(&self) -> &SensitiveAccessEvidence {
        &self.access_evidence
    }

    /// Return the originating sensitive-data access request identifier.
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.access_evidence.request_id()
    }

    /// Return the policy decision identifier associated with the handle.
    #[must_use]
    pub fn decision_id(&self) -> &str {
        self.access_evidence.decision_id()
    }

    /// Return the trusted handle issuance time as a Unix epoch second.
    #[must_use]
    pub const fn issued_epoch_seconds(&self) -> u64 {
        self.issued_epoch_seconds
    }

    /// Return the trusted handle expiry time as a Unix epoch second.
    #[must_use]
    pub const fn expires_epoch_seconds(&self) -> u64 {
        self.expires_epoch_seconds
    }

    /// Return the maximum number of broker resolutions authorized for the handle.
    #[must_use]
    pub const fn maximum_uses(&self) -> u32 {
        self.maximum_uses
    }

    /// Return the number of broker resolutions already observed for the handle.
    #[must_use]
    pub const fn resolution_count(&self) -> u32 {
        self.resolution_count
    }

    /// Return the trusted revocation time when the handle has been revoked.
    #[must_use]
    pub const fn revoked_epoch_seconds(&self) -> Option<u64> {
        self.revoked_epoch_seconds
    }

    /// Return whether trusted evidence records that this handle was revoked.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        self.revoked_epoch_seconds.is_some()
    }
}
