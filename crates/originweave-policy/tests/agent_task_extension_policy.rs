#![allow(clippy::expect_used)]

use originweave_core::{
    BrowserSessionId, BrowsingContextId, ExtensionAccessDecision, ExtensionAccessRequest,
    ExtensionAgentCapability, ExtensionId, evaluate_extension_access,
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

#[test]
fn empty_agent_task_extension_policy_denies_every_extension() {
    let policy = AgentTaskExtensionPolicy::new(session(31), [], 10, 20);
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");

    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy, session(31), 15),
        AgentTaskExtensionDecision::DenyNotManaged
    );
}

#[test]
fn managed_agent_task_extension_policy_allows_only_exact_identifiers() {
    let allowed = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let other = extension_id("bcdefghijklmnopabcdefghijklmnopa");
    let policy =
        AgentTaskExtensionPolicy::new(session(31), [allowed.clone(), allowed.clone()], 10, 20);

    assert_eq!(
        evaluate_agent_task_extension(&allowed, &policy, session(31), 10),
        AgentTaskExtensionDecision::AllowManagedExtension
    );
    assert_eq!(
        evaluate_agent_task_extension(&other, &policy, session(31), 19),
        AgentTaskExtensionDecision::DenyNotManaged
    );
}

#[test]
fn managed_agent_task_extension_policy_is_not_reusable_across_sessions() {
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let policy = AgentTaskExtensionPolicy::new(session(31), [extension.clone()], 10, 20);

    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy, session(37), 15),
        AgentTaskExtensionDecision::DenySessionMismatch
    );
    assert_eq!(
        evaluate_agent_task_extension(
            &extension_id("bcdefghijklmnopabcdefghijklmnopa"),
            &policy,
            session(37),
            15,
        ),
        AgentTaskExtensionDecision::DenySessionMismatch
    );
}

#[test]
fn managed_agent_task_extension_policy_fails_closed_outside_its_validity_window() {
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let policy = AgentTaskExtensionPolicy::new(session(31), [extension.clone()], 10, 20);

    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy, session(31), 9),
        AgentTaskExtensionDecision::DenyPolicyNotYetValid
    );
    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy, session(31), 20),
        AgentTaskExtensionDecision::DenyPolicyExpired
    );
    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy, session(31), u64::MAX),
        AgentTaskExtensionDecision::DenyPolicyExpired
    );
}

#[test]
fn invalid_managed_extension_policy_window_fails_closed_before_membership() {
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let reversed = AgentTaskExtensionPolicy::new(session(31), [extension.clone()], 20, 10);
    let empty = AgentTaskExtensionPolicy::new(session(31), [extension.clone()], 20, 20);

    assert_eq!(
        evaluate_agent_task_extension(&extension, &reversed, session(31), 15),
        AgentTaskExtensionDecision::DenyInvalidPolicyWindow
    );
    assert_eq!(
        evaluate_agent_task_extension(&extension, &empty, session(31), 20),
        AgentTaskExtensionDecision::DenyInvalidPolicyWindow
    );
}

#[test]
fn maximum_timestamp_window_remains_half_open_without_overflow() {
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let policy =
        AgentTaskExtensionPolicy::new(session(31), [extension.clone()], u64::MAX - 1, u64::MAX);

    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy, session(31), u64::MAX - 1),
        AgentTaskExtensionDecision::AllowManagedExtension
    );
    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy, session(31), u64::MAX),
        AgentTaskExtensionDecision::DenyPolicyExpired
    );
}

#[test]
fn managed_agent_task_extension_admission_does_not_mint_agent_capability() {
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let policy = AgentTaskExtensionPolicy::new(session(31), [extension.clone()], 10, 20);

    assert_eq!(
        evaluate_agent_task_extension(&extension, &policy, session(31), 15),
        AgentTaskExtensionDecision::AllowManagedExtension
    );

    let request = ExtensionAccessRequest::new(
        extension,
        session(31),
        context(37),
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&request, None),
        ExtensionAccessDecision::DenyMissingGrant
    );
}
