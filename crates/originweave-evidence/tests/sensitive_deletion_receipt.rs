#![allow(clippy::expect_used)]

//! Credential-free deletion-receipt contract for sensitive-data lifecycle evidence.
//!
//! A trusted storage owner may need to record that one exact declared copy was deleted or made
//! cryptographically unavailable. The receipt must identify only policy/evidence metadata and an
//! opaque target reference; it must never carry the deleted protected value itself.

use originweave_evidence::{
    MAX_SENSITIVE_IDENTIFIER_BYTES, SensitiveDeletionCause, SensitiveDeletionReceipt,
    SensitiveDeletionReceiptInput, SensitiveDeletionTarget, SensitiveEvidenceError,
};

fn valid_input() -> SensitiveDeletionReceiptInput {
    SensitiveDeletionReceiptInput {
        request_id: "delete-request-001".to_owned(),
        tenant_id: "tenant-alpha".to_owned(),
        target_reference: "record:customer:42".to_owned(),
        storage_scope_id: "primary-store".to_owned(),
        retention_policy_id: "retention-30d-v1".to_owned(),
        verification_reference: "delete-proof-001".to_owned(),
        target: SensitiveDeletionTarget::AuthoritativeRecord,
        cause: SensitiveDeletionCause::RetentionExpired,
        deletion_epoch_seconds: 1_786_000_000,
        verification_epoch_seconds: 1_786_000_001,
    }
}

#[test]
fn receipt_records_only_bounded_deletion_metadata() {
    let receipt = SensitiveDeletionReceipt::try_from(valid_input())
        .expect("bounded credential-free deletion metadata should be valid");

    assert_eq!(receipt.request_id(), "delete-request-001");
    assert_eq!(receipt.tenant_id(), "tenant-alpha");
    assert_eq!(receipt.target_reference(), "record:customer:42");
    assert_eq!(receipt.storage_scope_id(), "primary-store");
    assert_eq!(receipt.retention_policy_id(), "retention-30d-v1");
    assert_eq!(receipt.verification_reference(), "delete-proof-001");
    assert_eq!(receipt.target(), SensitiveDeletionTarget::AuthoritativeRecord);
    assert_eq!(receipt.cause(), SensitiveDeletionCause::RetentionExpired);
    assert_eq!(receipt.deletion_epoch_seconds(), 1_786_000_000);
    assert_eq!(receipt.verification_epoch_seconds(), 1_786_000_001);

    let debug = format!("{receipt:?}");
    assert!(!debug.contains("protected-value-sentinel"));
}

#[test]
fn receipt_supports_declared_copy_classes_and_deletion_causes() {
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
    let causes = [
        SensitiveDeletionCause::RetentionExpired,
        SensitiveDeletionCause::TenantDeletion,
        SensitiveDeletionCause::DataSubjectRequest,
        SensitiveDeletionCause::KeyRevocation,
        SensitiveDeletionCause::PolicyChange,
    ];

    for target in targets {
        for cause in causes {
            let mut input = valid_input();
            input.target = target;
            input.cause = cause;
            let receipt = SensitiveDeletionReceipt::try_from(input)
                .expect("typed target and cause should remain valid metadata");
            assert_eq!(receipt.target(), target);
            assert_eq!(receipt.cause(), cause);
        }
    }
}

#[test]
fn receipt_rejects_malformed_or_oversized_identifiers() {
    for field in [
        "request_id",
        "tenant_id",
        "target_reference",
        "storage_scope_id",
        "retention_policy_id",
        "verification_reference",
    ] {
        for invalid in [
            String::new(),
            "---".to_owned(),
            "contains space".to_owned(),
            "x".repeat(MAX_SENSITIVE_IDENTIFIER_BYTES + 1),
        ] {
            let mut input = valid_input();
            match field {
                "request_id" => input.request_id = invalid,
                "tenant_id" => input.tenant_id = invalid,
                "target_reference" => input.target_reference = invalid,
                "storage_scope_id" => input.storage_scope_id = invalid,
                "retention_policy_id" => input.retention_policy_id = invalid,
                "verification_reference" => input.verification_reference = invalid,
                _ => unreachable!(),
            }
            assert_eq!(
                SensitiveDeletionReceipt::try_from(input),
                Err(SensitiveEvidenceError::InvalidIdentifier),
                "{field} must fail closed"
            );
        }
    }

    let mut boundary = valid_input();
    boundary.target_reference = "x".repeat(MAX_SENSITIVE_IDENTIFIER_BYTES);
    assert!(SensitiveDeletionReceipt::try_from(boundary).is_ok());
}

#[test]
fn receipt_requires_positive_ordered_deletion_and_verification_times() {
    for (deletion, verification) in [(0, 1), (1, 0), (2, 1)] {
        let mut input = valid_input();
        input.deletion_epoch_seconds = deletion;
        input.verification_epoch_seconds = verification;
        assert_eq!(
            SensitiveDeletionReceipt::try_from(input),
            Err(SensitiveEvidenceError::InvalidLifecycle)
        );
    }

    let mut same_instant = valid_input();
    same_instant.verification_epoch_seconds = same_instant.deletion_epoch_seconds;
    assert!(SensitiveDeletionReceipt::try_from(same_instant).is_ok());
}
