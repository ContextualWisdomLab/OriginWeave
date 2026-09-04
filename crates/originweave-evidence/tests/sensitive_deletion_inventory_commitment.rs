#![allow(clippy::expect_used)]

//! Credential-free commitment evidence for a fully verified sensitive-data deletion inventory.
//!
//! A successful receipt-set check must be able to return a bounded, order-independent commitment
//! to the exact declared-copy inventory and request scope without retaining target references in
//! the commitment itself. This lets a durable evidence owner bind later audit material to the
//! inventory that was actually checked without pretending that OriginWeave discovered every copy.

use originweave_evidence::{
    SensitiveDeletionCause, SensitiveDeletionReceipt, SensitiveDeletionReceiptInput,
    SensitiveDeletionReceiptSetError, SensitiveDeletionRequirement, SensitiveDeletionTarget,
    verify_sensitive_deletion_receipt_set_with_commitment,
};

fn requirement(
    target: SensitiveDeletionTarget,
    target_reference: &str,
    storage_scope_id: &str,
) -> SensitiveDeletionRequirement {
    SensitiveDeletionRequirement::new(target, target_reference, storage_scope_id)
        .expect("bounded exact-copy requirement should be valid")
}

fn receipt(
    target: SensitiveDeletionTarget,
    target_reference: &str,
    storage_scope_id: &str,
) -> SensitiveDeletionReceipt {
    receipt_for_scope(
        target,
        target_reference,
        storage_scope_id,
        "delete-request-001",
        "tenant-alpha",
        "retention-30d-v1",
    )
}

fn receipt_for_scope(
    target: SensitiveDeletionTarget,
    target_reference: &str,
    storage_scope_id: &str,
    request_id: &str,
    tenant_id: &str,
    retention_policy_id: &str,
) -> SensitiveDeletionReceipt {
    SensitiveDeletionReceipt::try_from(SensitiveDeletionReceiptInput {
        request_id: request_id.to_owned(),
        tenant_id: tenant_id.to_owned(),
        target_reference: target_reference.to_owned(),
        storage_scope_id: storage_scope_id.to_owned(),
        retention_policy_id: retention_policy_id.to_owned(),
        verification_reference: format!("proof-{target_reference}"),
        target,
        cause: SensitiveDeletionCause::TenantDeletion,
        deletion_epoch_seconds: 1_786_000_000,
        verification_epoch_seconds: 1_786_000_001,
    })
    .expect("bounded deletion receipt should be valid")
}

