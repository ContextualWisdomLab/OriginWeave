#![allow(clippy::expect_used)]

//! Deterministic credential-free chain-link contract for sensitive audit evidence.
//!
//! The value does not persist records or compute/verify a cryptographic hash. It gives a durable
//! evidence owner one canonical unambiguous preimage and exact continuity checks so an external
//! SHA-256/signature service can make append-only sequencing independently verifiable.

use originweave_evidence::{
    SensitiveAuditChainLink, SensitiveAuditChainLinkError, SensitiveAuditChainLinkInput,
};

const FIRST_PAYLOAD_DIGEST: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const FIRST_CHAIN_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SECOND_PAYLOAD_DIGEST: &str =
    "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const SECOND_CHAIN_DIGEST: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn genesis_input() -> SensitiveAuditChainLinkInput {
    SensitiveAuditChainLinkInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: 1,
        previous_chain_digest: None,
        payload_digest: FIRST_PAYLOAD_DIGEST.to_owned(),
        chain_digest: FIRST_CHAIN_DIGEST.to_owned(),
    }
}

fn next_input() -> SensitiveAuditChainLinkInput {
    SensitiveAuditChainLinkInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: 2,
        previous_chain_digest: Some(FIRST_CHAIN_DIGEST.to_owned()),
        payload_digest: SECOND_PAYLOAD_DIGEST.to_owned(),
        chain_digest: SECOND_CHAIN_DIGEST.to_owned(),
    }
}

#[test]
fn genesis_and_next_link_preserve_exact_chain_context() {
    let genesis = SensitiveAuditChainLink::try_from(genesis_input()).expect("valid genesis link");
    let next = genesis
        .try_next(next_input())
        .expect("valid contiguous next link");

    assert_eq!(genesis.tenant_id(), "tenant-alpha");
    assert_eq!(genesis.audit_stream_id(), "sensitive-audit-v1");
    assert_eq!(genesis.sequence_number(), 1);
    assert_eq!(genesis.previous_chain_digest(), None);
    assert_eq!(genesis.payload_digest(), FIRST_PAYLOAD_DIGEST);
    assert_eq!(genesis.chain_digest(), FIRST_CHAIN_DIGEST);

    assert_eq!(next.sequence_number(), 2);
    assert_eq!(next.previous_chain_digest(), Some(FIRST_CHAIN_DIGEST));
    assert_eq!(next.payload_digest(), SECOND_PAYLOAD_DIGEST);
    assert_eq!(next.chain_digest(), SECOND_CHAIN_DIGEST);
}

#[test]
fn canonical_hash_preimage_is_deterministic_and_length_delimited() {
    let genesis = SensitiveAuditChainLink::try_from(genesis_input()).expect("valid genesis link");
    let next = genesis
        .try_next(next_input())
        .expect("valid contiguous next link");

    let genesis_preimage = genesis.canonical_hash_preimage();
    let repeated_preimage = genesis.canonical_hash_preimage();
    let next_preimage = next.canonical_hash_preimage();

    assert_eq!(genesis_preimage, repeated_preimage);
    assert_ne!(genesis_preimage, next_preimage);
    assert!(genesis_preimage.starts_with(b"originweave-sensitive-audit-chain-v1\0"));
    assert!(!genesis_preimage.windows(2).any(|window| window == b"//"));
    assert!(!genesis_preimage.is_empty());
}

#[test]
fn genesis_and_non_genesis_shape_fail_closed() {
    let mut nonzero_genesis = genesis_input();
    nonzero_genesis.sequence_number = 2;
    assert_eq!(
        SensitiveAuditChainLink::try_from(nonzero_genesis),
        Err(SensitiveAuditChainLinkError::MissingPreviousDigest)
    );

    let mut previous_on_genesis = genesis_input();
    previous_on_genesis.previous_chain_digest = Some(FIRST_CHAIN_DIGEST.to_owned());
    assert_eq!(
        SensitiveAuditChainLink::try_from(previous_on_genesis),
        Err(SensitiveAuditChainLinkError::UnexpectedPreviousDigest)
    );

    let mut zero_sequence = genesis_input();
    zero_sequence.sequence_number = 0;
    assert_eq!(
        SensitiveAuditChainLink::try_from(zero_sequence),
        Err(SensitiveAuditChainLinkError::InvalidSequence)
    );
}

