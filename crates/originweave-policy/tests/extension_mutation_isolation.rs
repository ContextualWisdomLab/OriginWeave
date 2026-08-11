#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserSessionId,
    BrowsingContextId, Capability, ExecutionPurpose, ExtensionAccessDecision,
    ExtensionAccessRequest, ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId,
    InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery, SessionMode,
    evaluate_extension_access,
};
use originweave_policy::{Decision, DenialReason, evaluate};

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

#[test]
fn explicit_extension_grant_cannot_authorize_cross_origin_mutation() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let source = origin("https://source.example");
    let target = origin("https://target.example");
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Submit]),
        BTreeSet::from([source.clone(), target.clone()]),
        BTreeSet::from([target.clone()]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Submit,
        source,
        target,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::CrossOriginMutation)
    );
}

#[test]
fn explicit_extension_grant_cannot_supply_missing_write_origin_authority() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin("https://app.example");
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Submit]),
        BTreeSet::from([site.clone()]),
        BTreeSet::new(),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Submit,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::OriginNotWritable)
    );
}

#[test]
fn explicit_extension_grant_cannot_turn_crawler_mode_into_mutation_authority() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin("https://public.example");
    let context = PolicyContext::new(
        SessionMode::Crawler,
        ExecutionPurpose::PublicCrawl,
        BTreeSet::from([Capability::Submit]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site.clone()]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Submit,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::CrawlerMutation)
    );
}

#[test]
fn explicit_extension_grant_cannot_pair_agent_task_with_public_crawl_purpose() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin("https://public.example");
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::PublicCrawl,
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
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::ModePurposeMismatch)
    );
}

#[test]
fn explicit_extension_grant_cannot_bypass_disallowed_robots_policy() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin("https://public.example");
    let context = PolicyContext::new(
        SessionMode::Crawler,
        ExecutionPurpose::PublicCrawl,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([site.clone()]),
        BTreeSet::new(),
        RobotsDecision::Disallowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::RobotsDisallowed)
    );
}

#[test]
fn explicit_extension_grant_cannot_bypass_unknown_robots_policy() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin("https://public.example");
    let context = PolicyContext::new(
        SessionMode::Crawler,
        ExecutionPurpose::PublicCrawl,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([site.clone()]),
        BTreeSet::new(),
        RobotsDecision::Unknown,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::RobotsUnknown)
    );
}

#[test]
fn explicit_extension_grant_cannot_bypass_missing_robots_policy() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin("https://public.example");
    let context = PolicyContext::new(
        SessionMode::Crawler,
        ExecutionPurpose::PublicCrawl,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([site.clone()]),
        BTreeSet::new(),
        RobotsDecision::NotApplicable,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::RobotsNotApplicable)
    );
}

#[test]
fn explicit_extension_grant_cannot_delegate_forbidden_r5_action() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin("https://consent.example");
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::LegalConsent]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site.clone()]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let proposed = ActionRequest::new(
        ActionKind::LegalConsent,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    );

    assert_eq!(
        evaluate(&proposed, &context),
        Decision::Deny(DenialReason::ForbiddenRisk)
    );
}

#[test]
fn explicit_extension_grant_cannot_turn_human_mode_into_agent_control() {
    let grant = action_proposal_grant();
    assert_extension_can_propose(&grant);

    let site = origin("https://human.example");
    let context = PolicyContext::new(
        SessionMode::Human,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Navigate]),
        BTreeSet::from([site.clone()]),
        BTreeSet::new(),
        RobotsDecision::NotApplicable,
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
        Decision::Deny(DenialReason::HumanModeNotAgentControlled)
    );
}
