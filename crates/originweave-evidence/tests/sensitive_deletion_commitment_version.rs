#![allow(clippy::expect_used)]

//! Version contracts for persisted sensitive-data deletion inventory commitments.
//!
//! Durable evidence must carry the canonicalization version explicitly so a future wire-format
//! change cannot silently reinterpret a stored digest under different semantics. The versioned
//! persistence envelope is intentionally separate from the existing in-process commitment input so
//! adding durable versioning does not break callers that already construct that bounded value.

use std::error::Error;

use originweave_evidence::{
    SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION, SensitiveDeletionCause,
    SensitiveDeletionPersistedCommitment, SensitiveDeletionPersistedCommitmentError,
    SensitiveDeletionPersistedCommitmentInput, SensitiveDeletionReceipt,
    SensitiveDeletionReceiptInput, SensitiveDeletionRequirement, SensitiveDeletionTarget,
    verify_persisted_sensitive_deletion_inventory_commitment,
    verify_sensitive_deletion_receipt_set_with_persisted_commitment,
};

const REQUEST_ID: &str = "delete-request-version-001";
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

fn emitted_commitment() -> (
    SensitiveDeletionPersistedCommitment,
    Vec<SensitiveDeletionRequirement>,
) {
    let authoritative = requirement(
        SensitiveDeletionTarget::AuthoritativeRecord,
        "record:customer:versioned",
        "primary-store",
    );
    let cache = requirement(
        SensitiveDeletionTarget::CacheCopy,
        "record:customer:versioned:cache",
        "edge-cache",
    );
    let receipts = [
        receipt(
            SensitiveDeletionTarget::AuthoritativeRecord,
            "record:customer:versioned",
            "primary-store",
        ),
        receipt(
            SensitiveDeletionTarget::CacheCopy,
            "record:customer:versioned:cache",
            "edge-cache",
        ),
    ];
    let requirements = vec![authoritative, cache];
    let commitment = verify_sensitive_deletion_receipt_set_with_persisted_commitment(
        &receipts,
        REQUEST_ID,
        TENANT_ID,
        RETENTION_POLICY_ID,
        &requirements,
    )
    .expect("complete receipt set should emit a versioned persisted commitment");

    (commitment, requirements)
}

#[test]
fn emitted_commitment_exposes_and_round_trips_the_persisted_wire_version() {
    let (emitted, requirements) = emitted_commitment();

    assert_eq!(
        emitted.commitment_version(),
        SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION
    );

    let reconstructed =
        SensitiveDeletionPersistedCommitment::try_from(SensitiveDeletionPersistedCommitmentInput {
            commitment_version: emitted.commitment_version(),
            request_id: emitted.request_id().to_owned(),
            tenant_id: emitted.tenant_id().to_owned(),
            retention_policy_id: emitted.retention_policy_id().to_owned(),
            declared_copy_count: emitted.declared_copy_count(),
            inventory_digest: emitted.inventory_digest().to_owned(),
        })
        .expect("supported persisted commitment version should reconstruct");

    assert_eq!(reconstructed, emitted);
    assert_eq!(
        verify_persisted_sensitive_deletion_inventory_commitment(
            &reconstructed,
            REQUEST_ID,
            TENANT_ID,
            RETENTION_POLICY_ID,
            &[requirements[1].clone(), requirements[0].clone()],
        ),
        Ok(())
    );
}

#[test]
fn reconstruction_rejects_unsupported_versions_before_other_metadata() {
    for commitment_version in [
        0,
        SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION + 1,
        u16::MAX,
    ] {
        let input = SensitiveDeletionPersistedCommitmentInput {
            commitment_version,
            request_id: "bad request".to_owned(),
            tenant_id: TENANT_ID.to_owned(),
            retention_policy_id: RETENTION_POLICY_ID.to_owned(),
            declared_copy_count: 0,
            inventory_digest: "not-a-digest".to_owned(),
        };

        assert_eq!(
            SensitiveDeletionPersistedCommitment::try_from(input),
            Err(SensitiveDeletionPersistedCommitmentError::UnsupportedCommitmentVersion)
        );
    }
}

#[test]
fn unsupported_version_error_is_credential_safe_and_standard() {
    let error = SensitiveDeletionPersistedCommitmentError::UnsupportedCommitmentVersion;

    assert_eq!(
        error.to_string(),
        "unsupported persisted sensitive deletion commitment version"
    );
    assert!(error.source().is_none());
}
