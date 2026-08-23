#![allow(clippy::expect_used)]

use originweave_core::{ActionIntentDigest, ActionKind, ApprovalScope, Origin};
use originweave_policy::{
    ApprovalLifecycleError, ApprovalLifecycleState, ApprovalPrincipalRef, EnterpriseApprovalRequest,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn approval_scope() -> ApprovalScope {
    ApprovalScope::new(
        ActionKind::ManagePermission,
        Origin::parse("https://app.example").expect("test origin must be valid"),
        ActionIntentDigest::parse(VALID_INTENT).expect("test intent digest must be valid"),
    )
}

fn principal(subject: &str) -> ApprovalPrincipalRef {
    ApprovalPrincipalRef::new("https://id.example", subject).expect("test principal must be valid")
}

#[test]
fn approval_cannot_predate_the_request_creation_time() {
    let mut request =
        EnterpriseApprovalRequest::new(approval_scope(), principal("maker"), 100, 200, 1)
            .expect("approval request must be valid");

    let error = request
        .approve(principal("checker"), 99)
        .expect_err("approval before request creation must fail closed");
    assert_eq!(error, ApprovalLifecycleError::NonMonotonicTime);
    assert_eq!(error.to_string(), "approval transition time moved backward");
    assert_eq!(request.state(), ApprovalLifecycleState::ApprovalRequested);
    assert_eq!(request.decision_actor(), None);
}

#[test]
fn approved_use_cannot_move_trusted_lifecycle_time_backward() {
    let scope = approval_scope();
    let mut request =
        EnterpriseApprovalRequest::new(scope.clone(), principal("maker"), 100, 200, 2)
            .expect("approval request must be valid");
    request
        .approve(principal("checker"), 150)
        .expect("approval at monotonic trusted time must succeed");

    assert_eq!(
        request.consume(&scope, 149),
        Err(ApprovalLifecycleError::NonMonotonicTime)
    );
    assert_eq!(request.state(), ApprovalLifecycleState::Approved);
    assert_eq!(request.uses_consumed(), 0);
}

#[test]
fn approved_revocation_cannot_move_trusted_lifecycle_time_backward() {
    let checker = principal("checker");
    let mut request =
        EnterpriseApprovalRequest::new(approval_scope(), principal("maker"), 100, 200, 1)
            .expect("approval request must be valid");
    request
        .approve(checker.clone(), 150)
        .expect("approval at monotonic trusted time must succeed");

    assert_eq!(
        request.revoke(&checker, 149),
        Err(ApprovalLifecycleError::NonMonotonicTime)
    );
    assert_eq!(request.state(), ApprovalLifecycleState::Approved);
}
