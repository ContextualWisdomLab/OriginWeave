#![allow(clippy::expect_used)]

use originweave_core::{
    BrowserSessionId, BrowsingContextId, ExtensionAccessDecision, ExtensionAccessRequest,
    ExtensionAgentCapability, ExtensionId, Origin, evaluate_extension_access,
};
use originweave_policy::{
    AgentTaskExtensionDecision, AgentTaskExtensionPolicy, evaluate_agent_task_extension,
};

fn extension_id(value: &str) -> ExtensionId {
    ExtensionId::parse(value).expect("valid extension identifier")
}

fn session(value: u64) -> BrowserSessionId {
    BrowserSessionId::new(value).expect("nonzero browser session")
}

fn context(value: u64) -> BrowsingContextId {
    BrowsingContextId::new(value).expect("nonzero browsing context")
}

fn origin() -> Origin {
    Origin::parse("https://app.example").expect("valid controlled origin")
}

#[test]
fn empty_agent_task_extension_policy_denies_every_extension() {
    let policy = AgentTaskExtensionPolicy::new([]);
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");

    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy),
        AgentTaskExtensionDecision::DenyNotManaged
    );
}

#[test]
fn managed_agent_task_extension_policy_allows_only_exact_identifiers() {
    let allowed = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let other = extension_id("bcdefghijklmnopabcdefghijklmnopa");
    let policy = AgentTaskExtensionPolicy::new([allowed.clone(), allowed.clone()]);

    assert_eq!(
        evaluate_agent_task_extension(&allowed, &policy),
        AgentTaskExtensionDecision::AllowManagedExtension
    );
    assert_eq!(
        evaluate_agent_task_extension(&other, &policy),
        AgentTaskExtensionDecision::DenyNotManaged
    );
}

#[test]
fn managed_agent_task_extension_admission_does_not_mint_agent_capability() {
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let policy = AgentTaskExtensionPolicy::new([extension.clone()]);

    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy),
        AgentTaskExtensionDecision::AllowManagedExtension
    );

    let request = ExtensionAccessRequest::new(
        extension,
        session(31),
        context(37),
        origin(),
        100,
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&request, None),
        ExtensionAccessDecision::DenyMissingGrant
    );
}
