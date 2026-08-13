//! Credential-free deletion receipts for sensitive-data lifecycle evidence.
//!
//! These value objects record bounded metadata supplied by a trusted lifecycle owner after one
//! declared storage copy has been deleted or made cryptographically unavailable. They do not
//! perform deletion, authenticate the storage owner, retain the deleted value, or make an opaque
//! verification reference independently trustworthy.

use std::collections::BTreeSet;
use std::fmt;

use sha2::{Digest, Sha256};

use super::{MAX_SENSITIVE_IDENTIFIER_BYTES, SensitiveEvidenceError};

const DELETION_INVENTORY_COMMITMENT_DOMAIN: &[u8] =
    b"originweave-sensitive-deletion-inventory-v1\0";

/// Maximum number of declared copies or receipts verified in one deletion-set evaluation.
pub const MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES: usize = 256;

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

/// Exact credential-free copy declaration that must be represented by one deletion receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDeletionRequirement {
    target: SensitiveDeletionTarget,
    target_reference: String,
    storage_scope_id: String,
}

impl SensitiveDeletionRequirement {
    /// Validate one exact declared copy without retaining any protected value.
    pub fn new(
        target: SensitiveDeletionTarget,
        target_reference: &str,
        storage_scope_id: &str,
    ) -> Result<Self, SensitiveEvidenceError> {
        if !valid_identifier(target_reference) || !valid_identifier(storage_scope_id) {
            return Err(SensitiveEvidenceError::InvalidIdentifier);
        }
        Ok(Self {
            target,
            target_reference: target_reference.to_owned(),
            storage_scope_id: storage_scope_id.to_owned(),
        })
    }
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
        if identifiers
            .into_iter()
            .any(|value| !valid_identifier(value))
        {
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

/// Failure returned when declared deletion requirements and supplied receipts are not complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveDeletionReceiptSetError {
    /// The caller supplied more declared-copy requirements than one evaluation may process.
    TooManyRequirements,
    /// The caller supplied more deletion receipts than one evaluation may process.
    TooManyReceipts,
    /// A caller-supplied request, tenant, or retention-policy identifier is invalid or oversized.
    InvalidScopeIdentifier,
    /// At least one receipt belongs to another deletion request.
    RequestMismatch,
    /// At least one receipt belongs to another tenant.
    TenantMismatch,
    /// At least one receipt uses another retention policy.
    RetentionPolicyMismatch,
    /// No exact-copy requirements were declared.
    EmptyRequirementSet,
    /// The same exact-copy requirement was declared more than once.
    DuplicateRequirement,
    /// At least one declared exact copy has no receipt.
    MissingReceipt,
    /// At least one receipt names an exact copy that was not declared.
    UnexpectedReceipt,
    /// More than one receipt names the same exact declared copy.
    DuplicateReceipt,
}

impl fmt::Display for SensitiveDeletionReceiptSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TooManyRequirements => "too many sensitive deletion requirements",
            Self::TooManyReceipts => "too many sensitive deletion receipts",
            Self::InvalidScopeIdentifier => "invalid sensitive deletion scope identifier",
            Self::RequestMismatch => "sensitive deletion request mismatch",
            Self::TenantMismatch => "sensitive deletion tenant mismatch",
            Self::RetentionPolicyMismatch => "sensitive deletion retention policy mismatch",
            Self::EmptyRequirementSet => "empty sensitive deletion requirement set",
            Self::DuplicateRequirement => "duplicate sensitive deletion requirement",
            Self::MissingReceipt => "missing sensitive deletion receipt",
            Self::UnexpectedReceipt => "unexpected sensitive deletion receipt",
            Self::DuplicateReceipt => "duplicate sensitive deletion receipt",
        })
    }
}

impl std::error::Error for SensitiveDeletionReceiptSetError {}

/// Credential-free commitment to the exact caller-declared inventory that passed verification.
///
/// The commitment retains bounded request-scope identifiers, declared-copy count, and a
/// deterministic SHA-256 digest. It does not retain target references or storage-scope identifiers,
/// discover undeclared copies, prove inventory exhaustiveness, authenticate a storage owner, or
/// make the digest an authenticated signature or durable record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDeletionReceiptSetCommitment {
    request_id: String,
    tenant_id: String,
    retention_policy_id: String,
    declared_copy_count: usize,
    inventory_digest: String,
}

impl SensitiveDeletionReceiptSetCommitment {
    /// Return the deletion request whose verified inventory produced this commitment.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return the tenant whose verified inventory produced this commitment.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Return the retention policy bound to the verified inventory.
    #[must_use]
    pub fn retention_policy_id(&self) -> &str {
        &self.retention_policy_id
    }

    /// Return the number of exact declared copies bound into the commitment.
    #[must_use]
    pub const fn declared_copy_count(&self) -> usize {
        self.declared_copy_count
    }

    /// Return the lowercase SHA-256 digest of the canonical declared-copy inventory preimage.
    #[must_use]
    pub fn inventory_digest(&self) -> &str {
        &self.inventory_digest
    }
}

