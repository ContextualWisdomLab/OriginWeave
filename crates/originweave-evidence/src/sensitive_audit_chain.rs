//! Deterministic credential-free linkage metadata for sensitive audit evidence.
//!
//! This module deliberately does not persist evidence. It gives a durable evidence owner a
//! canonical hash preimage plus exact tenant, stream, sequence, previous-digest continuity, and
//! deterministic SHA-256 verification before that owner signs, stores, or exports a record.

use std::fmt;

use sha2::{Digest, Sha256};

use super::{MAX_SENSITIVE_IDENTIFIER_BYTES, valid_sha256};

const HASH_PREIMAGE_DOMAIN: &[u8] = b"originweave-sensitive-audit-chain-v1\0";

/// Unvalidated metadata for one sensitive-audit chain link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveAuditChainLinkInput {
    /// Tenant that owns the sensitive-audit stream.
    pub tenant_id: String,
    /// Logical audit stream whose sequence must remain contiguous.
    pub audit_stream_id: String,
    /// One-based sequence number within the tenant-scoped audit stream.
    pub sequence_number: u64,
    /// Previous chain digest for non-genesis links, or `None` for sequence one.
    pub previous_chain_digest: Option<String>,
    /// Lowercase SHA-256 digest of the bounded evidence payload linked by this record.
    pub payload_digest: String,
    /// Lowercase SHA-256 digest recorded for this chain link by the durable evidence owner.
    pub chain_digest: String,
}

/// Unvalidated metadata for one separately trusted sensitive-audit checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveAuditChainCheckpointInput {
    /// Tenant whose audit history the checkpoint is intended to bind.
    pub tenant_id: String,
    /// Logical audit stream whose history the checkpoint is intended to bind.
    pub audit_stream_id: String,
    /// One-based sequence number of the checkpointed chain link.
    pub sequence_number: u64,
    /// Lowercase SHA-256 digest recorded for the checkpointed chain link.
    pub chain_digest: String,
}

/// Validation failure while constructing, verifying, or extending a sensitive-audit chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveAuditChainLinkError {
    /// A tenant or audit-stream identifier was empty, oversized, or ambiguous.
    InvalidIdentifier,
    /// A payload, previous-link, current-link, or checkpoint digest was not canonical SHA-256.
    InvalidDigest,
    /// The recorded current-link digest did not equal SHA-256 of its exact canonical preimage.
    ChainDigestMismatch,
    /// A sequence number was zero.
    InvalidSequence,
    /// A sequence-one genesis link carried a previous-chain digest.
    UnexpectedPreviousDigest,
    /// A non-genesis link omitted its previous-chain digest.
    MissingPreviousDigest,
    /// A complete-history verification request did not contain any links.
    EmptyHistory,
    /// A complete-history verification request did not begin with the genesis link.
    HistoryDoesNotStartAtGenesis,
    /// A next link changed tenant or logical audit stream.
    ChainContextMismatch,
    /// A next link did not use the exact contiguous sequence number.
    SequenceDiscontinuity,
    /// A next link did not point to the predecessor's recorded chain digest.
    PreviousDigestMismatch,
    /// The predecessor sequence cannot be incremented without overflowing `u64`.
    SequenceOverflow,
    /// A trusted checkpoint named a different tenant or logical audit stream.
    CheckpointContextMismatch,
    /// The checkpointed sequence is absent from the supplied verified history.
    CheckpointSequenceMissing,
    /// The supplied verified history disagrees with the trusted checkpoint digest.
    CheckpointDigestMismatch,
}

impl fmt::Display for SensitiveAuditChainLinkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentifier => "invalid sensitive audit chain identifier",
            Self::InvalidDigest => "invalid sensitive audit chain digest",
            Self::ChainDigestMismatch => {
                "sensitive audit chain digest does not match its canonical preimage"
            }
            Self::InvalidSequence => "invalid sensitive audit chain sequence",
            Self::UnexpectedPreviousDigest => {
                "genesis sensitive audit chain link has a previous digest"
            }
            Self::MissingPreviousDigest => {
                "non-genesis sensitive audit chain link is missing its previous digest"
            }
            Self::EmptyHistory => "sensitive audit chain history is empty",
            Self::HistoryDoesNotStartAtGenesis => {
                "sensitive audit chain history does not start at genesis"
            }
            Self::ChainContextMismatch => "sensitive audit chain context mismatch",
            Self::SequenceDiscontinuity => "sensitive audit chain sequence is not contiguous",
            Self::PreviousDigestMismatch => "sensitive audit chain previous digest mismatch",
            Self::SequenceOverflow => "sensitive audit chain sequence overflow",
            Self::CheckpointContextMismatch => "sensitive audit chain checkpoint context mismatch",
            Self::CheckpointSequenceMissing => {
                "sensitive audit chain checkpoint sequence is absent from history"
            }
            Self::CheckpointDigestMismatch => "sensitive audit chain checkpoint digest mismatch",
        })
    }
}

impl std::error::Error for SensitiveAuditChainLinkError {}

