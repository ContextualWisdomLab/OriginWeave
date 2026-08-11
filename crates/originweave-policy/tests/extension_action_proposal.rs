#![allow(clippy::expect_used)]

//! Compose exact extension proposal authority with ordinary typed-action policy.
//!
//! An extension grant can authorize only the right to propose one typed action for independent
//! OriginWeave policy evaluation. It must not manufacture instruction trust, action capability,
//! origin authority, secret authority, approval, or action success.

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserSessionId,
    BrowsingContextId, Capability, ExecutionPurpose, ExtensionAccessDecision,
    ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId, InstructionSource, Origin,
    PolicyContext, RiskClass, RobotsDecision, SecretDelivery, SessionMode,
};
use originweave_policy::{
    Decision, DenialReason, ExtensionProposalDecision, evaluate_extension_action_proposal,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn extension_id() -> ExtensionId {
    ExtensionId::parse("abcdefghijklmnopabcdefghijklmnop").expect("valid extension id")
}

fn browser_session() -> BrowserSessionId {
    BrowserSessionId::new(17).expect("nonzero browser session")
}

fn browsing_context() -> BrowsingContextId {
    BrowsingContextId::new(23).expect("nonzero browsing context")
}

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("valid test origin")
}

fn intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(VALID_INTENT).expect("valid intent digest")
}

fn proposal_grant() -> ExtensionAgentGrant {
    ExtensionAgentGrant::new(
        extension_id(),
        browser_session(),
        browsing_context(),
        [ExtensionAgentCapability::ProposeTypedAction],
    )
}

fn observe_request(source: InstructionSource) -> ActionRequest {
    let site = origin("https://app.example");
    ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site,
        source,
        SecretDelivery::None,
        intent(),
    )
}

fn observe_context() -> PolicyContext {
    PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([origin("https://app.example")]),
        BTreeSet::new(),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    )
}

#[test]
fn missing_extension_grant_stops_before_action_policy() {
    assert_eq!(
        evaluate_extension_action_proposal(
            &extension_id(),
            browser_session(),
            browsing_context(),
            None,
            &observe_request(InstructionSource::User),
            &observe_context(),
        ),
        ExtensionProposalDecision::ExtensionAccessDenied(ExtensionAccessDecision::DenyMissingGrant)
    );
}

#[test]
fn non_proposal_extension_capability_cannot_submit_an_action() {
    let grant = ExtensionAgentGrant::new(
        extension_id(),
        browser_session(),
        browsing_context(),
        [ExtensionAgentCapability::ObserveCurrentContext],
    );

    assert_eq!(
        evaluate_extension_action_proposal(
            &extension_id(),
            browser_session(),
            browsing_context(),
            Some(&grant),
            &observe_request(InstructionSource::User),
            &observe_context(),
        ),
        ExtensionProposalDecision::ExtensionAccessDenied(
            ExtensionAccessDecision::DenyCapabilityNotGranted
        )
    );
}

#[test]
fn exact_proposal_grant_reaches_ordinary_action_policy() {
    assert_eq!(
        evaluate_extension_action_proposal(
            &extension_id(),
            browser_session(),
            browsing_context(),
            Some(&proposal_grant()),
            &observe_request(InstructionSource::User),
            &observe_context(),
        ),
        ExtensionProposalDecision::ActionPolicy(Decision::Allow)
    );
}

#[test]
fn extension_transport_does_not_promote_web_content_to_instruction_authority() {
    assert_eq!(
        evaluate_extension_action_proposal(
            &extension_id(),
            browser_session(),
            browsing_context(),
            Some(&proposal_grant()),
            &observe_request(InstructionSource::WebContent),
            &observe_context(),
        ),
        ExtensionProposalDecision::ActionPolicy(Decision::Deny(
            DenialReason::UntrustedInstructionSource
        ))
    );
}

#[test]
fn extension_proposal_grant_cannot_manufacture_secret_fill_approval() {
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
    let request = ActionRequest::new(
        ActionKind::FillSecret,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::BrokerHandle,
        intent(),
    );

    assert_eq!(
        evaluate_extension_action_proposal(
            &extension_id(),
            browser_session(),
            browsing_context(),
            Some(&proposal_grant()),
            &request,
            &context,
        ),
        ExtensionProposalDecision::ActionPolicy(Decision::RequireApproval(RiskClass::R3))
    );
}
