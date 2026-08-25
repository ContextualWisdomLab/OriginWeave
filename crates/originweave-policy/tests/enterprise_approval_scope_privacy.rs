#![allow(clippy::expect_used)]

use originweave_core::{ActionIntentDigest, ActionKind, ApprovalScope, Origin};
use originweave_policy::{
    ApprovalLifecycleError, ApprovalLifecycleState, ApprovalPrincipalRef,
    EnterpriseApprovalRequest,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn approval_scope(origin: &str) -> ApprovalScope {
    ApprovalScope::new(
        ActionKind::Purchase,
        Origin::parse(origin).expect("test origin must be valid"),
        ActionIntentDigest::parse(VALID_INTENT).expect("test intent digest must be valid"),
    )
}

fn principal(subject: &str) -> ApprovalPrincipalRef {
    ApprovalPrincipalRef::new("https://id.example", subject).expect("test principal must be valid")
}

#[test]
fn mismatched_scope_at_expiry_does_not_disclose_or_mutate_lifecycle() {
    let authority_scope = approval_scope("https://app.example");
    let foreign_scope = approval_scope("https://other.example");
    let mut request = EnterpriseApprovalRequest::new(
        authority_scope,
        principal("maker"),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");
    request
        .approve(principal("checker"), 110)
        .expect("approval must succeed");

    assert_eq!(
        request.consume(&foreign_scope, 200),
        Err(ApprovalLifecycleError::ScopeMismatch)
    );
    assert_eq!(request.state(), ApprovalLifecycleState::Approved);
    assert_eq!(request.uses_consumed(), 0);
}
