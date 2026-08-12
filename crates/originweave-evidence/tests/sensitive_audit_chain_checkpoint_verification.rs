#![allow(clippy::expect_used)]

//! Trusted-checkpoint verification contract for sensitive-audit chain histories.
//!
//! A durable evidence reader may receive a separately trusted checkpoint for a prior link. The
//! verifier must first validate the complete supplied history, then require the exact checkpoint
//! tenant, stream, sequence, and chain digest. This deterministic contract does not authenticate,
//! store, publish, sign, or otherwise make the checkpoint trustworthy by itself.

use originweave_evidence::{
    SensitiveAuditChainCheckpoint, SensitiveAuditChainCheckpointInput, SensitiveAuditChainLink,
    SensitiveAuditChainLinkError, SensitiveAuditChainLinkInput,
    verify_sensitive_audit_chain_checkpoint, verify_sensitive_audit_chain_history,
};

const PAYLOAD_ONE: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const PAYLOAD_TWO: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const PAYLOAD_THREE: &str =
    "sha256:3333333333333333333333333333333333333333333333333333333333333333";
const REWRITTEN_PAYLOAD_TWO: &str =
    "sha256:4444444444444444444444444444444444444444444444444444444444444444";
const PLACEHOLDER_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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

fn checkpoint(link: &SensitiveAuditChainLink) -> SensitiveAuditChainCheckpoint {
    SensitiveAuditChainCheckpoint::try_from(SensitiveAuditChainCheckpointInput {
        tenant_id: link.tenant_id().to_owned(),
        audit_stream_id: link.audit_stream_id().to_owned(),
        sequence_number: link.sequence_number(),
        chain_digest: link.chain_digest().to_owned(),
    })
    .expect("valid link metadata should form a checkpoint")
}

#[test]
fn trusted_checkpoint_can_verify_a_longer_valid_history() {
    let history = valid_history();
    let trusted = checkpoint(&history[1]);

    assert_eq!(verify_sensitive_audit_chain_history(&history), Ok(()));
    assert_eq!(
        verify_sensitive_audit_chain_checkpoint(&history, &trusted),
        Ok(())
    );
    assert_eq!(trusted.tenant_id(), "tenant-alpha");
    assert_eq!(trusted.audit_stream_id(), "sensitive-audit-v1");
    assert_eq!(trusted.sequence_number(), 2);
    assert_eq!(trusted.chain_digest(), history[1].chain_digest());
}

#[test]
fn checkpoint_rejects_wrong_tenant_or_stream() {
    let history = valid_history();
    let wrong_tenant = SensitiveAuditChainCheckpoint::try_from(SensitiveAuditChainCheckpointInput {
        tenant_id: "tenant-beta".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: 2,
        chain_digest: history[1].chain_digest().to_owned(),
    })
    .expect("alternate bounded tenant should be syntactically valid");
    let wrong_stream = SensitiveAuditChainCheckpoint::try_from(SensitiveAuditChainCheckpointInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v2".to_owned(),
        sequence_number: 2,
        chain_digest: history[1].chain_digest().to_owned(),
    })
    .expect("alternate bounded stream should be syntactically valid");

    assert_eq!(
        verify_sensitive_audit_chain_checkpoint(&history, &wrong_tenant),
        Err(SensitiveAuditChainLinkError::CheckpointContextMismatch)
    );
    assert_eq!(
        verify_sensitive_audit_chain_checkpoint(&history, &wrong_stream),
        Err(SensitiveAuditChainLinkError::CheckpointContextMismatch)
    );
}

#[test]
fn checkpoint_sequence_beyond_loaded_history_fails_closed() {
    let history = valid_history();
    let future = SensitiveAuditChainCheckpoint::try_from(SensitiveAuditChainCheckpointInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: 4,
        chain_digest: PLACEHOLDER_DIGEST.to_owned(),
    })
    .expect("future checkpoint metadata should remain syntactically valid");

    assert_eq!(
        verify_sensitive_audit_chain_checkpoint(&history, &future),
        Err(SensitiveAuditChainLinkError::CheckpointSequenceMissing)
    );
}