#[test]
fn malformed_identifiers_and_digests_fail_closed() {
    for invalid_tenant in ["", "tenant with spaces", "---"] {
        let mut input = genesis_input();
        input.tenant_id = invalid_tenant.to_owned();
        assert_eq!(
            SensitiveAuditChainLink::try_from(input),
            Err(SensitiveAuditChainLinkError::InvalidIdentifier)
        );
    }

    for invalid_digest in [
        "",
        "sha256:ABCDEF1111111111111111111111111111111111111111111111111111111111",
        "sha256:1111",
        "sha512:1111111111111111111111111111111111111111111111111111111111111111",
    ] {
        let mut input = genesis_input();
        input.payload_digest = invalid_digest.to_owned();
        assert_eq!(
            SensitiveAuditChainLink::try_from(input),
            Err(SensitiveAuditChainLinkError::InvalidDigest)
        );
    }
}

#[test]
fn next_link_requires_exact_tenant_stream_sequence_and_previous_digest() {
    let genesis = SensitiveAuditChainLink::try_from(genesis_input()).expect("valid genesis link");

    let mut wrong_tenant = next_input();
    wrong_tenant.tenant_id = "tenant-beta".to_owned();
    assert_eq!(
        genesis.try_next(wrong_tenant),
        Err(SensitiveAuditChainLinkError::ChainContextMismatch)
    );

    let mut wrong_stream = next_input();
    wrong_stream.audit_stream_id = "other-stream".to_owned();
    assert_eq!(
        genesis.try_next(wrong_stream),
        Err(SensitiveAuditChainLinkError::ChainContextMismatch)
    );

    let mut skipped_sequence = next_input();
    skipped_sequence.sequence_number = 3;
    assert_eq!(
        genesis.try_next(skipped_sequence),
        Err(SensitiveAuditChainLinkError::SequenceDiscontinuity)
    );

    let mut wrong_previous = next_input();
    wrong_previous.previous_chain_digest = Some(SECOND_CHAIN_DIGEST.to_owned());
    assert_eq!(
        genesis.try_next(wrong_previous),
        Err(SensitiveAuditChainLinkError::PreviousDigestMismatch)
    );
}

#[test]
fn sequence_overflow_fails_before_next_link_can_be_admitted() {
    let terminal = SensitiveAuditChainLink::try_from(SensitiveAuditChainLinkInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: u64::MAX,
        previous_chain_digest: Some(FIRST_CHAIN_DIGEST.to_owned()),
        payload_digest: SECOND_PAYLOAD_DIGEST.to_owned(),
        chain_digest: SECOND_CHAIN_DIGEST.to_owned(),
    })
    .expect("valid terminal link shape");

    assert_eq!(
        terminal.try_next(next_input()),
        Err(SensitiveAuditChainLinkError::SequenceOverflow)
    );
}

#[test]
fn public_errors_are_stable_and_source_free() {
    let cases = [
        (
            SensitiveAuditChainLinkError::InvalidIdentifier,
            "invalid sensitive audit chain identifier",
        ),
        (
            SensitiveAuditChainLinkError::InvalidDigest,
            "invalid sensitive audit chain digest",
        ),
        (
            SensitiveAuditChainLinkError::InvalidSequence,
            "invalid sensitive audit chain sequence",
        ),
        (
            SensitiveAuditChainLinkError::UnexpectedPreviousDigest,
            "genesis sensitive audit chain link has a previous digest",
        ),
        (
            SensitiveAuditChainLinkError::MissingPreviousDigest,
            "non-genesis sensitive audit chain link is missing its previous digest",
        ),
        (
            SensitiveAuditChainLinkError::ChainContextMismatch,
            "sensitive audit chain context mismatch",
        ),
        (
            SensitiveAuditChainLinkError::SequenceDiscontinuity,
            "sensitive audit chain sequence is not contiguous",
        ),
        (
            SensitiveAuditChainLinkError::PreviousDigestMismatch,
            "sensitive audit chain previous digest mismatch",
        ),
        (
            SensitiveAuditChainLinkError::SequenceOverflow,
            "sensitive audit chain sequence overflow",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(std::error::Error::source(&error).is_none());
    }
}
