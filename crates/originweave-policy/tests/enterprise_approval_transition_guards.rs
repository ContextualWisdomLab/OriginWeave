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
    ApprovalPrincipalRef::new("https://id.example", subject)
        .expect("test principal must be valid")
}

#[test]
fn deny_expires_at_deadline_and_then_rejects_further_transitions() {
    let mut request = EnterpriseApprovalRequest::new(
        approval_scope(),
        principal("maker"),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");
    let checker = principal("checker");

    assert_eq!(
        request.deny(checker.clone(), 200),
        Err(ApprovalLifecycleError::Expired)
    );
    assert_eq!(request.state(), ApprovalLifecycleState::Expired);
    assert_eq!(
        request.deny(checker, 199),
        Err(ApprovalLifecycleError::InvalidState(
            ApprovalLifecycleState::Expired
        ))
    );
}

#[test]
fn withdraw_expires_at_deadline_and_then_rejects_further_transitions() {
    let maker = principal("maker");
    let mut request = EnterpriseApprovalRequest::new(
        approval_scope(),
        maker.clone(),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");

    assert_eq!(
        request.withdraw(&maker, 200),
        Err(ApprovalLifecycleError::Expired)
    );
    assert_eq!(request.state(), ApprovalLifecycleState::Expired);
    assert_eq!(
        request.withdraw(&maker, 199),
        Err(ApprovalLifecycleError::InvalidState(
            ApprovalLifecycleState::Expired
        ))
    );
}

#[test]
fn revoke_expires_at_deadline_and_then_rejects_further_transitions() {
    let checker = principal("checker");
    let mut request = EnterpriseApprovalRequest::new(
        approval_scope(),
        principal("maker"),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");
    request
        .approve(checker.clone(), 150)
        .expect("approval before deadline must succeed");

    assert_eq!(
        request.revoke(&checker, 200),
        Err(ApprovalLifecycleError::Expired)
    );
    assert_eq!(request.state(), ApprovalLifecycleState::Expired);
    assert_eq!(
        request.revoke(&checker, 199),
        Err(ApprovalLifecycleError::InvalidState(
            ApprovalLifecycleState::Expired
        ))
    );
}
