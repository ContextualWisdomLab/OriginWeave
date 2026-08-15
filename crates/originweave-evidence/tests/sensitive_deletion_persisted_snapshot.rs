#![allow(clippy::expect_used)]

//! Canonical export contract for persisted sensitive-data deletion commitments.
//!
//! A durable evidence owner must not have to duplicate field-copying or count-width conversion when
//! serializing a validated commitment. The production authority therefore owns conversion back to
//! the exact versioned wire input that it accepts during reconstruction.

use originweave_evidence::{
    SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION, SensitiveDeletionPersistedCommitment,
    SensitiveDeletionPersistedCommitmentInput,
};

#[test]
fn persisted_commitment_exports_exact_versioned_wire_snapshot() {
    let input = SensitiveDeletionPersistedCommitmentInput {
        commitment_version: SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION,
        request_id: "delete-request-export-001".to_owned(),
        tenant_id: "tenant-alpha".to_owned(),
        retention_policy_id: "retention-30d-v1".to_owned(),
        declared_copy_count: 256,
        inventory_digest: "a".repeat(64),
    };
    let commitment = SensitiveDeletionPersistedCommitment::try_from(input.clone())
        .expect("bounded supported persisted commitment should reconstruct");

    let exported = SensitiveDeletionPersistedCommitmentInput::from(&commitment);

    assert_eq!(exported, input);
}