#[test]
fn successful_verification_emits_order_independent_inventory_commitment() {
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

    let first = verify_sensitive_deletion_receipt_set_with_commitment(
        &[authoritative_receipt.clone(), cache_receipt.clone()],
        "delete-request-001",
        "tenant-alpha",
        "retention-30d-v1",
        &[authoritative.clone(), cache.clone()],
    )
    .expect("complete declared inventory should produce commitment evidence");
    let reordered = verify_sensitive_deletion_receipt_set_with_commitment(
        &[cache_receipt, authoritative_receipt],
        "delete-request-001",
        "tenant-alpha",
        "retention-30d-v1",
        &[cache, authoritative],
    )
    .expect("receipt and requirement order should not alter the commitment");

    assert_eq!(first, reordered);
    assert_eq!(first.request_id(), "delete-request-001");
    assert_eq!(first.tenant_id(), "tenant-alpha");
    assert_eq!(first.retention_policy_id(), "retention-30d-v1");
    assert_eq!(first.declared_copy_count(), 2);
    assert_eq!(first.inventory_digest().len(), 64);
    assert!(
        first
            .inventory_digest()
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert!(!first.inventory_digest().contains("customer"));
    assert!(!first.inventory_digest().contains("primary-store"));
}

#[test]
fn commitment_changes_when_the_declared_inventory_changes() {
    let receipt_a = receipt(
        SensitiveDeletionTarget::CacheCopy,
        "record:customer:17:cache",
        "edge-cache",
    );
    let receipt_b = receipt(
        SensitiveDeletionTarget::CacheCopy,
        "record:customer:18:cache",
        "edge-cache",
    );
    let first = verify_sensitive_deletion_receipt_set_with_commitment(
        &[receipt_a],
        "delete-request-001",
        "tenant-alpha",
        "retention-30d-v1",
        &[requirement(
            SensitiveDeletionTarget::CacheCopy,
            "record:customer:17:cache",
            "edge-cache",
        )],
    )
    .expect("first complete inventory should verify");
    let second = verify_sensitive_deletion_receipt_set_with_commitment(
        &[receipt_b],
        "delete-request-001",
        "tenant-alpha",
        "retention-30d-v1",
        &[requirement(
            SensitiveDeletionTarget::CacheCopy,
            "record:customer:18:cache",
            "edge-cache",
        )],
    )
    .expect("second complete inventory should verify");

    assert_ne!(first.inventory_digest(), second.inventory_digest());
}

#[test]
fn commitment_digest_binds_request_tenant_and_retention_scope() {
    let requirement = requirement(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:17",
        "primary-store",
    );
    let scopes = [
        ("delete-request-001", "tenant-alpha", "retention-30d-v1"),
        ("delete-request-002", "tenant-alpha", "retention-30d-v1"),
        ("delete-request-001", "tenant-beta", "retention-30d-v1"),
        ("delete-request-001", "tenant-alpha", "retention-90d-v2"),
    ];
    let commitments = scopes.map(|(request_id, tenant_id, retention_policy_id)| {
        let scoped_receipt = receipt_for_scope(
            SensitiveDeletionTarget::AuthoritativeRecord,
            "record:customer:17",
            "primary-store",
            request_id,
            tenant_id,
            retention_policy_id,
        );
        verify_sensitive_deletion_receipt_set_with_commitment(
            &[scoped_receipt],
            request_id,
            tenant_id,
            retention_policy_id,
            std::slice::from_ref(&requirement),
        )
        .expect("matching bounded request scope should verify")
    });

    for left in 0..commitments.len() {
        for right in (left + 1)..commitments.len() {
            assert_ne!(
                commitments[left].inventory_digest(),
                commitments[right].inventory_digest(),
                "request, tenant, and retention scope must contribute to the digest"
            );
        }
    }
}

#[test]
fn commitment_encoding_covers_every_declared_copy_class() {
    let targets = [
        SensitiveDeletionTarget::AuthoritativeRecord,
        SensitiveDeletionTarget::DerivedArtifact,
        SensitiveDeletionTarget::ModelArtifact,
        SensitiveDeletionTarget::ExportArtifact,
        SensitiveDeletionTarget::CacheCopy,
        SensitiveDeletionTarget::SearchIndexEntry,
        SensitiveDeletionTarget::VectorIndexEntry,
        SensitiveDeletionTarget::TemporaryFile,
        SensitiveDeletionTarget::BackupCopy,
    ];
    let requirements = targets
        .iter()
        .copied()
        .enumerate()
        .map(|(index, target)| {
            requirement(
                target,
                &format!("record:customer:17:copy:{index}"),
                &format!("storage-scope-{index}"),
            )
        })
        .collect::<Vec<_>>();
    let receipts = targets
        .iter()
        .copied()
        .enumerate()
        .map(|(index, target)| {
            receipt(
                target,
                &format!("record:customer:17:copy:{index}"),
                &format!("storage-scope-{index}"),
            )
        })
        .collect::<Vec<_>>();

    let commitment = verify_sensitive_deletion_receipt_set_with_commitment(
        &receipts,
        "delete-request-001",
        "tenant-alpha",
        "retention-30d-v1",
        &requirements,
    )
    .expect("every supported declared-copy class should be commit-able");

    assert_eq!(commitment.declared_copy_count(), targets.len());
}

#[test]
fn invalid_receipt_sets_do_not_emit_commitment_evidence() {
    let requirements = [requirement(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:17",
        "primary-store",
    )];

    assert_eq!(
        verify_sensitive_deletion_receipt_set_with_commitment(
            &[],
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &requirements,
        ),
        Err(SensitiveDeletionReceiptSetError::MissingReceipt)
    );
}