#[test]
fn trusted_checkpoint_detects_an_internally_valid_rewritten_history() {
    let original = valid_history();
    let trusted = checkpoint(&original[1]);

    let first = link("tenant-alpha", "sensitive-audit-v1", 1, None, PAYLOAD_ONE);
    let rewritten_second = link(
        "tenant-alpha",
        "sensitive-audit-v1",
        2,
        Some(first.chain_digest()),
        REWRITTEN_PAYLOAD_TWO,
    );
    let rewritten_third = link(
        "tenant-alpha",
        "sensitive-audit-v1",
        3,
        Some(rewritten_second.chain_digest()),
        PAYLOAD_THREE,
    );
    let rewritten = vec![first, rewritten_second, rewritten_third];

    assert_eq!(verify_sensitive_audit_chain_history(&rewritten), Ok(()));
    assert_eq!(
        verify_sensitive_audit_chain_checkpoint(&rewritten, &trusted),
        Err(SensitiveAuditChainLinkError::CheckpointDigestMismatch)
    );
}

#[test]
fn invalid_history_fails_before_checkpoint_comparison() {
    let history = valid_history();
    let trusted = checkpoint(&history[1]);
    let corrupted_genesis = SensitiveAuditChainLink::try_from(SensitiveAuditChainLinkInput {
        tenant_id: "tenant-alpha".to_owned(),
        audit_stream_id: "sensitive-audit-v1".to_owned(),
        sequence_number: 1,
        previous_chain_digest: None,
        payload_digest: PAYLOAD_ONE.to_owned(),
        chain_digest: PLACEHOLDER_DIGEST.to_owned(),
    })
    .expect("canonical but wrong digest should remain loadable");

    assert_eq!(
        verify_sensitive_audit_chain_checkpoint(&[corrupted_genesis], &trusted),
        Err(SensitiveAuditChainLinkError::ChainDigestMismatch)
    );
}

#[test]
fn checkpoint_input_validation_is_bounded_and_fail_closed() {
    let invalid_identifier = SensitiveAuditChainCheckpoint::try_from(
        SensitiveAuditChainCheckpointInput {
            tenant_id: "---".to_owned(),
            audit_stream_id: "sensitive-audit-v1".to_owned(),
            sequence_number: 1,
            chain_digest: PLACEHOLDER_DIGEST.to_owned(),
        },
    );
    let invalid_stream = SensitiveAuditChainCheckpoint::try_from(
        SensitiveAuditChainCheckpointInput {
            tenant_id: "tenant-alpha".to_owned(),
            audit_stream_id: "---".to_owned(),
            sequence_number: 1,
            chain_digest: PLACEHOLDER_DIGEST.to_owned(),
        },
    );
    let invalid_sequence = SensitiveAuditChainCheckpoint::try_from(
        SensitiveAuditChainCheckpointInput {
            tenant_id: "tenant-alpha".to_owned(),
            audit_stream_id: "sensitive-audit-v1".to_owned(),
            sequence_number: 0,
            chain_digest: PLACEHOLDER_DIGEST.to_owned(),
        },
    );
    let invalid_digest = SensitiveAuditChainCheckpoint::try_from(
        SensitiveAuditChainCheckpointInput {
            tenant_id: "tenant-alpha".to_owned(),
            audit_stream_id: "sensitive-audit-v1".to_owned(),
            sequence_number: 1,
            chain_digest: "sha256:ABC".to_owned(),
        },
    );

    assert_eq!(
        invalid_identifier,
        Err(SensitiveAuditChainLinkError::InvalidIdentifier)
    );
    assert_eq!(
        invalid_stream,
        Err(SensitiveAuditChainLinkError::InvalidIdentifier)
    );
    assert_eq!(
        invalid_sequence,
        Err(SensitiveAuditChainLinkError::InvalidSequence)
    );
    assert_eq!(invalid_digest, Err(SensitiveAuditChainLinkError::InvalidDigest));
}

#[test]
fn checkpoint_errors_have_stable_source_free_text() {
    let cases = [
        (
            SensitiveAuditChainLinkError::CheckpointContextMismatch,
            "sensitive audit chain checkpoint context mismatch",
        ),
        (
            SensitiveAuditChainLinkError::CheckpointSequenceMissing,
            "sensitive audit chain checkpoint sequence is absent from history",
        ),
        (
            SensitiveAuditChainLinkError::CheckpointDigestMismatch,
            "sensitive audit chain checkpoint digest mismatch",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(std::error::Error::source(&error).is_none());
    }
}
