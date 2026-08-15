//! Explicit wire-version envelope for persisted deletion inventory commitments.
//!
//! The in-process commitment value remains source-compatible for callers that already construct or
//! verify it. This module adds a separate durable boundary that records the canonicalization version
//! before reconstructing that bounded value, so future wire-format changes cannot silently
//! reinterpret stored evidence.

use std::fmt;

use super::{
    SensitiveDeletionInventoryCommitmentError, SensitiveDeletionReceipt,
    SensitiveDeletionReceiptSetCommitment, SensitiveDeletionReceiptSetCommitmentInput,
    SensitiveDeletionReceiptSetError, SensitiveDeletionRequirement,
    verify_sensitive_deletion_inventory_commitment,
    verify_sensitive_deletion_receipt_set_with_commitment,
};

const PERSISTED_COMMITMENT_WIRE_DOMAIN: &[u8] =
    b"originweave-sensitive-deletion-persisted-commitment\0";

/// Current durable wire version for sensitive deletion inventory commitments.
pub const SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION: u16 = 1;

/// Unvalidated durable metadata for one versioned sensitive deletion inventory commitment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDeletionPersistedCommitmentInput {
    /// Explicit durable wire and canonicalization version.
    pub commitment_version: u16,
    /// Deletion request identifier recorded with the commitment.
    pub request_id: String,
    /// Tenant identifier recorded with the commitment.
    pub tenant_id: String,
    /// Retention or lifecycle-policy identifier recorded with the commitment.
    pub retention_policy_id: String,
    /// Number of exact caller-declared copies represented by the commitment.
    pub declared_copy_count: u16,
    /// Lowercase 64-character SHA-256 digest of the canonical inventory preimage.
    pub inventory_digest: String,
}

/// Failure returned when a versioned persisted deletion commitment cannot be reconstructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveDeletionPersistedCommitmentError {
    /// The durable envelope uses a wire version unsupported by this OriginWeave build.
    UnsupportedCommitmentVersion,
    /// The version is supported, but the enclosed commitment metadata is invalid.
    InvalidCommitment(SensitiveDeletionInventoryCommitmentError),
}

impl fmt::Display for SensitiveDeletionPersistedCommitmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCommitmentVersion => {
                formatter.write_str("unsupported persisted sensitive deletion commitment version")
            }
            Self::InvalidCommitment(error) => {
                write!(
                    formatter,
                    "invalid persisted sensitive deletion commitment: {error}"
                )
            }
        }
    }
}

impl std::error::Error for SensitiveDeletionPersistedCommitmentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedCommitmentVersion => None,
            Self::InvalidCommitment(error) => Some(error),
        }
    }
}

/// Versioned durable envelope around one structurally validated deletion inventory commitment.
///
/// The envelope authenticates nothing by itself. Storage owners must protect persisted metadata
/// with their own integrity and access controls; this value only prevents silent interpretation of
/// a stored digest under an unknown canonicalization version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDeletionPersistedCommitment {
    commitment_version: u16,
    commitment: SensitiveDeletionReceiptSetCommitment,
}

impl TryFrom<SensitiveDeletionPersistedCommitmentInput> for SensitiveDeletionPersistedCommitment {
    type Error = SensitiveDeletionPersistedCommitmentError;

    fn try_from(input: SensitiveDeletionPersistedCommitmentInput) -> Result<Self, Self::Error> {
        if input.commitment_version != SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION {
            return Err(SensitiveDeletionPersistedCommitmentError::UnsupportedCommitmentVersion);
        }

        let commitment = SensitiveDeletionReceiptSetCommitment::try_from(
            SensitiveDeletionReceiptSetCommitmentInput {
                request_id: input.request_id,
                tenant_id: input.tenant_id,
                retention_policy_id: input.retention_policy_id,
                declared_copy_count: usize::from(input.declared_copy_count),
                inventory_digest: input.inventory_digest,
            },
        )
        .map_err(SensitiveDeletionPersistedCommitmentError::InvalidCommitment)?;

        Ok(Self {
            commitment_version: input.commitment_version,
            commitment,
        })
    }
}

impl From<&SensitiveDeletionPersistedCommitment> for SensitiveDeletionPersistedCommitmentInput {
    fn from(commitment: &SensitiveDeletionPersistedCommitment) -> Self {
        Self {
            commitment_version: commitment.commitment_version(),
            request_id: commitment.request_id().to_owned(),
            tenant_id: commitment.tenant_id().to_owned(),
            retention_policy_id: commitment.retention_policy_id().to_owned(),
            declared_copy_count: commitment.declared_copy_count(),
            inventory_digest: commitment.inventory_digest().to_owned(),
        }
    }
}

