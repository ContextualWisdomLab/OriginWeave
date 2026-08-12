//! Deterministic credential-free linkage metadata for sensitive audit evidence.
//!
//! This module deliberately does not persist evidence or compute a cryptographic digest. It gives
//! a durable evidence owner a canonical hash preimage plus exact tenant, stream, sequence, and
//! previous-digest continuity checks before that owner hashes, signs, stores, or exports a record.

use std::fmt;

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

/// Validation failure while constructing or extending a sensitive-audit chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveAuditChainLinkError {
    /// A tenant or audit-stream identifier was empty, oversized, or ambiguous.
    InvalidIdentifier,
    /// A payload, previous-link, or current-link digest was not canonical lowercase SHA-256.
    InvalidDigest,
    /// A sequence number was zero.
    InvalidSequence,
    /// A sequence-one genesis link carried a previous-chain digest.
    UnexpectedPreviousDigest,
    /// A non-genesis link omitted its previous-chain digest.
    MissingPreviousDigest,
    /// A next link changed tenant or logical audit stream.
    ChainContextMismatch,
    /// A next link did not use the exact contiguous sequence number.
    SequenceDiscontinuity,
    /// A next link did not point to the predecessor's recorded chain digest.
    PreviousDigestMismatch,
    /// The predecessor sequence cannot be incremented without overflowing `u64`.
    SequenceOverflow,
}

impl fmt::Display for SensitiveAuditChainLinkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentifier => "invalid sensitive audit chain identifier",
            Self::InvalidDigest => "invalid sensitive audit chain digest",
            Self::InvalidSequence => "invalid sensitive audit chain sequence",
            Self::UnexpectedPreviousDigest => {
                "genesis sensitive audit chain link has a previous digest"
            }
            Self::MissingPreviousDigest => {
                "non-genesis sensitive audit chain link is missing its previous digest"
            }
            Self::ChainContextMismatch => "sensitive audit chain context mismatch",
            Self::SequenceDiscontinuity => "sensitive audit chain sequence is not contiguous",
            Self::PreviousDigestMismatch => "sensitive audit chain previous digest mismatch",
            Self::SequenceOverflow => "sensitive audit chain sequence overflow",
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
        match (input.sequence_number, input.previous_chain_digest.as_deref()) {
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

    /// Build the canonical domain-separated preimage that an external hash/signature owner hashes.
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
        append_length_delimited(
            &mut preimage,
            self.previous_chain_digest.as_deref().unwrap_or_default().as_bytes(),
        );
        append_length_delimited(&mut preimage, self.payload_digest.as_bytes());
        preimage
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
