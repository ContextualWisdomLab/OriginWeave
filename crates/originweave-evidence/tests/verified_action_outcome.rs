#![allow(clippy::expect_used)]

use originweave_core::{ActionIntentDigest, ActionKind, Origin};
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
fn verified_post_condition_can_create_action_success_evidence() {
    let target = origin();
    let evidence = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        target.clone(),
        intent(),
        PostConditionKind::NodeStateChanged,
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
        provenance(VerificationResult::Verified),
    )
    .expect("verified post-condition should admit success evidence");

    assert_eq!(evidence.action(), ActionKind::Submit);
    assert_eq!(evidence.target_origin(), &target);
    assert_eq!(evidence.intent_digest(), &intent());
    assert_eq!(
        evidence.post_condition(),
        PostConditionKind::NodeStateChanged
    );
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
            PostConditionKind::NodeStateChanged,
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
        PostConditionKind::NodeStateChanged,
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
fn node_state_post_condition_provenance_must_match_the_action_target_origin() {
    let error = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        origin(),
        intent(),
        PostConditionKind::NodeStateChanged,
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
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
    let evidence = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        origin(),
        intent(),
        PostConditionKind::NodeStateChanged,
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
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
fn post_condition_kinds_cover_first_browser_vertical_slice_evidence() {
    for kind in [
        PostConditionKind::UrlChanged,
        PostConditionKind::NodeStateChanged,
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
        .expect("supported post-condition should admit verified evidence");

        assert_eq!(evidence.post_condition(), kind);
    }
}
