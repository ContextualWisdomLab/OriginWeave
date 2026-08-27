#![allow(clippy::expect_used)]

//! Keep extension proposal-grant evaluation separate from ordinary action policy.
//!
//! OriginWeave does not yet implement an adapter that converts an extension proposal into an
//! [`ActionRequest`]. These regressions therefore prove two independent fail-closed boundaries:
//! the exact extension/task/session/context/origin/unexpired grant permits only
//! `ProposeTypedAction`, while an ordinary user-sourced action request remains subject to the
//! core policy decision shown in each test.

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
    AgentTaskId::new(13).expect("nonzero agent task")
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
        agent_task(),
        browser_session(),
        browsing_context(),
        origin(EXTENSION_ORIGIN),
        UNEXPIRED_EXPIRES_AT_EPOCH_SECONDS,
        [ExtensionAgentCapability::ProposeTypedAction],
    )
}

fn assert_proposal_grant_is_independently_allowed(grant: &ExtensionAgentGrant) {
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
fn extension_proposal_grant_is_independent_of_cross_origin_mutation_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
fn extension_proposal_grant_is_independent_of_write_origin_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
fn extension_proposal_grant_is_independent_of_crawler_mutation_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
fn extension_proposal_grant_is_independent_of_mode_purpose_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
fn extension_proposal_grant_is_independent_of_disallowed_robots_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
fn extension_proposal_grant_is_independent_of_unknown_robots_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
fn extension_proposal_grant_is_independent_of_missing_robots_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
fn extension_proposal_grant_is_independent_of_non_delegable_r5_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
fn extension_proposal_grant_is_independent_of_human_mode_policy() {
    let grant = action_proposal_grant();
    assert_proposal_grant_is_independently_allowed(&grant);

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
