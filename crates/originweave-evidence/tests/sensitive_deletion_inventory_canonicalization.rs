#![allow(clippy::expect_used)]

//! Stability contract for the deletion-inventory commitment wire canonicalization.
//!
//! Durable audit evidence must not change merely because the Rust enum declaration order changes.
//! Canonical inventory ordering is therefore defined by the stable encoded target token followed by
//! the already-bounded target reference and storage-scope identifier.

use originweave_evidence::{
    SensitiveDeletionCause, SensitiveDeletionReceipt, SensitiveDeletionReceiptInput,
    SensitiveDeletionRequirement, SensitiveDeletionTarget,
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
    SensitiveDeletionReceipt::try_from(SensitiveDeletionReceiptInput {
        request_id: "delete-request-001".to_owned(),
        tenant_id: "tenant-alpha".to_owned(),
        target_reference: target_reference.to_owned(),
        storage_scope_id: storage_scope_id.to_owned(),
        retention_policy_id: "retention-30d-v1".to_owned(),
        verification_reference: format!("proof-{target_reference}"),
        target,
        cause: SensitiveDeletionCause::TenantDeletion,
        deletion_epoch_seconds: 1_786_000_000,
        verification_epoch_seconds: 1_786_000_001,
    })
    .expect("bounded deletion receipt should be valid")
}

#[test]
fn commitment_orders_inventory_by_stable_encoded_target_token() {
    let derived = requirement(
        SensitiveDeletionTarget::DerivedArtifact,
        "record:customer:17:derived",
        "artifact-store",
    );
    let backup = requirement(
        SensitiveDeletionTarget::BackupCopy,
        "record:customer:17:backup",
        "backup-store",
    );
    let derived_receipt = receipt(
        SensitiveDeletionTarget::DerivedArtifact,
        "record:customer:17:derived",
        "artifact-store",
    );
    let backup_receipt = receipt(
        SensitiveDeletionTarget::BackupCopy,
        "record:customer:17:backup",
        "backup-store",
    );

    let commitment = verify_sensitive_deletion_receipt_set_with_commitment(
        &[derived_receipt, backup_receipt],
        "delete-request-001",
        "tenant-alpha",
        "retention-30d-v1",
        &[derived, backup],
    )
    .expect("complete declared inventory should produce commitment evidence");

    assert_eq!(
        commitment.inventory_digest(),
        "96626582e572f2b377e7a50f802cb97a9c4792e56af26906f5fee5505bf9cf45",
        "wire-stable commitment ordering must follow encoded target tokens rather than enum ordinals"
    );
}
