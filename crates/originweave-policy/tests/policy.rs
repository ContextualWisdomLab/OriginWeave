#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionKind, ActionRequest, ApprovalEvidence, ApprovalScope, Capability,
    ExecutionPurpose, InstructionSource, Origin, PolicyContext, RiskClass, RobotsDecision,
    SecretDelivery, SessionMode,
};
use originweave_policy::{Decision, DenialReason, evaluate};

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("valid test origin")
}

fn context(
    mode: SessionMode,
    purpose: ExecutionPurpose,
    capability: Capability,
    source: &Origin,
    target: &Origin,
) -> PolicyContext {
    PolicyContext::new(
        mode,
        purpose,
        BTreeSet::from([capability]),
        BTreeSet::from([source.clone(), target.clone()]),
        BTreeSet::from([source.clone(), target.clone()]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    )
}

fn request(action: ActionKind, source: &Origin, target: &Origin) -> ActionRequest {
    ActionRequest::new(
        action,
        source.clone(),
        target.clone(),
        InstructionSource::User,
        SecretDelivery::None,
    )
}

#[test]
fn policy_allows_low_risk_read_and_draft_work() {
    let site = origin("https://calendar.example");

    for action in [ActionKind::Observe, ActionKind::Navigate, ActionKind::Draft] {
        let ctx = context(
            SessionMode::Assist,
            ExecutionPurpose::UserDelegatedTask,
            action.required_capability(),
            &site,
            &site,
        );
        assert_eq!(evaluate(&request(action, &site, &site), &ctx), Decision::Allow);
    }
}

#[test]
fn policy_rejects_non_agent_and_untrusted_instruction_sources() {
    let site = origin("https://example.com");
    let human = context(
        SessionMode::Human,
        ExecutionPurpose::UserDelegatedTask,
        Capability::Observe,
        &site,
        &site,
    );
    assert_eq!(
        evaluate(&request(ActionKind::Observe, &site, &site), &human),
        Decision::Deny(DenialReason::HumanModeNotAgentControlled)
    );

    let agent = context(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        Capability::Observe,
        &site,
        &site,
    );
    let injected = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site.clone(),
        InstructionSource::WebContent,
        SecretDelivery::None,
    );
    assert_eq!(
        evaluate(&injected, &agent),
        Decision::Deny(DenialReason::UntrustedInstructionSource)
    );
}

#[test]
fn policy_enforces_capabilities_and_origin_grants() {
    let source = origin("https://app.example");
    let target = origin("https://api.example");
    let base = context(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        Capability::Observe,
        &source,
        &target,
    );
    assert_eq!(
        evaluate(&request(ActionKind::Navigate, &source, &target), &base),
        Decision::Deny(DenialReason::MissingCapability(Capability::Navigate))
    );

    let unreadable = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([source.clone()]),
        BTreeSet::new(),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    assert_eq!(
        evaluate(&request(ActionKind::Observe, &source, &target), &unreadable),
        Decision::Deny(DenialReason::OriginNotReadable)
    );

    let cross_origin = context(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        Capability::Submit,
        &source,
        &target,
    );
    assert_eq!(
        evaluate(&request(ActionKind::Submit, &source, &target), &cross_origin),
        Decision::Deny(DenialReason::CrossOriginMutation)
    );

    let unwritable = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Submit]),
        BTreeSet::from([source.clone()]),
        BTreeSet::new(),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    assert_eq!(
        evaluate(&request(ActionKind::Submit, &source, &source), &unwritable),
        Decision::Deny(DenialReason::OriginNotWritable)
    );
}

