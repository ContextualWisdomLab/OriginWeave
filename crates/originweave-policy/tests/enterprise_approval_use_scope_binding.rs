#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, ApprovalScope, Capability,
    ExecutionPurpose, InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery,
    SessionMode,
};
use originweave_policy::{
    ApprovalLifecycleError, ApprovalPrincipalRef, EnterpriseApprovalRequest,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn site() -> Origin {
    Origin::parse("https://shop.example").expect("test origin must be valid")
}

fn intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(VALID_INTENT).expect("test intent digest must be valid")
}

fn purchase_scope() -> ApprovalScope {
    ApprovalScope::new(ActionKind::Purchase, site(), intent())
}

fn principal(subject: &str) -> ApprovalPrincipalRef {
    ApprovalPrincipalRef::new("https://id.example", subject).expect("test principal must be valid")
}

fn observe_request() -> ActionRequest {
    let origin = site();
    ActionRequest::new(
        ActionKind::Observe,
        origin.clone(),
        origin,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    )
}

fn observe_context() -> PolicyContext {
    let origin = site();
    PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([origin.clone()]),
        BTreeSet::from([origin]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    )
}

fn issued_purchase_use(
    consume_at_epoch_seconds: u64,
) -> originweave_policy::EnterpriseApprovalUse {
    let approved_scope = purchase_scope();
    let mut approval = EnterpriseApprovalRequest::new(
        approved_scope.clone(),
        principal("maker"),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");
    approval
        .approve(principal("checker"), 110)
        .expect("distinct checker must approve");
    approval
        .consume(&approved_scope, consume_at_epoch_seconds)
        .expect("approved exact scope must yield one bounded use")
}

#[test]
fn consumed_approval_use_rejects_a_different_low_risk_scope() {
    let approval_use = issued_purchase_use(120);

    assert_eq!(
        approval_use.evaluate_at(&observe_request(), &observe_context(), 130),
        Err(ApprovalLifecycleError::ScopeMismatch)
    );
}

#[test]
fn mismatched_use_scope_is_rejected_before_lifecycle_state_is_disclosed() {
    let approval_use = issued_purchase_use(199);

    assert_eq!(
        approval_use.evaluate_at(&observe_request(), &observe_context(), 200),
        Err(ApprovalLifecycleError::ScopeMismatch)
    );
}
