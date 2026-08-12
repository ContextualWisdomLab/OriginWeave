#![allow(clippy::expect_used)]

use originweave_core::{
    ActionIntentDigest, ActionKind, BrowserSessionId, BrowsingContextId, DocumentEpoch,
    ObservedNodeHandle, Origin,
};
use originweave_evidence::{
    EvidenceSourceKind, PostConditionKind, ProvenanceRecord, VerificationResult,
    VerifiedActionOutcomeError, VerifiedActionOutcomeEvidence,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VALID_SOURCE_HASH: &str =
    "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const DISPATCHED_AT_MILLISECONDS: u64 = 1_000;
const OBSERVED_AT_MILLISECONDS: u64 = 1_025;

fn intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(VALID_INTENT).expect("valid intent digest")
}

fn origin() -> Origin {
    Origin::parse("https://app.example").expect("valid test origin")
}

fn node(node_id: u64) -> ObservedNodeHandle {
    ObservedNodeHandle::new(
        BrowserSessionId::new(7).expect("valid browser session"),
        BrowsingContextId::new(11).expect("valid browsing context"),
        origin(),
        DocumentEpoch::new(13).expect("valid document epoch"),
        node_id,
    )
    .expect("valid observed node")
}

fn provenance(result: VerificationResult) -> ProvenanceRecord {
    provenance_at("https://app.example/receipt", result)
}

fn provenance_at(source_url: &str, result: VerificationResult) -> ProvenanceRecord {
    ProvenanceRecord::new(
        source_url,
        "dom:#receipt-status",
        VALID_SOURCE_HASH,
        EvidenceSourceKind::DomTree,
        result,
    )
    .expect("valid provenance")
}

#[test]
fn verified_non_node_post_condition_can_create_action_success_evidence() {
    let target = origin();
    let evidence = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        target.clone(),
        intent(),
        PostConditionKind::UrlChanged,
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
        provenance(VerificationResult::Verified),
    )
    .expect("verified non-node post-condition should admit success evidence");

    assert_eq!(evidence.action(), ActionKind::Submit);
    assert_eq!(evidence.target_origin(), &target);
    assert_eq!(evidence.intent_digest(), &intent());
    assert_eq!(evidence.post_condition(), PostConditionKind::UrlChanged);
    assert_eq!(evidence.target_node(), None);
    assert_eq!(
        evidence.dispatched_at_milliseconds(),
        DISPATCHED_AT_MILLISECONDS
    );
    assert_eq!(
        evidence.observed_at_milliseconds(),
        OBSERVED_AT_MILLISECONDS
    );
    assert_eq!(
        evidence.provenance().verification_result(),
        VerificationResult::Verified
    );
}

#[test]
fn unverified_or_rejected_post_condition_cannot_be_recorded_as_success() {
    for result in [VerificationResult::Unverified, VerificationResult::Rejected] {
        let error = VerifiedActionOutcomeEvidence::new(
            ActionKind::Submit,
            origin(),
            intent(),
            PostConditionKind::UrlChanged,
            DISPATCHED_AT_MILLISECONDS,
            OBSERVED_AT_MILLISECONDS,
            provenance(result),
        )
        .expect_err("non-verified post-condition must fail closed");

        assert_eq!(error, VerifiedActionOutcomeError::PostConditionNotVerified);
        assert_eq!(
            error.to_string(),
            "action success requires an independently verified post-condition"
        );
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn post_condition_observation_cannot_predate_action_dispatch() {
    let error = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        origin(),
        intent(),
        PostConditionKind::UrlChanged,
        2_000,
        1_999,
        provenance(VerificationResult::Verified),
    )
    .expect_err("pre-dispatch observation cannot prove action success");

    assert_eq!(
        error,
        VerifiedActionOutcomeError::PostConditionPredatesDispatch {
            dispatched_at_milliseconds: 2_000,
            observed_at_milliseconds: 1_999,
        }
    );
    assert_eq!(
        error.to_string(),
        "post-condition observation at 1999 ms predates action dispatch at 2000 ms"
    );
}

#[test]
fn generic_constructor_cannot_bypass_node_identity_binding() {
    let error = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        origin(),
        intent(),
        PostConditionKind::NodeStateChanged,
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
        provenance(VerificationResult::Verified),
    )
    .expect_err("node-state success must require exact node authority");

    assert_eq!(error, VerifiedActionOutcomeError::NodeStateTargetRequired);
    assert_eq!(
        error.to_string(),
        "node-state post-condition requires the exact governed action target node"
    );
}

