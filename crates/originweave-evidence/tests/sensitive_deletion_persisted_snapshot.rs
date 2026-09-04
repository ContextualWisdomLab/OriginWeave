#![allow(clippy::expect_used)]

//! Canonical export contract for persisted sensitive-data deletion commitments.
//!
//! A durable evidence owner must not have to duplicate field-copying or count-width conversion when
//! serializing a validated commitment. The production authority therefore owns conversion back to
//! the exact versioned wire input that it accepts during reconstruction.

use std::error::Error;

use originweave_evidence::{
    SENSITIVE_DELETION_INVENTORY_COMMITMENT_VERSION, SensitiveDeletionPersistedCommitment,
    SensitiveDeletionPersistedCommitmentError, SensitiveDeletionPersistedCommitmentInput,
};

const WIRE_DOMAIN: &[u8] = b"originweave-sensitive-deletion-persisted-commitment\0";

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

fn wire_with_fields(fields: &[&[u8]]) -> Vec<u8> {
    let mut wire = WIRE_DOMAIN.to_vec();
    for field in fields {
        wire.extend_from_slice(field.len().to_string().as_bytes());
        wire.push(b':');
        wire.extend_from_slice(field);
    }
    wire
}

fn complete_wire_with_version(version: &[u8]) -> Vec<u8> {
    wire_with_fields(&[
        version,
        b"delete-request-export-001",
        b"tenant-alpha",
        b"retention-30d-v1",
        b"256",
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ])
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
    leading_zero_length.splice(
        WIRE_DOMAIN.len()..WIRE_DOMAIN.len() + 1,
        b"01".iter().copied(),
    );

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

#[test]
fn persisted_commitment_wire_parser_rejects_every_noncanonical_length_form() {
    for suffix in [
        b"1".as_slice(),
        b":1".as_slice(),
        b"0000:1".as_slice(),
        b"x:1".as_slice(),
    ] {
        let mut wire = WIRE_DOMAIN.to_vec();
        wire.extend_from_slice(suffix);
        assert_eq!(
            SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(&wire),
            Err(SensitiveDeletionPersistedCommitmentError::InvalidWireEncoding),
            "length encoding must fail closed: {suffix:?}"
        );
    }
}

#[test]
fn persisted_commitment_wire_parser_rejects_every_noncanonical_integer_form() {
    for version in [
        b"".as_slice(),
        b"123456".as_slice(),
        b"01".as_slice(),
        b"x".as_slice(),
        b"65536".as_slice(),
    ] {
        let wire = complete_wire_with_version(version);
        assert_eq!(
            SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(&wire),
            Err(SensitiveDeletionPersistedCommitmentError::InvalidWireEncoding),
            "integer encoding must fail closed: {version:?}"
        );
    }

    let invalid_count = wire_with_fields(&[
        b"1",
        b"delete-request-export-001",
        b"tenant-alpha",
        b"retention-30d-v1",
        b"x",
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ]);
    assert_eq!(
        SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(&invalid_count),
        Err(SensitiveDeletionPersistedCommitmentError::InvalidWireEncoding),
    );
}

#[test]
fn persisted_commitment_wire_parser_rejects_wrong_domain_oversize_and_invalid_utf8() {
    let mut wrong_domain = persisted_commitment().canonical_wire_bytes();
    wrong_domain[0] = b'X';

    let mut oversized = WIRE_DOMAIN.to_vec();
    oversized.resize(10_000, b'x');

    let invalid_utf8_request = wire_with_fields(&[
        b"1",
        &[0xff],
        b"tenant-alpha",
        b"retention-30d-v1",
        b"256",
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ]);
    let invalid_utf8_tenant = wire_with_fields(&[
        b"1",
        b"delete-request-export-001",
        &[0xff],
        b"retention-30d-v1",
        b"256",
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ]);
    let invalid_utf8_policy = wire_with_fields(&[
        b"1",
        b"delete-request-export-001",
        b"tenant-alpha",
        &[0xff],
        b"256",
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ]);
    let invalid_utf8_digest = wire_with_fields(&[
        b"1",
        b"delete-request-export-001",
        b"tenant-alpha",
        b"retention-30d-v1",
        b"256",
        &[0xff],
    ]);

    for hostile_wire in [
        &wrong_domain[..],
        &oversized[..],
        &invalid_utf8_request[..],
        &invalid_utf8_tenant[..],
        &invalid_utf8_policy[..],
        &invalid_utf8_digest[..],
    ] {
        assert_eq!(
            SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(hostile_wire),
            Err(SensitiveDeletionPersistedCommitmentError::InvalidWireEncoding),
        );
    }
}

#[test]
fn persisted_commitment_wire_parser_rejects_each_missing_field_boundary() {
    let complete_fields: [&[u8]; 6] = [
        b"1",
        b"delete-request-export-001",
        b"tenant-alpha",
        b"retention-30d-v1",
        b"256",
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ];

    for included_field_count in 1..6 {
        let wire = wire_with_fields(&complete_fields[..included_field_count]);
        assert_eq!(
            SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(&wire),
            Err(SensitiveDeletionPersistedCommitmentError::InvalidWireEncoding),
            "wire missing field after index {included_field_count} must fail closed"
        );
    }
}

#[test]
fn persisted_commitment_wire_parser_preserves_typed_version_and_metadata_errors() {
    let unsupported = complete_wire_with_version(b"2");
    assert_eq!(
        SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(&unsupported),
        Err(SensitiveDeletionPersistedCommitmentError::UnsupportedCommitmentVersion),
    );

    let empty_request = wire_with_fields(&[
        b"1",
        b"",
        b"tenant-alpha",
        b"retention-30d-v1",
        b"256",
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ]);
    assert!(matches!(
        SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(&empty_request),
        Err(SensitiveDeletionPersistedCommitmentError::InvalidCommitment(_))
    ));
}

#[test]
fn invalid_wire_error_is_deterministic_and_source_free() {
    let error = SensitiveDeletionPersistedCommitment::from_canonical_wire_bytes(b"not-originweave")
        .expect_err("non-OriginWeave wire bytes must fail closed");

    assert_eq!(
        error,
        SensitiveDeletionPersistedCommitmentError::InvalidWireEncoding
    );
    assert_eq!(
        error.to_string(),
        "persisted sensitive deletion commitment wire encoding is invalid"
    );
    assert!(error.source().is_none());
}