/// Verify that every exact declared sensitive-data copy has exactly one matching deletion receipt.
///
/// This comparison is credential-free and deliberately does not discover copies, authenticate
/// storage owners, perform deletion, or prove that the caller's requirement set is exhaustive.
/// Caller-supplied set sizes and scope identifiers are bounded before internal index allocation or
/// entry-level validation.
pub fn verify_sensitive_deletion_receipt_set(
    receipts: &[SensitiveDeletionReceipt],
    request_id: &str,
    tenant_id: &str,
    retention_policy_id: &str,
    requirements: &[SensitiveDeletionRequirement],
) -> Result<(), SensitiveDeletionReceiptSetError> {
    if requirements.len() > MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES {
        return Err(SensitiveDeletionReceiptSetError::TooManyRequirements);
    }
    if receipts.len() > MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES {
        return Err(SensitiveDeletionReceiptSetError::TooManyReceipts);
    }
    if !valid_identifier(request_id)
        || !valid_identifier(tenant_id)
        || !valid_identifier(retention_policy_id)
    {
        return Err(SensitiveDeletionReceiptSetError::InvalidScopeIdentifier);
    }
    if requirements.is_empty() {
        return Err(SensitiveDeletionReceiptSetError::EmptyRequirementSet);
    }

    let mut required_copies = BTreeSet::new();
    for requirement in requirements {
        let key = (
            requirement.target,
            requirement.target_reference.as_str(),
            requirement.storage_scope_id.as_str(),
        );
        if !required_copies.insert(key) {
            return Err(SensitiveDeletionReceiptSetError::DuplicateRequirement);
        }
    }

    let mut received_copies = BTreeSet::new();
    for receipt in receipts {
        if receipt.request_id() != request_id {
            return Err(SensitiveDeletionReceiptSetError::RequestMismatch);
        }
        if receipt.tenant_id() != tenant_id {
            return Err(SensitiveDeletionReceiptSetError::TenantMismatch);
        }
        if receipt.retention_policy_id() != retention_policy_id {
            return Err(SensitiveDeletionReceiptSetError::RetentionPolicyMismatch);
        }

        let key = (
            receipt.target(),
            receipt.target_reference(),
            receipt.storage_scope_id(),
        );
        if !required_copies.contains(&key) {
            return Err(SensitiveDeletionReceiptSetError::UnexpectedReceipt);
        }
        if !received_copies.insert(key) {
            return Err(SensitiveDeletionReceiptSetError::DuplicateReceipt);
        }
    }

    if received_copies.len() != required_copies.len() {
        return Err(SensitiveDeletionReceiptSetError::MissingReceipt);
    }
    Ok(())
}

/// Verify an exact deletion receipt set and commit to the declared inventory that passed.
///
/// Verification runs before commitment construction, so no commitment is returned for malformed,
/// incomplete, cross-scope, duplicate, unexpected, or oversized input. The SHA-256 digest is a
/// deterministic commitment to the bounded caller-supplied inventory and request scope, not proof
/// that the inventory enumerates every real copy and not an authenticated signature. Target
/// references and storage-scope identifiers are hashed into the commitment but are not retained by
/// the returned value; callers must still treat the digest as potentially linkable metadata.
pub fn verify_sensitive_deletion_receipt_set_with_commitment(
    receipts: &[SensitiveDeletionReceipt],
    request_id: &str,
    tenant_id: &str,
    retention_policy_id: &str,
    requirements: &[SensitiveDeletionRequirement],
) -> Result<SensitiveDeletionReceiptSetCommitment, SensitiveDeletionReceiptSetError> {
    verify_sensitive_deletion_receipt_set(
        receipts,
        request_id,
        tenant_id,
        retention_policy_id,
        requirements,
    )?;

    let ordered_requirements = requirements
        .iter()
        .map(|requirement| {
            (
                requirement.target,
                requirement.target_reference.as_str(),
                requirement.storage_scope_id.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    let mut preimage = DELETION_INVENTORY_COMMITMENT_DOMAIN.to_vec();
    append_length_delimited(&mut preimage, request_id.as_bytes());
    append_length_delimited(&mut preimage, tenant_id.as_bytes());
    append_length_delimited(&mut preimage, retention_policy_id.as_bytes());
    let declared_copy_count = requirements.len().to_string();
    append_length_delimited(&mut preimage, declared_copy_count.as_bytes());
    for (target, target_reference, storage_scope_id) in ordered_requirements {
        append_length_delimited(&mut preimage, deletion_target_token(target));
        append_length_delimited(&mut preimage, target_reference.as_bytes());
        append_length_delimited(&mut preimage, storage_scope_id.as_bytes());
    }
    let digest = Sha256::digest(preimage);

    Ok(SensitiveDeletionReceiptSetCommitment {
        request_id: request_id.to_owned(),
        tenant_id: tenant_id.to_owned(),
        retention_policy_id: retention_policy_id.to_owned(),
        declared_copy_count: requirements.len(),
        inventory_digest: format!("{digest:x}"),
    })
}

const fn deletion_target_token(target: SensitiveDeletionTarget) -> &'static [u8] {
    match target {
        SensitiveDeletionTarget::AuthoritativeRecord => b"authoritative_record",
        SensitiveDeletionTarget::DerivedArtifact => b"derived_artifact",
        SensitiveDeletionTarget::ModelArtifact => b"model_artifact",
        SensitiveDeletionTarget::ExportArtifact => b"export_artifact",
        SensitiveDeletionTarget::CacheCopy => b"cache_copy",
        SensitiveDeletionTarget::SearchIndexEntry => b"search_index_entry",
        SensitiveDeletionTarget::VectorIndexEntry => b"vector_index_entry",
        SensitiveDeletionTarget::TemporaryFile => b"temporary_file",
        SensitiveDeletionTarget::BackupCopy => b"backup_copy",
    }
}

fn append_length_delimited(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(value.len().to_string().as_bytes());
    output.push(b':');
    output.extend_from_slice(value);
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SENSITIVE_IDENTIFIER_BYTES
        && value.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}