/// Immutable credential-free linkage metadata for one sensitive-audit record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveAuditChainLink {
    tenant_id: String,
    audit_stream_id: String,
    sequence_number: u64,
    previous_chain_digest: Option<String>,
    payload_digest: String,
    chain_digest: String,
}

impl TryFrom<SensitiveAuditChainLinkInput> for SensitiveAuditChainLink {
    type Error = SensitiveAuditChainLinkError;

    fn try_from(input: SensitiveAuditChainLinkInput) -> Result<Self, Self::Error> {
        if !valid_identifier(&input.tenant_id) || !valid_identifier(&input.audit_stream_id) {
            return Err(SensitiveAuditChainLinkError::InvalidIdentifier);
        }
        if input.sequence_number == 0 {
            return Err(SensitiveAuditChainLinkError::InvalidSequence);
        }
        match (
            input.sequence_number,
            input.previous_chain_digest.as_deref(),
        ) {
            (1, Some(_)) => return Err(SensitiveAuditChainLinkError::UnexpectedPreviousDigest),
            (1, None) => {}
            (_, None) => return Err(SensitiveAuditChainLinkError::MissingPreviousDigest),
            (_, Some(_)) => {}
        }
        if !valid_sha256(&input.payload_digest)
            || !valid_sha256(&input.chain_digest)
            || input
                .previous_chain_digest
                .as_deref()
                .is_some_and(|digest| !valid_sha256(digest))
        {
            return Err(SensitiveAuditChainLinkError::InvalidDigest);
        }
        Ok(Self {
            tenant_id: input.tenant_id,
            audit_stream_id: input.audit_stream_id,
            sequence_number: input.sequence_number,
            previous_chain_digest: input.previous_chain_digest,
            payload_digest: input.payload_digest,
            chain_digest: input.chain_digest,
        })
    }
}

impl SensitiveAuditChainLink {
    /// Return the tenant identifier bound to this audit stream.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Return the logical audit-stream identifier.
    #[must_use]
    pub fn audit_stream_id(&self) -> &str {
        &self.audit_stream_id
    }

    /// Return the one-based sequence number for this link.
    #[must_use]
    pub const fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the predecessor chain digest, or `None` for the genesis link.
    #[must_use]
    pub fn previous_chain_digest(&self) -> Option<&str> {
        self.previous_chain_digest.as_deref()
    }

    /// Return the lowercase SHA-256 digest of the linked evidence payload.
    #[must_use]
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }

    /// Return the lowercase SHA-256 digest recorded for this chain link.
    #[must_use]
    pub fn chain_digest(&self) -> &str {
        &self.chain_digest
    }

    /// Build the canonical domain-separated preimage hashed by the chain-link verifier or signer.
    ///
    /// The preimage includes tenant, stream, sequence, predecessor digest, and payload digest in a
    /// fixed order. Each field is decimal-length-delimited, and the current `chain_digest` is
    /// intentionally excluded because including it would make the digest definition recursive.
    #[must_use]
    pub fn canonical_hash_preimage(&self) -> Vec<u8> {
        let mut preimage = HASH_PREIMAGE_DOMAIN.to_vec();
        append_length_delimited(&mut preimage, self.tenant_id.as_bytes());
        append_length_delimited(&mut preimage, self.audit_stream_id.as_bytes());
        let sequence = self.sequence_number.to_string();
        append_length_delimited(&mut preimage, sequence.as_bytes());
        let previous_digest = match self.previous_chain_digest.as_deref() {
            Some(digest) => digest.as_bytes(),
            None => &[],
        };
        append_length_delimited(&mut preimage, previous_digest);
        append_length_delimited(&mut preimage, self.payload_digest.as_bytes());
        preimage
    }

    /// Compute the canonical lowercase SHA-256 identifier for this link's exact hash preimage.
    ///
    /// This deterministic digest does not authenticate a signer, make storage append-only, or
    /// prevent an authorized storage owner from rewriting and rehashing an entire history.
    #[must_use]
    pub fn computed_chain_digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_hash_preimage());
        format!("sha256:{digest:x}")
    }

    /// Verify that the recorded chain digest equals SHA-256 of the exact canonical preimage.
    ///
    /// Success proves only deterministic preimage-to-digest consistency. It does not authenticate
    /// the record, persist it, verify a signature, provide atomic append semantics, or prevent a
    /// storage authority from replacing and rehashing an entire history.
    pub fn verify_chain_digest(&self) -> Result<(), SensitiveAuditChainLinkError> {
        if self.computed_chain_digest() != self.chain_digest {
            return Err(SensitiveAuditChainLinkError::ChainDigestMismatch);
        }
        Ok(())
    }

    /// Validate and return the exact contiguous successor to this link.
    pub fn try_next(
        &self,
        input: SensitiveAuditChainLinkInput,
    ) -> Result<Self, SensitiveAuditChainLinkError> {
        let Some(expected_sequence) = self.sequence_number.checked_add(1) else {
            return Err(SensitiveAuditChainLinkError::SequenceOverflow);
        };
        let next = Self::try_from(input)?;
        if next.tenant_id != self.tenant_id || next.audit_stream_id != self.audit_stream_id {
            return Err(SensitiveAuditChainLinkError::ChainContextMismatch);
        }
        if next.sequence_number != expected_sequence {
            return Err(SensitiveAuditChainLinkError::SequenceDiscontinuity);
        }
        if next.previous_chain_digest.as_deref() != Some(self.chain_digest.as_str()) {
            return Err(SensitiveAuditChainLinkError::PreviousDigestMismatch);
        }
        Ok(next)
    }
}

