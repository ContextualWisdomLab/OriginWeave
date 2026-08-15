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

fn persisted_commitment() -> SensitiveDeletionPersistedCommitment {
    SensitiveDeletionPersistedCommitment::try_from(SensitiveDeletionPersistedCommitmentInput {
        commitment_version: SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION,
        request_id: "delete-request-export-001".to_owned(),
        tenant_id: "tenant-alpha".to_owned(),
        retention_policy_id: "retention-30d-v1".to_owned(),
        declared_copy_count: 256,
        inventory_digest: "a".repeat(64),
    })
    .expect("bounded supported persisted commitment should reconstruct")
}

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

#[test]
fn persisted_commitment_has_architecture_independent_canonical_wire_bytes() {
    let commitment = persisted_commitment();

    let mut expected = b"originweave-sensitive-deletion-persisted-commitment\0\
1:125:delete-request-export-00112:tenant-alpha16:retention-30d-v13:25664:"
        .to_vec();
    expected.extend_from_slice("a".repeat(64).as_bytes());

    assert_eq!(commitment.canonical_wire_bytes(), expected);
}

#[test]
fn persisted_commitment_canonical_wire_round_trips_through_originweave() {
    let commitment = persisted_commitment();
    let wire = commitment.canonical_wire_bytes();

    let reconstructed = SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(&wire)
        .expect("canonical OriginWeave wire bytes should reconstruct");

    assert_eq!(reconstructed, commitment);
}

#[test]
fn persisted_commitment_canonical_wire_rejects_ambiguous_or_trailing_encodings() {
    let commitment = persisted_commitment();
    let wire = commitment.canonical_wire_bytes();

    let mut leading_zero_length = wire.clone();
    let domain_length = b"originweave-sensitive-deletion-persisted-commitment\0".len();
    leading_zero_length.splice(domain_length..domain_length + 1, b"01".iter().copied());

    let mut trailing_bytes = wire.clone();
    trailing_bytes.extend_from_slice(b"junk");

    let truncated = &wire[..wire.len() - 1];

    for hostile_wire in [&leading_zero_length[..], &trailing_bytes[..], truncated] {
        assert!(
            SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(hostile_wire).is_err(),
            "hostile wire encoding must fail closed: {hostile_wire:?}"
        );
    }
}
