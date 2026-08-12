//! Credential-free deletion receipts for sensitive-data lifecycle evidence.
//!
//! These value objects record bounded metadata supplied by a trusted lifecycle owner after one
//! declared storage copy has been deleted or made cryptographically unavailable. They do not
//! perform deletion, authenticate the storage owner, retain the deleted value, or make an opaque
//! verification reference independently trustworthy.

use super::{MAX_SENSITIVE_IDENTIFIER_BYTES, SensitiveEvidenceError};

/// Declared storage-copy class whose deletion or cryptographic unavailability was verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SensitiveDeletionTarget {
    /// The authoritative record owned by the system of record.
    AuthoritativeRecord,
    /// A derived artifact produced from protected source data.
    DerivedArtifact,
    /// A model, adapter, or other model-associated artifact derived from protected data.
    ModelArtifact,
    /// An exported artifact delivered outside the authoritative store.
    ExportArtifact,
    /// A cache-resident copy.
    CacheCopy,
    /// An entry in a lexical or structured search index.
    SearchIndexEntry,
    /// An entry in a vector or embedding index.
    VectorIndexEntry,
    /// A temporary file or other explicitly ephemeral filesystem copy.
    TemporaryFile,
    /// A backup-resident copy governed by a separate backup lifecycle.
    BackupCopy,
}

/// Declared lifecycle cause for deleting or making a sensitive-data copy unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SensitiveDeletionCause {
    /// The copy reached its approved retention deadline.
    RetentionExpired,
    /// The owning tenant requested deletion.
    TenantDeletion,
    /// A data subject exercised an applicable deletion right.
    DataSubjectRequest,
    /// Cryptographic material required to recover the copy was revoked.
    KeyRevocation,
    /// A governing policy change required the copy to become unavailable.
    PolicyChange,
}

/// Unvalidated credential-free metadata for one sensitive-data deletion receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDeletionReceiptInput {
    /// Correlation identifier for the deletion request.
    pub request_id: String,
    /// Tenant that owned the deleted or unavailable copy.
    pub tenant_id: String,
    /// Opaque reference identifying the exact declared copy, never its protected value.
    pub target_reference: String,
    /// Identifier for the storage scope that owned the declared copy.
    pub storage_scope_id: String,
    /// Identifier for the retention or lifecycle policy governing the deletion.
    pub retention_policy_id: String,
    /// Opaque reference to separately held verification evidence.
    pub verification_reference: String,
    /// Declared storage-copy class.
    pub target: SensitiveDeletionTarget,
    /// Declared lifecycle cause.
    pub cause: SensitiveDeletionCause,
    /// Trusted Unix epoch second at which deletion or cryptographic unavailability completed.
    pub deletion_epoch_seconds: u64,
    /// Trusted Unix epoch second at which the lifecycle owner verified that outcome.
    pub verification_epoch_seconds: u64,
}

/// Immutable credential-free receipt for one verified sensitive-data deletion outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDeletionReceipt {
    request_id: String,
    tenant_id: String,
    target_reference: String,
    storage_scope_id: String,
    retention_policy_id: String,
    verification_reference: String,
    target: SensitiveDeletionTarget,
    cause: SensitiveDeletionCause,
    deletion_epoch_seconds: u64,
    verification_epoch_seconds: u64,
}

impl TryFrom<SensitiveDeletionReceiptInput> for SensitiveDeletionReceipt {
    type Error = SensitiveEvidenceError;

    fn try_from(input: SensitiveDeletionReceiptInput) -> Result<Self, Self::Error> {
        let identifiers = [
            input.request_id.as_str(),
            input.tenant_id.as_str(),
            input.target_reference.as_str(),
            input.storage_scope_id.as_str(),
            input.retention_policy_id.as_str(),
            input.verification_reference.as_str(),
        ];
        if identifiers.into_iter().any(|value| !valid_identifier(value)) {
            return Err(SensitiveEvidenceError::InvalidIdentifier);
        }
        if input.deletion_epoch_seconds == 0
            || input.verification_epoch_seconds == 0
            || input.verification_epoch_seconds < input.deletion_epoch_seconds
        {
            return Err(SensitiveEvidenceError::InvalidLifecycle);
        }
        Ok(Self {
            request_id: input.request_id,
            tenant_id: input.tenant_id,
            target_reference: input.target_reference,
            storage_scope_id: input.storage_scope_id,
            retention_policy_id: input.retention_policy_id,
            verification_reference: input.verification_reference,
            target: input.target,
            cause: input.cause,
            deletion_epoch_seconds: input.deletion_epoch_seconds,
            verification_epoch_seconds: input.verification_epoch_seconds,
        })
    }
}

impl SensitiveDeletionReceipt {
    /// Return the originating deletion-request identifier.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return the tenant identifier bound to this receipt.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Return the opaque reference for the exact declared copy.
    #[must_use]
    pub fn target_reference(&self) -> &str {
        &self.target_reference
    }

    /// Return the identifier of the storage scope that owned the declared copy.
    #[must_use]
    pub fn storage_scope_id(&self) -> &str {
        &self.storage_scope_id
    }

    /// Return the lifecycle or retention-policy identifier governing the deletion.
    #[must_use]
    pub fn retention_policy_id(&self) -> &str {
        &self.retention_policy_id
    }

    /// Return the opaque reference to separately held verification evidence.
    #[must_use]
    pub fn verification_reference(&self) -> &str {
        &self.verification_reference
    }

    /// Return the declared storage-copy class.
    #[must_use]
    pub const fn target(&self) -> SensitiveDeletionTarget {
        self.target
    }

    /// Return the declared lifecycle cause.
    #[must_use]
    pub const fn cause(&self) -> SensitiveDeletionCause {
        self.cause
    }

    /// Return the trusted deletion-completion time as a Unix epoch second.
    #[must_use]
    pub const fn deletion_epoch_seconds(&self) -> u64 {
        self.deletion_epoch_seconds
    }

    /// Return the trusted verification time as a Unix epoch second.
    #[must_use]
    pub const fn verification_epoch_seconds(&self) -> u64 {
        self.verification_epoch_seconds
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SENSITIVE_IDENTIFIER_BYTES
        && value.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}
