#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, ApprovalScope, Capability,
    ExecutionPurpose, InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery,
    SessionMode,
};
use originweave_policy::{
    ApprovalLifecycleError, ApprovalLifecycleState, ApprovalPrincipalRef, Decision, DenialReason,
    EnterpriseApprovalRequest, EnterpriseApprovalUse,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn site() -> Origin {
    Origin::parse("https://shop.example").expect("test origin must be valid")
}

fn intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(VALID_INTENT).expect("test intent digest must be valid")
}

fn scope() -> ApprovalScope {
    ApprovalScope::new(ActionKind::Purchase, site(), intent())
}

fn principal(subject: &str) -> ApprovalPrincipalRef {
    ApprovalPrincipalRef::new("https://id.example", subject).expect("test principal must be valid")
}

fn purchase_request() -> ActionRequest {
    let origin = site();
    ActionRequest::new(
        ActionKind::Purchase,
        origin.clone(),
        origin,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    )
}

fn policy_context(capabilities: BTreeSet<Capability>) -> PolicyContext {
    let origin = site();
    PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        capabilities,
        BTreeSet::from([origin.clone()]),
        BTreeSet::from([origin]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    )
}

#[test]
fn consumed_enterprise_approval_is_one_shot_policy_input() {
    let approval_scope = scope();
    let mut approval =
        EnterpriseApprovalRequest::new(approval_scope.clone(), principal("maker"), 100, 200, 1)
            .expect("approval request must be valid");
    approval
        .approve(principal("checker"), 110)
        .expect("distinct checker must approve");

    let approval_use = approval
        .consume(&approval_scope, 120)
        .expect("approved exact scope must yield one bounded use");
    assert_eq!(approval.state(), ApprovalLifecycleState::Consumed);
    assert_eq!(approval.uses_consumed(), 1);

    let decision = approval_use.evaluate_at(
        &purchase_request(),
        &policy_context(BTreeSet::from([Capability::Purchase])),
        130,
    );
    assert_eq!(decision, Ok(Decision::Allow));
    assert_eq!(
        approval.consume(&approval_scope, 130),
        Err(ApprovalLifecycleError::InvalidState(
            ApprovalLifecycleState::Consumed
        ))
    );
}

#[test]
fn policy_denial_burns_the_already_consumed_approval_use() {
    let approval_scope = scope();
    let mut approval =
        EnterpriseApprovalRequest::new(approval_scope.clone(), principal("maker"), 100, 200, 1)
            .expect("approval request must be valid");
    approval
        .approve(principal("checker"), 110)
        .expect("distinct checker must approve");

    let approval_use = approval
        .consume(&approval_scope, 120)
        .expect("approved exact scope must yield one bounded use");
    assert_eq!(
        approval_use.evaluate_at(&purchase_request(), &policy_context(BTreeSet::new()), 130),
        Ok(Decision::Deny(DenialReason::MissingCapability(
            Capability::Purchase
        )))
    );
    assert_eq!(approval.state(), ApprovalLifecycleState::Consumed);
    assert_eq!(
        approval.consume(&approval_scope, 130),
        Err(ApprovalLifecycleError::InvalidState(
            ApprovalLifecycleState::Consumed
        ))
    );
}

#[test]
fn consumed_approval_use_expires_before_policy_evaluation() {
    let approval_scope = scope();
    let mut approval =
        EnterpriseApprovalRequest::new(approval_scope.clone(), principal("maker"), 100, 200, 1)
            .expect("approval request must be valid");
    approval
        .approve(principal("checker"), 110)
        .expect("distinct checker must approve");

    let approval_use = approval
        .consume(&approval_scope, 199)
        .expect("pre-deadline consumption must succeed");
    assert_eq!(approval.state(), ApprovalLifecycleState::Consumed);
    assert_eq!(
        approval_use.evaluate_at(
            &purchase_request(),
            &policy_context(BTreeSet::from([Capability::Purchase])),
            200,
        ),
        Err(ApprovalLifecycleError::Expired)
    );
}

#[test]
fn consumed_approval_use_rejects_trusted_time_rollback() {
    let approval_scope = scope();
    let mut approval =
        EnterpriseApprovalRequest::new(approval_scope.clone(), principal("maker"), 100, 200, 1)
            .expect("approval request must be valid");
    approval
        .approve(principal("checker"), 110)
        .expect("distinct checker must approve");

    let approval_use = approval
        .consume(&approval_scope, 120)
        .expect("approved exact scope must yield one bounded use");
    assert_eq!(
        approval_use.evaluate_at(
            &purchase_request(),
            &policy_context(BTreeSet::from([Capability::Purchase])),
            119,
        ),
        Err(ApprovalLifecycleError::NonMonotonicTime)
    );
}

#[test]
fn enterprise_approval_use_is_not_cloneable() {
    trait AmbiguousIfClone<A> {
        fn marker() {}
    }

    impl<T: ?Sized> AmbiguousIfClone<()> for T {}

    struct CloneImplemented;
    impl<T: Clone> AmbiguousIfClone<CloneImplemented> for T {}

    let _ = <EnterpriseApprovalUse as AmbiguousIfClone<_>>::marker;
}
