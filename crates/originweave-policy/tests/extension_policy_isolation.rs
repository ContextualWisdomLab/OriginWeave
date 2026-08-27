#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, AgentTaskId, ApprovalEvidence, BrowserSessionId,
    BrowsingContextId, Capability, ExecutionPurpose, ExtensionAccessDecision,
    ExtensionAccessRequest, ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId,
    InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery, SessionMode,
    evaluate_extension_access,
};
use originweave_policy::{Decision, DenialReason, evaluate};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const EXTENSION_ORIGIN: &str = "https://extension.example";
const UNEXPIRED_NOW_EPOCH_SECONDS: u64 = 1_700_000_000;
const UNEXPIRED_EXPIRES_AT_EPOCH_SECONDS: u64 = 1_700_000_600;

fn extension_id() -> ExtensionId {
    ExtensionId::parse("abcdefghijklmnopabcdefghijklmnop").expect("valid extension id")
}

fn agent_task() -> AgentTaskId {
    AgentTaskId::new(5).expect("nonzero agent task")
}

fn browser_session() -> BrowserSessionId {
    BrowserSessionId::new(7).expect("nonzero browser session")
}

fn browsing_context() -> BrowsingContextId {
    BrowsingContextId::new(11).expect("nonzero browsing context")
}

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("valid test origin")
}

fn intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(VALID_INTENT).expect("valid intent digest")
}

fn action_proposal_grant() -> ExtensionAgentGrant {
    ExtensionAgentGrant::new(
        extension_id(),
        agent_task(),
        browser_session(),
        browsing_context(),
        origin(EXTENSION_ORIGIN),
        UNEXPIRED_EXPIRES_AT_EPOCH_SECONDS,
        [ExtensionAgentCapability::ProposeTypedAction],
    )
}

fn assert_extension_can_only_propose(grant: &ExtensionAgentGrant) {
    let request = ExtensionAccessRequest::new(
        extension_id(),
        agent_task(),
        browser_session(),
        browsing_context(),
        origin(EXTENSION_ORIGIN),
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&request, Some(grant)),
        ExtensionAccessDecision::Allow
    );
}

#[test]
fn explicit_extension_grant_does_not_widen_agent_origin_authority() {
    let grant = action_proposal_grant();
    assert_extension_can_only_propose(&grant);

    let allowed = origin("https://app.example");
    let forbidden = origin("https://outside.example");
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Navigate]),
        BTreeSet::from([allowed.clone()]),
        BTreeSet::new(),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Navigate,
        allowed,
        forbidden,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::OriginNotReadable)
    );
}

#[test]
fn explicit_extension_grant_does_not_supply_agent_action_capability() {
    let grant = action_proposal_grant();
    assert_extension_can_only_propose(&grant);

    let site = origin("https://app.example");
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
        ActionKind::Navigate,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::MissingCapability(Capability::Navigate))
    );
}

#[test]
fn untrusted_extension_content_cannot_become_a_policy_instruction() {
    let grant = action_proposal_grant();
    assert_extension_can_only_propose(&grant);

    let site = origin("https://app.example");
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Navigate]),
        BTreeSet::from([site.clone()]),
        BTreeSet::new(),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Navigate,
        site.clone(),
        site,
        InstructionSource::WebContent,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::UntrustedInstructionSource)
    );
}

#[test]
fn explicit_extension_grant_cannot_turn_raw_secret_delivery_into_a_fill_capability() {
    let grant = action_proposal_grant();
    assert_extension_can_only_propose(&grant);

    let site = origin("https://app.example");
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::FillSecret]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site.clone()]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::FillSecret,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::RawValue,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::SecretBrokerRequired)
    );
}

#[test]
fn explicit_extension_grant_cannot_attach_secret_material_to_non_secret_action() {
    let grant = action_proposal_grant();
    assert_extension_can_only_propose(&grant);

    let site = origin("https://app.example");
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Navigate]),
        BTreeSet::from([site.clone()]),
        BTreeSet::new(),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Navigate,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::RawValue,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::UnexpectedSecretMaterial)
    );
}
