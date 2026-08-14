#![allow(clippy::expect_used)]

//! Version contracts for persisted sensitive-data deletion inventory commitments.
//!
//! Durable evidence must carry the canonicalization version explicitly so a future wire-format
//! change cannot silently reinterpret a stored digest under different semantics.

use std::error::Error;

use originweave_evidence::{
    SensitiveDeletionCause, SensitiveDeletionInventoryCommitmentError, SensitiveDeletionReceipt,
    SensitiveDeletionReceiptInput, SensitiveDeletionReceiptSetCommitment,
    SensitiveDeletionReceiptSetCommitmentInput, SensitiveDeletionRequirement,
    SensitiveDeletionTarget, verify_sensitive_deletion_receipt_set_with_commitment,
};

const REQUEST_ID: &str = "delete-request-version-001";
const TENANT_ID: &str = "tenant-alpha";
const RETENTION_POLICY_ID: &str = "retention-30d-v1";
const COMMITMENT_VERSION_V1: u16 = 1;

fn emitted_commitment() -> SensitiveDeletionReceiptSetCommitment {
    let requirement = SensitiveDeletionRequirement::new(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:versioned",
        "primary-store",
    )
    .expect("bounded deletion requirement should be valid");
    let receipt = SensitiveDeletionReceipt::try_from(SensitiveDeletionReceiptInput {
        request_id: REQUEST_ID.to_owned(),
        tenant_id: TENANT_ID.to_owned(),
        target_reference: "record:customer:versioned".to_owned(),
        storage_scope_id: "primary-store".to_owned(),
        retention_policy_id: RETENTION_POLICY_ID.to_owned(),
        verification_reference: "proof-versioned".to_owned(),
        target: SensitiveDeletionTarget::AuthoritativeRecord,
        cause: SensitiveDeletionCause::TenantDeletion,
        deletion_epoch_seconds: 1_786_000_000,
        verification_epoch_seconds: 1_786_000_001,
    })
    .expect("bounded deletion receipt should be valid");

    verify_sensitive_deletion_receipt_set_with_commitment(
        &[receipt],
        REQUEST_ID,
        TENANT_ID,
        RETENTION_POLICY_ID,
        &[requirement],
    )
    .expect("complete receipt set should emit a deletion inventory commitment")
}

#[test]
fn emitted_commitment_exposes_the_persisted_wire_version() {
    let emitted = emitted_commitment();

    assert_eq!(emitted.commitment_version(), COMMITMENT_VERSION_V1);

    let reconstructed = SensitiveDeletionReceiptSetCommitment::try_from(
        SensitiveDeletionReceiptSetCommitmentInput {
            commitment_version: emitted.commitment_version(),
            request_id: emitted.request_id().to_owned(),
            tenant_id: emitted.tenant_id().to_owned(),
            retention_policy_id: emitted.retention_policy_id().to_owned(),
            declared_copy_count: emitted.declared_copy_count(),
            inventory_digest: emitted.inventory_digest().to_owned(),
        },
    )
    .expect("supported persisted commitment version should reconstruct");

    assert_eq!(reconstructed, emitted);
}

#[test]
fn reconstruction_rejects_unsupported_persisted_wire_versions() {
    for commitment_version in [0, COMMITMENT_VERSION_V1 + 1, u16::MAX] {
        let input = SensitiveDeletionReceiptSetCommitmentInput {
            commitment_version,
            request_id: REQUEST_ID.to_owned(),
            tenant_id: TENANT_ID.to_owned(),
            retention_policy_id: RETENTION_POLICY_ID.to_owned(),
            declared_copy_count: 1,
            inventory_digest: "0".repeat(64),
        };

        assert_eq!(
            SensitiveDeletionReceiptSetCommitment::try_from(input),
            Err(SensitiveDeletionInventoryCommitmentError::UnsupportedCommitmentVersion)
        );
    }
}

#[test]
fn unsupported_version_error_is_credential_safe_and_standard() {
    let error = SensitiveDeletionInventoryCommitmentError::UnsupportedCommitmentVersion;

    assert_eq!(
        error.to_string(),
        "unsupported sensitive deletion commitment version"
    );
    assert!(error.source().is_none());
}