/// Immutable credential-free metadata for a separately trusted chain checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveAuditChainCheckpoint {
    tenant_id: String,
    audit_stream_id: String,
    sequence_number: u64,
    chain_digest: String,
}

impl TryFrom<SensitiveAuditChainCheckpointInput> for SensitiveAuditChainCheckpoint {
    type Error = SensitiveAuditChainLinkError;

    fn try_from(input: SensitiveAuditChainCheckpointInput) -> Result<Self, Self::Error> {
        if !valid_identifier(&input.tenant_id) || !valid_identifier(&input.audit_stream_id) {
            return Err(SensitiveAuditChainLinkError::InvalidIdentifier);
        }
        if input.sequence_number == 0 {
            return Err(SensitiveAuditChainLinkError::InvalidSequence);
        }
        if !valid_sha256(&input.chain_digest) {
            return Err(SensitiveAuditChainLinkError::InvalidDigest);
        }
        Ok(Self {
            tenant_id: input.tenant_id,
            audit_stream_id: input.audit_stream_id,
            sequence_number: input.sequence_number,
            chain_digest: input.chain_digest,
        })
    }
}

impl SensitiveAuditChainCheckpoint {
    /// Return the tenant identifier named by this checkpoint.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Return the logical audit-stream identifier named by this checkpoint.
    #[must_use]
    pub fn audit_stream_id(&self) -> &str {
        &self.audit_stream_id
    }

    /// Return the one-based sequence number named by this checkpoint.
    #[must_use]
    pub const fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the lowercase SHA-256 chain digest named by this checkpoint.
    #[must_use]
    pub fn chain_digest(&self) -> &str {
        &self.chain_digest
    }
}

/// Verify one loaded sensitive-audit history from genesis through its final link.
///
/// Every recorded link digest is checked against its canonical preimage, and every successor is
/// revalidated against the exact predecessor tenant, stream, sequence, and digest. The history
/// must contain at least one link and begin at sequence one. Success does not authenticate a
/// signer, persist or atomically append records, or prevent an authorized storage owner from
/// replacing and rehashing an entire history.
pub fn verify_sensitive_audit_chain_history(
    history: &[SensitiveAuditChainLink],
) -> Result<(), SensitiveAuditChainLinkError> {
    let Some(first) = history.first() else {
        return Err(SensitiveAuditChainLinkError::EmptyHistory);
    };
    if first.sequence_number() != 1 {
        return Err(SensitiveAuditChainLinkError::HistoryDoesNotStartAtGenesis);
    }
    first.verify_chain_digest()?;

    for pair in history.windows(2) {
        let previous = &pair[0];
        let next = &pair[1];
        next.verify_chain_digest()?;
        previous.try_next(SensitiveAuditChainLinkInput {
            tenant_id: next.tenant_id().to_owned(),
            audit_stream_id: next.audit_stream_id().to_owned(),
            sequence_number: next.sequence_number(),
            previous_chain_digest: next.previous_chain_digest().map(str::to_owned),
            payload_digest: next.payload_digest().to_owned(),
            chain_digest: next.chain_digest().to_owned(),
        })?;
    }

    Ok(())
}

/// Verify a complete loaded history against one separately trusted prior checkpoint.
///
/// The supplied history is fully verified first. The checkpoint must then name the exact tenant,
/// logical audit stream, one-based sequence, and recorded digest of a link present in that history.
/// A checkpoint may refer to an earlier link while the verified history contains later appended
/// links. Success does not authenticate, sign, timestamp, publish, persist, or otherwise make the
/// checkpoint trustworthy; callers must obtain it from an independently trusted authority.
pub fn verify_sensitive_audit_chain_checkpoint(
    history: &[SensitiveAuditChainLink],
    checkpoint: &SensitiveAuditChainCheckpoint,
) -> Result<(), SensitiveAuditChainLinkError> {
    verify_sensitive_audit_chain_history(history)?;
    let first = &history[0];
    if first.tenant_id() != checkpoint.tenant_id()
        || first.audit_stream_id() != checkpoint.audit_stream_id()
    {
        return Err(SensitiveAuditChainLinkError::CheckpointContextMismatch);
    }
    let Some(anchored) = history
        .iter()
        .find(|link| link.sequence_number() == checkpoint.sequence_number())
    else {
        return Err(SensitiveAuditChainLinkError::CheckpointSequenceMissing);
    };
    if anchored.chain_digest() != checkpoint.chain_digest() {
        return Err(SensitiveAuditChainLinkError::CheckpointDigestMismatch);
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SENSITIVE_IDENTIFIER_BYTES
        && value.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn append_length_delimited(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(value.len().to_string().as_bytes());
    output.push(b':');
    output.extend_from_slice(value);
}
