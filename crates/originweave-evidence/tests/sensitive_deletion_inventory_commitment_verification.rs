#![allow(clippy::expect_used)]

//! Verification contracts for persisted sensitive-data deletion inventory commitments.
//!
//! A durable evidence owner must be able to reconstruct a bounded commitment from credential-free
//! persisted metadata and verify it against the exact expected request scope and declared-copy
//! inventory. Deterministic verification does not authenticate the persistence owner or prove that
//! the caller's declared inventory enumerates every real copy.

use std::error::Error;

use originweave_evidence::{
    MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES, SensitiveDeletionCause,
    SensitiveDeletionInventoryCommitmentError, SensitiveDeletionReceipt,
    SensitiveDeletionReceiptInput, SensitiveDeletionReceiptSetCommitment,
    SensitiveDeletionReceiptSetCommitmentInput, SensitiveDeletionRequirement,
    SensitiveDeletionTarget, verify_sensitive_deletion_inventory_commitment,
    verify_sensitive_deletion_receipt_set_with_commitment,
};

const REQUEST_ID: &str = "delete-request-001";
const TENANT_ID: &str = "tenant-alpha";
const RETENTION_POLICY_ID: &str = "retention-30d-v1";

fn requirement(
    target: SensitiveDeletionTarget,
    target_reference: &str,
    storage_scope_id: &str,
) -> SensitiveDeletionRequirement {
    SensitiveDeletionRequirement::new(target, target_reference, storage_scope_id)
        .expect("bounded deletion requirement should be valid")
}

fn receipt(
    target: SensitiveDeletionTarget,
    target_reference: &str,
    storage_scope_id: &str,
) -> SensitiveDeletionReceipt {
    SensitiveDeletionReceipt::try_from(SensitiveDeletionReceiptInput {
        request_id: REQUEST_ID.to_owned(),
        tenant_id: TENANT_ID.to_owned(),
        target_reference: target_reference.to_owned(),
        storage_scope_id: storage_scope_id.to_owned(),
        retention_policy_id: RETENTION_POLICY_ID.to_owned(),
        verification_reference: format!("proof-{target_reference}"),
        target,
        cause: SensitiveDeletionCause::TenantDeletion,
        deletion_epoch_seconds: 1_786_000_000,
        verification_epoch_seconds: 1_786_000_001,
    })
    .expect("bounded deletion receipt should be valid")
}

fn emitted_commitment(
    requirements: &[SensitiveDeletionRequirement],
    receipts: &[SensitiveDeletionReceipt],
) -> SensitiveDeletionReceiptSetCommitment {
    verify_sensitive_deletion_receipt_set_with_commitment(
        receipts,
        REQUEST_ID,
        TENANT_ID,
        RETENTION_POLICY_ID,
        requirements,
    )
    .expect("complete receipt set should emit a deletion inventory commitment")
}

fn reconstructed_commitment(
    commitment: &SensitiveDeletionReceiptSetCommitment,
) -> SensitiveDeletionReceiptSetCommitment {
    SensitiveDeletionReceiptSetCommitment::try_from(SensitiveDeletionReceiptSetCommitmentInput {
        request_id: commitment.request_id().to_owned(),
        tenant_id: commitment.tenant_id().to_owned(),
        retention_policy_id: commitment.retention_policy_id().to_owned(),
        declared_copy_count: commitment.declared_copy_count(),
        inventory_digest: commitment.inventory_digest().to_owned(),
    })
    .expect("emitted bounded commitment should reconstruct")
}

fn one_copy() -> (SensitiveDeletionRequirement, SensitiveDeletionReceipt) {
    (
        requirement(
            SensitiveDeletionTarget::AuthoritativeRecord,
            "record:customer:17",
            "primary-store",
        ),
        receipt(
            SensitiveDeletionTarget::AuthoritativeRecord,
            "record:customer:17",
            "primary-store",
        ),
    )
}

#[test]
fn persisted_commitment_reconstructs_and_verifies_order_independently() {
    let authoritative = requirement(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:17",
        "primary-store",
    );
    let cache = requirement(
        SensitiveDeletionTarget::CacheCopy,
        "record:customer:17:cache",
        "edge-cache",
    );
    let authoritative_receipt = receipt(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:17",
        "primary-store",
    );
    let cache_receipt = receipt(
        SensitiveDeletionTarget::CacheCopy,
        "record:customer:17:cache",
        "edge-cache",
    );
    let emitted = emitted_commitment(
        &[authoritative.clone(), cache.clone()],
        &[authoritative_receipt, cache_receipt],
    );
    let loaded = reconstructed_commitment(&emitted);

    assert_eq!(loaded, emitted);
    assert_eq!(
        verify_sensitive_deletion_inventory_commitment(
            &loaded,
            REQUEST_ID,
            TENANT_ID,
            RETENTION_POLICY_ID,
            &[cache, authoritative],
        ),
        Ok(())
    );
}

