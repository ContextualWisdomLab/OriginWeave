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

fn intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(VALID_INTENT).expect("valid intent digest")
}

fn origin() -> Origin {
    Origin::parse("https://app.example").expect("valid test origin")
}

fn provenance(result: VerificationResult) -> ProvenanceRecord {
    ProvenanceRecord::new(
        "https://app.example/receipt",
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
            provenance(VerificationResult::Verified),
        )
        .expect("supported post-condition should admit verified evidence");

        assert_eq!(evidence.post_condition(), kind);
    }
}
