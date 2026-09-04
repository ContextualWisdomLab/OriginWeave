//! Fixed-width wire contract for persisted sensitive-data deletion commitments.
//!
//! Persisted evidence crosses process and machine boundaries. Its public wire-facing count must
//! therefore use a fixed-width integer rather than architecture-dependent `usize`, while the
//! existing bounded validator remains authoritative for the accepted 1..=256 semantic range.

use originweave_evidence::{
    SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION, SensitiveDeletionInventoryCommitmentError,
    SensitiveDeletionPersistedCommitment, SensitiveDeletionPersistedCommitmentError,
    SensitiveDeletionPersistedCommitmentInput,
};

#[test]
fn persisted_declared_copy_count_is_fixed_width_and_still_bounded() {
    let input = SensitiveDeletionPersistedCommitmentInput {
        commitment_version: SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION,
        request_id: "delete-request-wire-001".to_owned(),
        tenant_id: "tenant-alpha".to_owned(),
        retention_policy_id: "retention-30d-v1".to_owned(),
        declared_copy_count: u16::MAX,
        inventory_digest: "0".repeat(64),
    };

    assert_eq!(
        SensitiveDeletionPersistedCommitment::try_from(input),
        Err(
            SensitiveDeletionPersistedCommitmentError::InvalidCommitment(
                SensitiveDeletionInventoryCommitmentError::InvalidDeclaredCopyCount,
            )
        )
    );
}
