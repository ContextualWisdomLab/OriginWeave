#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserSessionId,
    BrowsingContextId, Capability, ExecutionPurpose, ExtensionAccessDecision,
    ExtensionAccessRequest, ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId,
    InstructionSource, Origin, PolicyContext, RiskClass, RobotsDecision, SecretDelivery,
    SessionMode, evaluate_extension_access,
};
use originweave_policy::{Decision, DenialReason, evaluate};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn extension_id() -> ExtensionId {
    ExtensionId::parse("abcdefghijklmnopabcdefghijklmnop").expect("valid extension id")
}

fn browser_session() -> BrowserSessionId {
    BrowserSessionId::new(7).expect("nonzero browser session")
}

fn browsing_context() -> BrowsingContextId {
    BrowsingContextId::new(11).expect("nonzero browsing context")
}

fn origin() -> Origin {
    Origin::parse("https://login.example").expect("valid test origin")
}

fn intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(VALID_INTENT).expect("valid intent digest")
}

fn action_proposal_grant() -> ExtensionAgentGrant {
    ExtensionAgentGrant::new(
        extension_id(),
        browser_session(),
        browsing_context(),
        [ExtensionAgentCapability::ProposeTypedAction],
    )
}

fn assert_extension_can_propose(grant: &ExtensionAgentGrant) {
    let request = ExtensionAccessRequest::new(
        extension_id(),
        browser_session(),
        browsing_context(),
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&request, Some(grant)),
        ExtensionAccessDecision::Allow
    );
}

fn secret_context(site: &Origin) -> PolicyContext {
    PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::FillSecret]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site.clone()]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    )
}

#[test]
fn extension_action_grant_cannot_turn_raw_secret_delivery_into_authority() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin();
    let proposed = ActionRequest::new(
        ActionKind::FillSecret,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::RawValue,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &secret_context(&site)),
        Decision::Deny(DenialReason::SecretBrokerRequired)
    );
}

#[test]
fn extension_action_grant_cannot_skip_secret_broker_approval() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin();
    let proposed = ActionRequest::new(
        ActionKind::FillSecret,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::BrokerHandle,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &secret_context(&site)),
        Decision::RequireApproval(RiskClass::R3)
    );
}

#[test]
fn extension_action_grant_cannot_attach_broker_material_to_nonsecret_actions() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin();
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([site.clone()]),
        BTreeSet::new(),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::BrokerHandle,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::UnexpectedSecretMaterial)
    );
}