#[test]
fn node_state_post_condition_provenance_must_match_the_action_target_origin() {
    let target_node = node(17);
    let error = VerifiedActionOutcomeEvidence::new_node_state(
        ActionKind::Submit,
        target_node.clone(),
        intent(),
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
        target_node,
        provenance_at(
            "https://attacker.example/receipt",
            VerificationResult::Verified,
        ),
    )
    .expect_err("a different origin cannot prove the target node changed");

    assert_eq!(
        error,
        VerifiedActionOutcomeError::PostConditionOriginMismatch
    );
    assert_eq!(
        error.to_string(),
        "node-state post-condition provenance must match the governed action target origin"
    );
}

#[test]
fn node_state_origin_comparison_uses_canonical_origin_semantics() {
    let target_node = node(17);
    let evidence = VerifiedActionOutcomeEvidence::new_node_state(
        ActionKind::Submit,
        target_node.clone(),
        intent(),
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
        target_node,
        provenance_at(
            "https://APP.EXAMPLE:443/receipt",
            VerificationResult::Verified,
        ),
    )
    .expect("canonical equivalent source origin should be accepted");

    assert_eq!(evidence.target_origin(), &origin());
}

#[test]
fn same_monotonic_tick_is_allowed_for_coarse_clock_sources() {
    let evidence = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        origin(),
        intent(),
        PostConditionKind::NetworkMutationObserved,
        4_000,
        4_000,
        provenance(VerificationResult::Verified),
    )
    .expect("coarse monotonic clocks may observe within the dispatch tick");

    assert_eq!(evidence.dispatched_at_milliseconds(), 4_000);
    assert_eq!(evidence.observed_at_milliseconds(), 4_000);
}

#[test]
fn non_node_post_condition_kinds_remain_supported() {
    for kind in [
        PostConditionKind::UrlChanged,
        PostConditionKind::DialogStateChanged,
        PostConditionKind::NetworkMutationObserved,
    ] {
        let evidence = VerifiedActionOutcomeEvidence::new(
            ActionKind::Submit,
            origin(),
            intent(),
            kind,
            DISPATCHED_AT_MILLISECONDS,
            OBSERVED_AT_MILLISECONDS,
            provenance(VerificationResult::Verified),
        )
        .expect("supported non-node post-condition should admit verified evidence");

        assert_eq!(evidence.post_condition(), kind);
        assert_eq!(evidence.target_node(), None);
    }
}

#[test]
fn node_state_success_binds_the_exact_action_target_node() {
    let target_node = node(17);
    let evidence = VerifiedActionOutcomeEvidence::new_node_state(
        ActionKind::Submit,
        target_node.clone(),
        intent(),
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
        target_node.clone(),
        provenance(VerificationResult::Verified),
    )
    .expect("the exact observed target node should prove its node-state post-condition");

    assert_eq!(evidence.target_origin(), target_node.origin());
    assert_eq!(evidence.target_node(), Some(&target_node));
    assert_eq!(
        evidence.post_condition(),
        PostConditionKind::NodeStateChanged
    );
}

#[test]
fn same_origin_different_node_cannot_prove_node_state_success() {
    let error = VerifiedActionOutcomeEvidence::new_node_state(
        ActionKind::Submit,
        node(17),
        intent(),
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
        node(18),
        provenance(VerificationResult::Verified),
    )
    .expect_err("a different same-origin node must not prove the action target changed");

    assert_eq!(error, VerifiedActionOutcomeError::PostConditionNodeMismatch);
    assert_eq!(
        error.to_string(),
        "node-state post-condition must observe the exact governed action target node"
    );
}
