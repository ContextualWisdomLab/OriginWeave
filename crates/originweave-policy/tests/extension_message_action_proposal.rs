#![allow(clippy::expect_used)]

//! Raw extension messages remain untrusted observations when they propose typed actions.
//!
//! A Chrome/OriginWeave extension grant may authorize the right to propose a typed action, but
//! extension-produced message content cannot select `User` or `EnterprisePolicy` instruction trust.
//! A future trusted adapter that independently authenticates human or managed-policy provenance
//! needs a separate boundary; this raw-message path always enters ordinary policy as web content.

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ApprovalEvidence, BrowserSessionId, BrowsingContextId,
    Capability, ExecutionPurpose, ExtensionAccessDecision, ExtensionAgentCapability,
    ExtensionAgentGrant, ExtensionId, Origin, PolicyContext, RobotsDecision, SecretDelivery,
    SessionMode,
};
use originweave_policy::{
    Decision, DenialReason, ExtensionMessageActionProposal, ExtensionProposalDecision,
    evaluate_extension_message_action_proposal,
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

fn observe_proposal() -> ExtensionMessageActionProposal {
    let site = origin("https://app.example");
    ExtensionMessageActionProposal::new(
        ActionKind::Observe,
        site.clone(),
        site,
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
fn missing_extension_grant_stops_before_raw_message_policy() {
    assert_eq!(
        evaluate_extension_message_action_proposal(
            &extension_id(),
            browser_session(),
            browsing_context(),
            None,
            &observe_proposal(),
            &observe_context(),
        ),
        ExtensionProposalDecision::ExtensionAccessDenied(ExtensionAccessDecision::DenyMissingGrant)
    );
}

#[test]
fn exact_proposal_grant_cannot_promote_raw_extension_message_to_trusted_instruction() {
    assert_eq!(
        evaluate_extension_message_action_proposal(
            &extension_id(),
            browser_session(),
            browsing_context(),
            Some(&proposal_grant()),
            &observe_proposal(),
            &observe_context(),
        ),
        ExtensionProposalDecision::ActionPolicy(Decision::Deny(
            DenialReason::UntrustedInstructionSource
        ))
    );
}

#[test]
fn raw_extension_message_cannot_hide_broker_material_inside_non_secret_action() {
    let site = origin("https://app.example");
    let proposal = ExtensionMessageActionProposal::new(
        ActionKind::Observe,
        site.clone(),
        site,
        SecretDelivery::BrokerHandle,
        intent(),
    );

    // Raw extension message trust fails before later secret/action checks. This ordering prevents
    // the extension transport from probing which additional action-policy state would have matched.
    assert_eq!(
        evaluate_extension_message_action_proposal(
            &extension_id(),
            browser_session(),
            browsing_context(),
            Some(&proposal_grant()),
            &proposal,
            &observe_context(),
        ),
        ExtensionProposalDecision::ActionPolicy(Decision::Deny(
            DenialReason::UntrustedInstructionSource
        ))
    );
}