impl SensitiveDeletionPersistedCommitment {
    /// Return the durable wire and canonicalization version.
    #[must_use]
    pub const fn commitment_version(&self) -> u16 {
        self.commitment_version
    }

    /// Return the deletion request bound to the enclosed commitment.
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.commitment.request_id()
    }

    /// Return the tenant bound to the enclosed commitment.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        self.commitment.tenant_id()
    }

    /// Return the retention policy bound to the enclosed commitment.
    #[must_use]
    pub fn retention_policy_id(&self) -> &str {
        self.commitment.retention_policy_id()
    }

    /// Return the exact declared-copy count bound to the enclosed commitment.
    ///
    /// Construction limits this value to at most 256, so conversion to the
    /// fixed-width persisted representation is lossless.
    #[must_use]
    pub const fn declared_copy_count(&self) -> u16 {
        self.commitment.declared_copy_count() as u16
    }

    /// Return the canonical lowercase SHA-256 inventory digest.
    #[must_use]
    pub fn inventory_digest(&self) -> &str {
        self.commitment.inventory_digest()
    }

    /// Encode the validated commitment into deterministic architecture-independent wire bytes.
    ///
    /// The wire form starts with a domain separator and then length-delimits, in order, the wire
    /// version, request, tenant, retention policy, declared-copy count, and inventory digest. This
    /// makes field boundaries and integer text representation explicit for durable signing,
    /// hashing, or cross-process storage. The bytes are not authenticated by this method and still
    /// require an external integrity and access-control boundary.
    #[must_use]
    pub fn canonical_wire_bytes(&self) -> Vec<u8> {
        let mut output = PERSISTED_COMMITMENT_WIRE_DOMAIN.to_vec();
        let commitment_version = self.commitment_version().to_string();
        let declared_copy_count = self.declared_copy_count().to_string();
        append_persisted_wire_field(&mut output, commitment_version.as_bytes());
        append_persisted_wire_field(&mut output, self.request_id().as_bytes());
        append_persisted_wire_field(&mut output, self.tenant_id().as_bytes());
        append_persisted_wire_field(&mut output, self.retention_policy_id().as_bytes());
        append_persisted_wire_field(&mut output, declared_copy_count.as_bytes());
        append_persisted_wire_field(&mut output, self.inventory_digest().as_bytes());
        output
    }
}

fn append_persisted_wire_field(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(value.len().to_string().as_bytes());
    output.push(b':');
    output.extend_from_slice(value);
}

/// Verify an exact deletion receipt set and return a versioned durable commitment envelope.
///
/// Exact receipt-set verification completes before an envelope is emitted. The version describes
/// only this crate's canonical wire semantics; it is not an authenticity or persistence guarantee.
pub fn verify_sensitive_deletion_receipt_set_with_persisted_commitment(
    receipts: &[SensitiveDeletionReceipt],
    request_id: &str,
    tenant_id: &str,
    retention_policy_id: &str,
    requirements: &[SensitiveDeletionRequirement],
) -> Result<SensitiveDeletionPersistedCommitment, SensitiveDeletionReceiptSetError> {
    let commitment = verify_sensitive_deletion_receipt_set_with_commitment(
        receipts,
        request_id,
        tenant_id,
        retention_policy_id,
        requirements,
    )?;

    Ok(SensitiveDeletionPersistedCommitment {
        commitment_version: SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION,
        commitment,
    })
}

/// Verify a reconstructed versioned commitment against exact expected scope and inventory.
///
/// Reconstruction already rejects unsupported wire versions. This verifier delegates exact scope,
/// count, duplicate-inventory, and canonical digest checks to the existing bounded commitment
/// authority; success does not authenticate persistence or prove inventory exhaustiveness.
pub fn verify_persisted_sensitive_deletion_inventory_commitment(
    commitment: &SensitiveDeletionPersistedCommitment,
    request_id: &str,
    tenant_id: &str,
    retention_policy_id: &str,
    requirements: &[SensitiveDeletionRequirement],
) -> Result<(), SensitiveDeletionInventoryCommitmentError> {
    verify_sensitive_deletion_inventory_commitment(
        &commitment.commitment,
        request_id,
        tenant_id,
        retention_policy_id,
        requirements,
    )
}
