#![allow(clippy::expect_used)]

//! Cryptographic verification contract for sensitive-audit chain linkage.
//!
//! The chain-link value already defines a canonical domain-separated preimage. These regressions
//! require OriginWeave to compute SHA-256 over exactly that preimage and fail closed when a stored
//! chain digest does not match, without persisting records or treating a digest as a signature.

use originweave_evidence::{
    SensitiveAuditChainLink, SensitiveAuditChainLinkError, SensitiveAuditChainLinkInput,
};

const PAYLOAD_DIGEST: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const EXPECTED_CHAIN_DIGEST: &str =
    "sha256:fe9799296d80c1d755c513c7f71773cb7c32ba47edfcf1922aadb1efb28413ea";
const WRONG_CHAIN_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn genesis_input(chain_digest: &str) -> SensitiveAuditChainLinkInput {
    SensitiveAuditChainLinkInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: 1,
        previous_chain_digest: None,
        payload_digest: PAYLOAD_DIGEST.to_owned(),
        chain_digest: chain_digest.to_owned(),
    }
}

#[test]
fn canonical_preimage_hash_matches_known_sha256_vector() {
    let link = SensitiveAuditChainLink::try_from(genesis_input(EXPECTED_CHAIN_DIGEST))
        .expect("canonical digest should construct a chain link");

    assert_eq!(link.computed_chain_digest(), EXPECTED_CHAIN_DIGEST);
    assert_eq!(link.verify_chain_digest(), Ok(()));
}

#[test]
fn stored_digest_mismatch_fails_closed_without_changing_the_link() {
    let link = SensitiveAuditChainLink::try_from(genesis_input(WRONG_CHAIN_DIGEST))
        .expect("canonical but incorrect digest should remain loadable for verification");

    assert_eq!(link.computed_chain_digest(), EXPECTED_CHAIN_DIGEST);
    let error = link
        .verify_chain_digest()
        .expect_err("mismatched stored chain digest must fail closed");
    assert_eq!(error, SensitiveAuditChainLinkError::ChainDigestMismatch);
    assert_eq!(
        error.to_string(),
        "sensitive audit chain digest does not match its canonical preimage"
    );
    assert!(std::error::Error::source(&error).is_none());
    assert_eq!(link.chain_digest(), WRONG_CHAIN_DIGEST);
}

#[test]
fn payload_change_changes_computed_digest_and_invalidates_replayed_digest() {
    let original = SensitiveAuditChainLink::try_from(genesis_input(EXPECTED_CHAIN_DIGEST))
        .expect("valid original chain link");
    let mut changed = genesis_input(EXPECTED_CHAIN_DIGEST);
    changed.payload_digest =
        "sha256:2222222222222222222222222222222222222222222222222222222222222222".to_owned();
    let changed = SensitiveAuditChainLink::try_from(changed).expect("valid changed payload link");

    assert_ne!(
        changed.computed_chain_digest(),
        original.computed_chain_digest()
    );
    assert_eq!(
        changed.verify_chain_digest(),
        Err(SensitiveAuditChainLinkError::ChainDigestMismatch)
    );
}
