#![allow(clippy::expect_used)]

//! Resource bounds for exact sensitive-data deletion receipt-set verification.
//!
//! The verifier must reject oversized caller-supplied sets and scope identifiers before building
//! internal indexes or evaluating duplicate/unexpected entries. This keeps untrusted lifecycle
//! inventory size and comparison work from becoming unbounded CPU or memory work at the
//! deterministic evidence boundary.

use originweave_evidence::{
    MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES, MAX_SENSITIVE_IDENTIFIER_BYTES,
    SensitiveDeletionCause, SensitiveDeletionReceipt, SensitiveDeletionReceiptInput,
    SensitiveDeletionReceiptSetError, SensitiveDeletionRequirement, SensitiveDeletionTarget,
    verify_sensitive_deletion_receipt_set,
};

fn requirement(index: usize) -> SensitiveDeletionRequirement {
    SensitiveDeletionRequirement::new(
        SensitiveDeletionTarget::AuthoritativeRecord,
        &format!("record:customer:{index}"),
        "primary-store",
    )
    .expect("bounded exact-copy requirement should be valid")
}

fn receipt(index: usize) -> SensitiveDeletionReceipt {
    SensitiveDeletionReceipt::try_from(SensitiveDeletionReceiptInput {
        request_id: "delete-request-001".to_owned(),
        tenant_id: "tenant-alpha".to_owned(),
        target_reference: format!("record:customer:{index}"),
        storage_scope_id: "primary-store".to_owned(),
        retention_policy_id: "retention-30d-v1".to_owned(),
        verification_reference: format!("proof-primary-{index}"),
        target: SensitiveDeletionTarget::AuthoritativeRecord,
        cause: SensitiveDeletionCause::TenantDeletion,
        deletion_epoch_seconds: 1_786_000_000,
        verification_epoch_seconds: 1_786_000_001,
    })
    .expect("bounded deletion receipt should be valid")
}

#[test]
fn verifier_rejects_requirement_sets_over_the_resource_ceiling() {
    let requirements = (0..=MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES)
        .map(requirement)
        .collect::<Vec<_>>();

    assert_eq!(
        verify_sensitive_deletion_receipt_set(
            &[],
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &requirements,
        ),
        Err(SensitiveDeletionReceiptSetError::TooManyRequirements)
    );
}

#[test]
fn verifier_rejects_receipt_sets_over_the_resource_ceiling_before_entry_checks() {
    let requirements = [requirement(0)];
    let receipts = (0..=MAX_SENSITIVE_DELETION_RECEIPT_SET_ENTRIES)
        .map(receipt)
        .collect::<Vec<_>>();

    assert_eq!(
        verify_sensitive_deletion_receipt_set(
            &receipts,
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &requirements,
        ),
        Err(SensitiveDeletionReceiptSetError::TooManyReceipts)
    );
}

#[test]
fn verifier_rejects_oversized_scope_identifiers_before_entry_checks() {
    let requirements = [requirement(0)];
    let oversized = "a".repeat(MAX_SENSITIVE_IDENTIFIER_BYTES + 1);

    for (request_id, tenant_id, retention_policy_id) in [
        (oversized.as_str(), "tenant-alpha", "retention-30d-v1"),
        ("delete-request-001", oversized.as_str(), "retention-30d-v1"),
        ("delete-request-001", "tenant-alpha", oversized.as_str()),
    ] {
        assert_eq!(
            verify_sensitive_deletion_receipt_set(
                &[],
                request_id,
                tenant_id,
                retention_policy_id,
                &requirements,
            ),
            Err(SensitiveDeletionReceiptSetError::InvalidScopeIdentifier)
        );
    }
}
