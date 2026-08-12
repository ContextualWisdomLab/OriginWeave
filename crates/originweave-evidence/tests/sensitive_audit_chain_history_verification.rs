#![allow(clippy::expect_used)]

//! Complete-history verification contract for sensitive-audit chain linkage.
//!
//! A durable evidence reader or exporter must be able to validate a bounded loaded history from
//! genesis through its final link without manually composing per-link digest and adjacency checks.
//! This contract remains deterministic in-memory verification: it does not persist records,
//! authenticate a signer, or make storage append-only.

use originweave_evidence::{
    SensitiveAuditChainLink, SensitiveAuditChainLinkError, SensitiveAuditChainLinkInput,
    verify_sensitive_audit_chain_history,
};

const PAYLOAD_ONE: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const PAYLOAD_TWO: &str =
    "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const PAYLOAD_THREE: &str =
    "sha256:3333333333333333333333333333333333333333333333333333333333333333";
const PLACEHOLDER_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_DIGEST: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn link(
    tenant_id: &str,
    audit_stream_id: &str,
    sequence_number: u64,
    previous_chain_digest: Option<&str>,
    payload_digest: &str,
) -> SensitiveAuditChainLink {
    let input = SensitiveAuditChainLinkInput {
        tenant_id: tenant_id.to_owned(),
        audit_stream_id: audit_stream_id.to_owned(),
        sequence_number,
        previous_chain_digest: previous_chain_digest.map(str::to_owned),
        payload_digest: payload_digest.to_owned(),
        chain_digest: PLACEHOLDER_DIGEST.to_owned(),
    };
    let provisional = SensitiveAuditChainLink::try_from(input.clone())
        .expect("fixture metadata should form a chain link");
    let mut exact = input;
    exact.chain_digest = provisional.computed_chain_digest();
    SensitiveAuditChainLink::try_from(exact).expect("computed digest should remain canonical")
}

fn valid_history() -> Vec<SensitiveAuditChainLink> {
    let first = link("tenant-alpha", "sensitive-audit-v1", 1, None, PAYLOAD_ONE);
    let second = link(
        "tenant-alpha",
        "sensitive-audit-v1",
        2,
        Some(first.chain_digest()),
        PAYLOAD_TWO,
    );
    let third = link(
        "tenant-alpha",
        "sensitive-audit-v1",
        3,
        Some(second.chain_digest()),
        PAYLOAD_THREE,
    );
    vec![first, second, third]
}

#[test]
fn complete_history_verifies_from_genesis_through_final_link() {
    let history = valid_history();

    assert_eq!(verify_sensitive_audit_chain_history(&history), Ok(()));
    assert_eq!(
        verify_sensitive_audit_chain_history(&history[..1]),
        Ok(())
    );
}

#[test]
fn empty_history_fails_closed_with_stable_source_free_error() {
    let error = verify_sensitive_audit_chain_history(&[])
        .expect_err("an empty export must not be accepted as a verified history");

    assert_eq!(error, SensitiveAuditChainLinkError::EmptyHistory);
    assert_eq!(error.to_string(), "sensitive audit chain history is empty");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn truncated_prefix_that_does_not_start_at_genesis_fails_closed() {
    let history = valid_history();
    let error = verify_sensitive_audit_chain_history(&history[1..])
        .expect_err("a complete-history verifier must require the genesis link");

    assert_eq!(
        error,
        SensitiveAuditChainLinkError::HistoryDoesNotStartAtGenesis
    );
    assert_eq!(
        error.to_string(),
        "sensitive audit chain history does not start at genesis"
    );
}

#[test]
fn corrupted_genesis_digest_fails_before_later_links_can_mask_it() {
    let corrupted = SensitiveAuditChainLink::try_from(SensitiveAuditChainLinkInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: 1,
        previous_chain_digest: None,
        payload_digest: PAYLOAD_ONE.to_owned(),
        chain_digest: PLACEHOLDER_DIGEST.to_owned(),
    })
    .expect("canonical but incorrect digest should remain loadable");

    assert_eq!(
        verify_sensitive_audit_chain_history(&[corrupted]),
        Err(SensitiveAuditChainLinkError::ChainDigestMismatch)
    );
}

#[test]
fn corrupted_successor_digest_fails_closed() {
    let first = link("tenant-alpha", "sensitive-audit-v1", 1, None, PAYLOAD_ONE);
    let corrupted = SensitiveAuditChainLink::try_from(SensitiveAuditChainLinkInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: 2,
        previous_chain_digest: Some(first.chain_digest().to_owned()),
        payload_digest: PAYLOAD_TWO.to_owned(),
        chain_digest: PLACEHOLDER_DIGEST.to_owned(),
    })
    .expect("canonical but incorrect successor digest should remain loadable");

    assert_eq!(
        verify_sensitive_audit_chain_history(&[first, corrupted]),
        Err(SensitiveAuditChainLinkError::ChainDigestMismatch)
    );
}

#[test]
fn successor_context_change_fails_closed_even_with_a_valid_digest() {
    let first = link("tenant-alpha", "sensitive-audit-v1", 1, None, PAYLOAD_ONE);
    let second = link(
        "tenant-beta",
        "sensitive-audit-v1",
        2,
        Some(first.chain_digest()),
        PAYLOAD_TWO,
    );

    assert_eq!(
        verify_sensitive_audit_chain_history(&[first, second]),
        Err(SensitiveAuditChainLinkError::ChainContextMismatch)
    );
}

#[test]
fn successor_sequence_gap_fails_closed_even_with_a_valid_digest() {
    let first = link("tenant-alpha", "sensitive-audit-v1", 1, None, PAYLOAD_ONE);
    let third = link(
        "tenant-alpha",
        "sensitive-audit-v1",
        3,
        Some(first.chain_digest()),
        PAYLOAD_THREE,
    );

    assert_eq!(
        verify_sensitive_audit_chain_history(&[first, third]),
        Err(SensitiveAuditChainLinkError::SequenceDiscontinuity)
    );
}

#[test]
fn successor_previous_digest_mismatch_fails_closed_even_with_a_valid_digest() {
    let first = link("tenant-alpha", "sensitive-audit-v1", 1, None, PAYLOAD_ONE);
    let second = link(
        "tenant-alpha",
        "sensitive-audit-v1",
        2,
        Some(OTHER_DIGEST),
        PAYLOAD_TWO,
    );

    assert_eq!(
        verify_sensitive_audit_chain_history(&[first, second]),
        Err(SensitiveAuditChainLinkError::PreviousDigestMismatch)
    );
}
