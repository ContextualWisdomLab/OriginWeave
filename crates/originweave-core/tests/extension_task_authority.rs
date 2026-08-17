#![allow(clippy::expect_used)]

use originweave_core::{
    AgentTaskId, BrowserSessionId, BrowsingContextId, ExtensionAccessDecision,
    ExtensionAccessRequest, ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId, Origin,
    evaluate_extension_access,
};

fn extension_id(value: &str) -> ExtensionId {
    ExtensionId::parse(value).expect("valid extension id")
}

fn task(value: u64) -> AgentTaskId {
    AgentTaskId::new(value).expect("nonzero Agent Task identity")
}

fn session(value: u64) -> BrowserSessionId {
    BrowserSessionId::new(value).expect("nonzero browser session")
}

fn context(value: u64) -> BrowsingContextId {
    BrowsingContextId::new(value).expect("nonzero browsing context")
}

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("canonical origin")
}

#[test]
fn agent_task_identity_rejects_zero() {
    assert!(AgentTaskId::new(0).is_err());
    assert_eq!(task(29).value(), 29);
}

#[test]
fn extension_agent_grants_are_non_transferable_between_agent_tasks() {
    let id = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let granted_origin = origin("https://agent.example");
    let grant = ExtensionAgentGrant::new(
        id.clone(),
        task(29),
        session(7),
        context(11),
        granted_origin.clone(),
        1_700_000_600,
        [ExtensionAgentCapability::ProposeTypedAction],
    );

    let exact_task = ExtensionAccessRequest::new(
        id.clone(),
        task(29),
        session(7),
        context(11),
        granted_origin.clone(),
        1_700_000_000,
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&exact_task, Some(&grant)),
        ExtensionAccessDecision::Allow
    );

    let other_task = ExtensionAccessRequest::new(
        id,
        task(30),
        session(7),
        context(11),
        granted_origin,
        1_700_000_000,
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&other_task, Some(&grant)),
        ExtensionAccessDecision::DenyAgentTaskMismatch
    );
}
