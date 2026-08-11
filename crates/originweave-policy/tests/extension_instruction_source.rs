#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, Capability, ExecutionPurpose,
    InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery, SessionMode,
};
use originweave_policy::{Decision, DenialReason, evaluate};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn extension_produced_instruction_is_untrusted_even_for_allowed_observation() {
    let site = Origin::parse("https://example.com").expect("valid test origin");
    let request = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site.clone(),
        InstructionSource::Extension,
        SecretDelivery::None,
        ActionIntentDigest::parse(VALID_INTENT).expect("valid intent digest"),
    );
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );

    assert_eq!(
        evaluate(&request, &context),
        Decision::Deny(DenialReason::UntrustedInstructionSource)
    );
}
