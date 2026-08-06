#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, ApprovalScope, Capability,
    ExecutionPurpose, InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery,
    SessionMode,
};
use originweave_policy::{Decision, DenialReason, evaluate};

const INTENT_A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const INTENT_B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[test]
fn approval_for_one_purchase_payload_cannot_authorize_another() {
    let origin = Origin::parse("https://shop.example").expect("origin");
    let digest_a = ActionIntentDigest::parse(INTENT_A).expect("intent A");
    let digest_b = ActionIntentDigest::parse(INTENT_B).expect("intent B");
    let request_b = ActionRequest::new(
        ActionKind::Purchase,
        origin.clone(),
        origin.clone(),
        InstructionSource::User,
        SecretDelivery::None,
        digest_b,
    );
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Purchase]),
        BTreeSet::from([origin.clone()]),
        BTreeSet::from([origin.clone()]),
        RobotsDecision::NotApplicable,
        ApprovalEvidence::UserConfirmed(ApprovalScope::new(ActionKind::Purchase, origin, digest_a)),
    );

    assert_eq!(
        evaluate(&request_b, &context),
        Decision::Deny(DenialReason::ApprovalScopeMismatch)
    );
}