#[test]
fn crawler_mode_is_read_only_and_requires_explicit_robots_permission() {
    let site = origin("https://catalog.example");
    let mut ctx = context(
        SessionMode::Crawler,
        ExecutionPurpose::PublicCrawl,
        Capability::Extract,
        &site,
        &site,
    );
    assert_eq!(
        evaluate(&request(ActionKind::Extract, &site, &site), &ctx),
        Decision::Allow
    );

    ctx.set_robots_decision(RobotsDecision::Disallowed);
    assert_eq!(
        evaluate(&request(ActionKind::Extract, &site, &site), &ctx),
        Decision::Deny(DenialReason::RobotsDisallowed)
    );
    ctx.set_robots_decision(RobotsDecision::Unknown);
    assert_eq!(
        evaluate(&request(ActionKind::Extract, &site, &site), &ctx),
        Decision::Deny(DenialReason::RobotsUnknown)
    );
    ctx.set_robots_decision(RobotsDecision::NotApplicable);
    assert_eq!(
        evaluate(&request(ActionKind::Extract, &site, &site), &ctx),
        Decision::Deny(DenialReason::RobotsNotApplicable)
    );

    let mutation = context(
        SessionMode::Crawler,
        ExecutionPurpose::PublicCrawl,
        Capability::Draft,
        &site,
        &site,
    );
    assert_eq!(
        evaluate(&request(ActionKind::Draft, &site, &site), &mutation),
        Decision::Deny(DenialReason::CrawlerMutation)
    );
}

#[test]
fn secret_fill_requires_a_broker_and_other_actions_reject_secret_material() {
    let site = origin("https://login.example");
    let ctx = context(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        Capability::FillSecret,
        &site,
        &site,
    );
    let no_broker = request(ActionKind::FillSecret, &site, &site);
    assert_eq!(
        evaluate(&no_broker, &ctx),
        Decision::Deny(DenialReason::SecretBrokerRequired)
    );
    let raw = ActionRequest::new(
        ActionKind::FillSecret,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::RawValue,
    );
    assert_eq!(
        evaluate(&raw, &ctx),
        Decision::Deny(DenialReason::SecretBrokerRequired)
    );

    let brokered = ActionRequest::new(
        ActionKind::FillSecret,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::BrokerHandle,
    );
    assert_eq!(
        evaluate(&brokered, &ctx),
        Decision::RequireApproval(RiskClass::R3)
    );

    let observe_ctx = context(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        Capability::Observe,
        &site,
        &site,
    );
    let secret_on_read = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::BrokerHandle,
    );
    assert_eq!(
        evaluate(&secret_on_read, &observe_ctx),
        Decision::Deny(DenialReason::UnexpectedSecretMaterial)
    );
}

#[test]
fn high_risk_actions_require_exact_approval_and_legal_consent_is_forbidden() {
    let site = origin("https://shop.example");
    let purchase_request = request(ActionKind::Purchase, &site, &site);
    let mut ctx = context(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        Capability::Purchase,
        &site,
        &site,
    );
    assert_eq!(
        evaluate(&purchase_request, &ctx),
        Decision::RequireApproval(RiskClass::R4)
    );

    let wrong_scope = ApprovalScope::new(ActionKind::Delete, site.clone());
    ctx.set_approval(ApprovalEvidence::UserConfirmed(wrong_scope));
    assert_eq!(
        evaluate(&purchase_request, &ctx),
        Decision::Deny(DenialReason::ApprovalScopeMismatch)
    );

    let exact_scope = ApprovalScope::new(ActionKind::Purchase, site.clone());
    ctx.set_approval(ApprovalEvidence::UserConfirmed(exact_scope.clone()));
    assert_eq!(evaluate(&purchase_request, &ctx), Decision::Allow);
    ctx.set_approval(ApprovalEvidence::EnterprisePolicy(exact_scope));
    assert_eq!(evaluate(&purchase_request, &ctx), Decision::Allow);

    let legal_ctx = context(
        SessionMode::AgentTask,
        ExecutionPurpose::EnterpriseAuthorizedTask,
        Capability::LegalConsent,
        &site,
        &site,
    );
    assert_eq!(
        evaluate(&request(ActionKind::LegalConsent, &site, &site), &legal_ctx),
        Decision::Deny(DenialReason::ForbiddenRisk)
    );
}

#[test]
fn enterprise_policy_instruction_is_trusted_but_still_governed() {
    let site = origin("https://admin.example");
    let ctx = context(
        SessionMode::AgentTask,
        ExecutionPurpose::EnterpriseAuthorizedTask,
        Capability::Observe,
        &site,
        &site,
    );
    let req = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site,
        InstructionSource::EnterprisePolicy,
        SecretDelivery::None,
    );
    assert_eq!(evaluate(&req, &ctx), Decision::Allow);
}