#[test]
fn reconstruction_rejects_malformed_scope_count_and_digest() {
    let valid_digest = "a".repeat(64);
    let cases = [
        (
            SensitiveDeletionReceiptSetCommitmentInput {
                request_id: "bad request".to_owned(),
                tenant_id: TENANT_ID.to_owned(),
                retention_policy_id: RETENTION_POLICY_ID.to_owned(),
                declared_copy_count: 1,
                inventory_digest: valid_digest.clone(),
            },
            SensitiveDeletionInventoryCommitmentError::InvalidScopeIdentifier,
        ),
        (
            SensitiveDeletionReceiptSetCommitmentInput {
                request_id: REQUEST_ID.to_owned(),
                tenant_id: "bad tenant".to_owned(),
                retention_policy_id: RETENTION_POLICY_ID.to_owned(),
                declared_copy_count: 1,
                inventory_digest: valid_digest.clone(),
            },
            SensitiveDeletionInventoryCommitmentError::InvalidScopeIdentifier,
        ),
        (
            SensitiveDeletionReceiptSetCommitmentInput {
                request_id: REQUEST_ID.to_owned(),
                tenant_id: TENANT_ID.to_owned(),
                retention_policy_id: "bad retention".to_owned(),
                declared_copy_count: 1,
                inventory_digest: valid_digest.clone(),
            },
            SensitiveDeletionInventoryCommitmentError::InvalidScopeIdentifier,
        ),
        (
            SensitiveDeletionReceiptSetCommitmentInput {
                request_id: REQUEST_ID.to_owned(),
                tenant_id: TENANT_ID.to_owned(),
                retention_policy_id: RETENTION_POLICY_ID.to_owned(),
                declared_copy_count: 0,
                inventory_digest: valid_digest.clone(),
            },
            SensitiveDeletionInventoryCommitmentError::InvalidDeclaredCopyCount,
        ),
        (
            SensitiveDeletionReceiptSetCommitmentInput {
                request_id: REQUEST_ID.to_owned(),
                tenant_id: TENANT_ID.to_owned(),
                retention_policy_id: RETENTION_POLICY_ID.to_owned(),
                declared_copy_count: MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES + 1,
                inventory_digest: valid_digest.clone(),
            },
            SensitiveDeletionInventoryCommitmentError::InvalidDeclaredCopyCount,
        ),
        (
            SensitiveDeletionReceiptSetCommitmentInput {
                request_id: REQUEST_ID.to_owned(),
                tenant_id: TENANT_ID.to_owned(),
                retention_policy_id: RETENTION_POLICY_ID.to_owned(),
                declared_copy_count: 1,
                inventory_digest: "a".repeat(63),
            },
            SensitiveDeletionInventoryCommitmentError::InvalidInventoryDigest,
        ),
        (
            SensitiveDeletionReceiptSetCommitmentInput {
                request_id: REQUEST_ID.to_owned(),
                tenant_id: TENANT_ID.to_owned(),
                retention_policy_id: RETENTION_POLICY_ID.to_owned(),
                declared_copy_count: 1,
                inventory_digest: "A".repeat(64),
            },
            SensitiveDeletionInventoryCommitmentError::InvalidInventoryDigest,
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(
            SensitiveDeletionReceiptSetCommitment::try_from(input),
            Err(expected)
        );
    }
}

#[test]
fn verifier_rejects_invalid_expected_scope_before_inventory_work() {
    let (requirement, receipt) = one_copy();
    let commitment = emitted_commitment(std::slice::from_ref(&requirement), &[receipt]);
    let cases = [
        ("bad request", TENANT_ID, RETENTION_POLICY_ID),
        (REQUEST_ID, "bad tenant", RETENTION_POLICY_ID),
        (REQUEST_ID, TENANT_ID, "bad retention"),
    ];

    for (request_id, tenant_id, retention_policy_id) in cases {
        assert_eq!(
            verify_sensitive_deletion_inventory_commitment(
                &commitment,
                request_id,
                tenant_id,
                retention_policy_id,
                std::slice::from_ref(&requirement),
            ),
            Err(SensitiveDeletionInventoryCommitmentError::InvalidScopeIdentifier)
        );
    }
}

#[test]
fn verifier_binds_exact_expected_request_tenant_and_retention_scope() {
    let (requirement, receipt) = one_copy();
    let commitment = emitted_commitment(std::slice::from_ref(&requirement), &[receipt]);
    let cases = [
        (
            "delete-request-002",
            TENANT_ID,
            RETENTION_POLICY_ID,
            SensitiveDeletionInventoryCommitmentError::RequestMismatch,
        ),
        (
            REQUEST_ID,
            "tenant-beta",
            RETENTION_POLICY_ID,
            SensitiveDeletionInventoryCommitmentError::TenantMismatch,
        ),
        (
            REQUEST_ID,
            TENANT_ID,
            "retention-90d-v2",
            SensitiveDeletionInventoryCommitmentError::RetentionPolicyMismatch,
        ),
    ];

    for (request_id, tenant_id, retention_policy_id, expected) in cases {
        assert_eq!(
            verify_sensitive_deletion_inventory_commitment(
                &commitment,
                request_id,
                tenant_id,
                retention_policy_id,
                std::slice::from_ref(&requirement),
            ),
            Err(expected)
        );
    }
}

#[test]
fn verifier_rejects_empty_duplicate_and_oversized_declared_inventory() {
    let (declared_requirement, receipt) = one_copy();
    let commitment = emitted_commitment(std::slice::from_ref(&declared_requirement), &[receipt]);

    assert_eq!(
        verify_sensitive_deletion_inventory_commitment(
            &commitment,
            REQUEST_ID,
            TENANT_ID,
            RETENTION_POLICY_ID,
            &[],
        ),
        Err(SensitiveDeletionInventoryCommitmentError::EmptyRequirementSet)
    );
    assert_eq!(
        verify_sensitive_deletion_inventory_commitment(
            &commitment,
            REQUEST_ID,
            TENANT_ID,
            RETENTION_POLICY_ID,
            &[declared_requirement.clone(), declared_requirement.clone()],
        ),
        Err(SensitiveDeletionInventoryCommitmentError::DuplicateRequirement)
    );

    let oversized = (0..=MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES)
        .map(|index| {
            requirement(
                SensitiveDeletionTarget::CacheCopy,
                &format!("record:customer:{index}:cache"),
                "edge-cache",
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        verify_sensitive_deletion_inventory_commitment(
            &commitment,
            REQUEST_ID,
            TENANT_ID,
            RETENTION_POLICY_ID,
            &oversized,
        ),
        Err(SensitiveDeletionInventoryCommitmentError::TooManyRequirements)
    );
}

#[test]
fn verifier_rejects_declared_count_and_inventory_digest_mismatch() {
    let (requirement, receipt) = one_copy();
    let emitted = emitted_commitment(std::slice::from_ref(&requirement), &[receipt]);
    let wrong_count = SensitiveDeletionReceiptSetCommitment::try_from(
        SensitiveDeletionReceiptSetCommitmentInput {
            request_id: REQUEST_ID.to_owned(),
            tenant_id: TENANT_ID.to_owned(),
            retention_policy_id: RETENTION_POLICY_ID.to_owned(),
            declared_copy_count: 2,
            inventory_digest: emitted.inventory_digest().to_owned(),
        },
    )
    .expect("bounded mismatched count should still be structurally valid");
    assert_eq!(
        verify_sensitive_deletion_inventory_commitment(
            &wrong_count,
            REQUEST_ID,
            TENANT_ID,
            RETENTION_POLICY_ID,
            std::slice::from_ref(&requirement),
        ),
        Err(SensitiveDeletionInventoryCommitmentError::DeclaredCopyCountMismatch)
    );

    let wrong_digest = SensitiveDeletionReceiptSetCommitment::try_from(
        SensitiveDeletionReceiptSetCommitmentInput {
            request_id: REQUEST_ID.to_owned(),
            tenant_id: TENANT_ID.to_owned(),
            retention_policy_id: RETENTION_POLICY_ID.to_owned(),
            declared_copy_count: 1,
            inventory_digest: "0".repeat(64),
        },
    )
    .expect("canonical but incorrect digest should reconstruct");
    assert_eq!(
        verify_sensitive_deletion_inventory_commitment(
            &wrong_digest,
            REQUEST_ID,
            TENANT_ID,
            RETENTION_POLICY_ID,
            &[requirement],
        ),
        Err(SensitiveDeletionInventoryCommitmentError::InventoryDigestMismatch)
    );
}

#[test]
fn public_commitment_errors_are_credential_safe_standard_errors() {
    let cases = [
        (
            SensitiveDeletionInventoryCommitmentError::InvalidScopeIdentifier,
            "invalid sensitive deletion commitment scope identifier",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::InvalidDeclaredCopyCount,
            "invalid sensitive deletion commitment declared copy count",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::InvalidInventoryDigest,
            "invalid sensitive deletion commitment inventory digest",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::TooManyRequirements,
            "too many sensitive deletion commitment requirements",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::EmptyRequirementSet,
            "empty sensitive deletion commitment requirement set",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::DuplicateRequirement,
            "duplicate sensitive deletion commitment requirement",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::RequestMismatch,
            "sensitive deletion commitment request mismatch",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::TenantMismatch,
            "sensitive deletion commitment tenant mismatch",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::RetentionPolicyMismatch,
            "sensitive deletion commitment retention policy mismatch",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::DeclaredCopyCountMismatch,
            "sensitive deletion commitment declared copy count mismatch",
        ),
        (
            SensitiveDeletionInventoryCommitmentError::InventoryDigestMismatch,
            "sensitive deletion commitment inventory digest mismatch",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        assert!(error.source().is_none());
    }
}
