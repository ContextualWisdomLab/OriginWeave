#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, AgentTaskId, ApprovalEvidence, BrowserSessionId,
    BrowsingContextId, Capability, ExecutionPurpose, ExtensionAccessDecision,
    ExtensionAccessRequest, ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId,
    InstructionSource, Origin, PolicyContext, RiskClass, RobotsDecision, SecretDelivery,
    SessionMode, evaluate_extension_access,
};
use originweave_policy::{Decision, evaluate};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
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

fn origin() -> Origin {
    Origin::parse("https://login.example").expect("valid test origin")
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
        origin(),
        UNEXPIRED_EXPIRES_AT_EPOCH_SECONDS,
        [ExtensionAgentCapability::ProposeTypedAction],
    )
}

fn assert_extension_can_propose(grant: &ExtensionAgentGrant) {
    let request = ExtensionAccessRequest::new(
        extension_id(),
        agent_task(),
        browser_session(),
        browsing_context(),
        origin(),
        UNEXPIRED_NOW_EPOCH_SECONDS,
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
