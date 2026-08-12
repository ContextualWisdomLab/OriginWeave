#![allow(clippy::expect_used)]

//! Exact declared-copy completeness contract for sensitive-data deletion receipts.
//!
//! A single credential-free deletion receipt proves only one declared copy. Completion of a
//! tenant/request lifecycle step must not be inferred until every exact copy requirement is
//! represented once and only once under the same request, tenant, and retention policy.

use originweave_evidence::{
    SensitiveDeletionCause, SensitiveDeletionReceipt, SensitiveDeletionReceiptInput,
    SensitiveDeletionReceiptSetError, SensitiveDeletionRequirement, SensitiveDeletionTarget,
    SensitiveEvidenceError, verify_sensitive_deletion_receipt_set,
};

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
        verification_reference: format!("proof-{storage_scope_id}"),
        target,
        cause: SensitiveDeletionCause::TenantDeletion,
        deletion_epoch_seconds: 1_786_000_000,
        verification_epoch_seconds: 1_786_000_001,
    })
    .expect("bounded deletion receipt should be valid")
}

fn requirement(
    target: SensitiveDeletionTarget,
    target_reference: &str,
    storage_scope_id: &str,
) -> SensitiveDeletionRequirement {
    SensitiveDeletionRequirement::new(target, target_reference, storage_scope_id)
        .expect("bounded exact-copy requirement should be valid")
}

#[test]
fn exact_declared_copy_set_is_accepted_once_every_requirement_has_one_receipt() {
    let requirements = [
        requirement(
            SensitiveDeletionTarget::AuthoritativeRecord,
            "record:customer:42",
            "primary-store",
        ),
        requirement(
            SensitiveDeletionTarget::SearchIndexEntry,
            "search:customer:42",
            "customer-search",
        ),
        requirement(
            SensitiveDeletionTarget::BackupCopy,
            "backup:customer:42:2026-08-12",
            "backup-vault-a",
        ),
    ];
    let receipts = [
        receipt(
            SensitiveDeletionTarget::BackupCopy,
            "backup:customer:42:2026-08-12",
            "backup-vault-a",
        ),
        receipt(
            SensitiveDeletionTarget::AuthoritativeRecord,
            "record:customer:42",
            "primary-store",
        ),
        receipt(
            SensitiveDeletionTarget::SearchIndexEntry,
            "search:customer:42",
            "customer-search",
        ),
    ];

    assert_eq!(
        verify_sensitive_deletion_receipt_set(
            &receipts,
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &requirements,
        ),
        Ok(())
    );
}

#[test]
fn verifier_rejects_scope_mismatch_before_claiming_set_completeness() {
    let requirements = [requirement(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:42",
        "primary-store",
    )];

    for (field, expected_error) in [
        ("request", SensitiveDeletionReceiptSetError::RequestMismatch),
        ("tenant", SensitiveDeletionReceiptSetError::TenantMismatch),
        (
            "retention",
            SensitiveDeletionReceiptSetError::RetentionPolicyMismatch,
        ),
    ] {
        let mut input = SensitiveDeletionReceiptInput {
            request_id: "delete-request-001".to_owned(),
            tenant_id: "tenant-alpha".to_owned(),
            target_reference: "record:customer:42".to_owned(),
            storage_scope_id: "primary-store".to_owned(),
            retention_policy_id: "retention-30d-v1".to_owned(),
            verification_reference: "proof-primary".to_owned(),
            target: SensitiveDeletionTarget::AuthoritativeRecord,
            cause: SensitiveDeletionCause::TenantDeletion,
            deletion_epoch_seconds: 1_786_000_000,
            verification_epoch_seconds: 1_786_000_001,
        };
        match field {
            "request" => input.request_id = "delete-request-other".to_owned(),
            "tenant" => input.tenant_id = "tenant-beta".to_owned(),
            "retention" => input.retention_policy_id = "retention-90d-v2".to_owned(),
            _ => unreachable!(),
        }
        let mismatched = SensitiveDeletionReceipt::try_from(input)
            .expect("mismatched but bounded receipt should remain individually valid");
        assert_eq!(
            verify_sensitive_deletion_receipt_set(
                &[mismatched],
                "delete-request-001",
                "tenant-alpha",
                "retention-30d-v1",
                &requirements,
            ),
            Err(expected_error),
            "{field} mismatch must fail closed",
        );
    }
}

#[test]
fn verifier_rejects_missing_unexpected_and_duplicate_exact_copies() {
    let primary = requirement(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:42",
        "primary-store",
    );
    let cache = requirement(
        SensitiveDeletionTarget::CacheCopy,
        "cache:customer:42",
        "profile-cache",
    );
    let requirements = [primary.clone(), cache];
    let primary_receipt = receipt(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:42",
        "primary-store",
    );

    assert_eq!(
        verify_sensitive_deletion_receipt_set(
            std::slice::from_ref(&primary_receipt),
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &requirements,
        ),
        Err(SensitiveDeletionReceiptSetError::MissingReceipt)
    );

    let unexpected = receipt(
        SensitiveDeletionTarget::VectorIndexEntry,
        "vector:customer:42",
        "embedding-index",
    );
    assert_eq!(
        verify_sensitive_deletion_receipt_set(
            &[primary_receipt.clone(), unexpected],
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &requirements,
        ),
        Err(SensitiveDeletionReceiptSetError::UnexpectedReceipt)
    );

    assert_eq!(
        verify_sensitive_deletion_receipt_set(
            &[primary_receipt.clone(), primary_receipt],
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &[primary],
        ),
        Err(SensitiveDeletionReceiptSetError::DuplicateReceipt)
    );
}

#[test]
fn verifier_rejects_empty_or_duplicate_requirements() {
    let primary = requirement(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:42",
        "primary-store",
    );
    let primary_receipt = receipt(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:42",
        "primary-store",
    );

    assert_eq!(
        verify_sensitive_deletion_receipt_set(
            std::slice::from_ref(&primary_receipt),
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &[],
        ),
        Err(SensitiveDeletionReceiptSetError::EmptyRequirementSet)
    );
    assert_eq!(
        verify_sensitive_deletion_receipt_set(
            &[primary_receipt],
            "delete-request-001",
            "tenant-alpha",
            "retention-30d-v1",
            &[primary.clone(), primary],
        ),
        Err(SensitiveDeletionReceiptSetError::DuplicateRequirement)
    );
}

#[test]
fn exact_copy_requirement_reuses_bounded_identifier_validation() {
    assert_eq!(
        SensitiveDeletionRequirement::new(
            SensitiveDeletionTarget::TemporaryFile,
            "contains space",
            "scratch-store",
        ),
        Err(SensitiveEvidenceError::InvalidIdentifier)
    );
    assert_eq!(
        SensitiveDeletionRequirement::new(
            SensitiveDeletionTarget::TemporaryFile,
            "temp:customer:42",
            "---",
        ),
        Err(SensitiveEvidenceError::InvalidIdentifier)
    );
}
