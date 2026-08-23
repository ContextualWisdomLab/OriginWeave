#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    ActionIntentDigest, ActionKind, ApprovalEvidence, ApprovalLifecycleError,
    ApprovalLifecycleState, ApprovalPrincipalRef, ApprovalPrincipalRefError, ApprovalScope,
    EnterpriseApprovalRequest, Origin,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn approval_scope(action: ActionKind) -> ApprovalScope {
    ApprovalScope::new(
        action,
        Origin::parse("https://app.example").expect("test origin must be valid"),
        ActionIntentDigest::parse(VALID_INTENT).expect("test intent digest must be valid"),
    )
}

fn principal(issuer: &str, subject: &str) -> ApprovalPrincipalRef {
    ApprovalPrincipalRef::new(issuer, subject).expect("test principal must be valid")
}

#[test]
fn principal_identity_is_exact_issuer_subject_tuple() {
    let first = principal("https://id.example", "user-123");
    let same = principal("https://id.example", "user-123");
    let other_issuer = principal("https://other-id.example", "user-123");

    assert_eq!(first, same);
    assert_ne!(first, other_issuer);
    assert_eq!(first.issuer(), "https://id.example");
    assert_eq!(first.subject(), "user-123");
}

#[test]
fn principal_rejects_empty_ambiguous_or_oversized_references() {
    assert_eq!(
        ApprovalPrincipalRef::new("", "user-123"),
        Err(ApprovalPrincipalRefError::InvalidIssuer)
    );
    assert_eq!(
        ApprovalPrincipalRef::new(" https://id.example", "user-123"),
        Err(ApprovalPrincipalRefError::InvalidIssuer)
    );
    assert_eq!(
        ApprovalPrincipalRef::new("https://id.example", "user\n123"),
        Err(ApprovalPrincipalRefError::InvalidSubject)
    );
    assert_eq!(
        ApprovalPrincipalRef::new("https://id.example", &"x".repeat(257)),
        Err(ApprovalPrincipalRefError::InvalidSubject)
    );
}

#[test]
fn constructor_rejects_invalid_lifetime_use_limit_and_non_delegable_consent() {
    let requester = principal("https://id.example", "maker");
    let scope = approval_scope(ActionKind::Purchase);

    assert_eq!(
        EnterpriseApprovalRequest::new(scope.clone(), requester.clone(), 100, 100, 1),
        Err(ApprovalLifecycleError::InvalidValidityWindow)
    );
    assert_eq!(
        EnterpriseApprovalRequest::new(scope, requester.clone(), 100, 200, 0),
        Err(ApprovalLifecycleError::InvalidUseLimit)
    );
    assert_eq!(
        EnterpriseApprovalRequest::new(
            approval_scope(ActionKind::LegalConsent),
            requester,
            100,
            200,
            1,
        ),
        Err(ApprovalLifecycleError::NonDelegableAction)
    );
}

#[test]
fn distinct_checker_approves_exact_intent_and_single_use_consumes_it() {
    let requester = principal("https://id.example", "maker");
    let checker = principal("https://id.example", "checker");
    let scope = approval_scope(ActionKind::Purchase);
    let mut request = EnterpriseApprovalRequest::new(scope.clone(), requester.clone(), 100, 200, 1)
        .expect("approval request must be valid");

    assert_eq!(request.state(), ApprovalLifecycleState::ApprovalRequested);
    assert_eq!(request.scope(), &scope);
    assert_eq!(request.requester(), &requester);
    assert_eq!(request.requested_at_epoch_seconds(), 100);
    assert_eq!(request.expires_at_epoch_seconds(), 200);
    assert_eq!(request.max_uses(), 1);
    assert_eq!(request.uses_consumed(), 0);
    assert_eq!(request.decision_actor(), None);

    request
        .approve(checker.clone(), 110)
        .expect("distinct checker must be able to approve");
    assert_eq!(request.state(), ApprovalLifecycleState::Approved);
    assert_eq!(request.decision_actor(), Some(&checker));

    let evidence = request
        .consume(&scope, 120)
        .expect("approved exact scope must be consumable");
    assert_eq!(evidence, ApprovalEvidence::UserConfirmed(scope));
    assert_eq!(request.uses_consumed(), 1);
    assert_eq!(request.state(), ApprovalLifecycleState::Consumed);
}

#[test]
fn maker_checker_rejects_self_approval_without_mutation() {
    let maker = principal("https://id.example", "maker");
    let mut request = EnterpriseApprovalRequest::new(
        approval_scope(ActionKind::Delete),
        maker.clone(),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");

    assert_eq!(
        request.approve(maker, 110),
        Err(ApprovalLifecycleError::SelfApproval)
    );
    assert_eq!(request.state(), ApprovalLifecycleState::ApprovalRequested);
    assert_eq!(request.decision_actor(), None);
}

#[test]
fn scope_mutation_fails_closed_without_consuming_approval() {
    let mut request = EnterpriseApprovalRequest::new(
        approval_scope(ActionKind::Purchase),
        principal("https://id.example", "maker"),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");
    request
        .approve(principal("https://id.example", "checker"), 110)
        .expect("approval must succeed");
    let mutated_scope = ApprovalScope::new(
        ActionKind::Purchase,
        Origin::parse("https://other.example").expect("test origin must be valid"),
        ActionIntentDigest::parse(VALID_INTENT).expect("test digest must be valid"),
    );

    assert_eq!(
        request.consume(&mutated_scope, 120),
        Err(ApprovalLifecycleError::ScopeMismatch)
    );
    assert_eq!(request.uses_consumed(), 0);
    assert_eq!(request.state(), ApprovalLifecycleState::Approved);
}

#[test]
fn expiry_is_strict_and_transitions_fail_closed_at_deadline() {
    let checker = principal("https://id.example", "checker");
    let scope = approval_scope(ActionKind::Submit);
    let mut not_yet_approved = EnterpriseApprovalRequest::new(
        scope.clone(),
        principal("https://id.example", "maker-a"),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");

    assert_eq!(
        not_yet_approved.approve(checker.clone(), 200),
        Err(ApprovalLifecycleError::Expired)
    );
    assert_eq!(not_yet_approved.state(), ApprovalLifecycleState::Expired);

    let mut approved = EnterpriseApprovalRequest::new(
        scope.clone(),
        principal("https://id.example", "maker-b"),
        100,
        200,
        1,
    )
    .expect("approval request must be valid");
    approved
        .approve(checker, 150)
        .expect("approval before deadline must succeed");

    assert_eq!(
        approved.consume(&scope, 200),
        Err(ApprovalLifecycleError::Expired)
    );
    assert_eq!(approved.state(), ApprovalLifecycleState::Expired);
    assert_eq!(approved.uses_consumed(), 0);
}

#[test]
fn bounded_multi_use_approval_consumes_exactly_the_configured_count() {
    let scope = approval_scope(ActionKind::Upload);
    let mut request = EnterpriseApprovalRequest::new(
        scope.clone(),
        principal("https://id.example", "maker"),
        100,
        300,
        2,
    )
    .expect("approval request must be valid");
    request
        .approve(principal("https://id.example", "checker"), 110)
        .expect("approval must succeed");

    assert!(matches!(
        request.consume(&scope, 120),
        Ok(ApprovalEvidence::UserConfirmed(_))
    ));
    assert_eq!(request.state(), ApprovalLifecycleState::Approved);
    assert_eq!(request.uses_consumed(), 1);
    assert!(matches!(
        request.consume(&scope, 130),
        Ok(ApprovalEvidence::UserConfirmed(_))
    ));
    assert_eq!(request.state(), ApprovalLifecycleState::Consumed);
    assert_eq!(request.uses_consumed(), 2);
    assert_eq!(
        request.consume(&scope, 140),
        Err(ApprovalLifecycleError::InvalidState(
            ApprovalLifecycleState::Consumed
        ))
    );
}

#[test]
fn denial_withdrawal_and_revocation_are_terminal_and_role_bound() {
    let maker = principal("https://id.example", "maker");
    let checker = principal("https://id.example", "checker");
    let stranger = principal("https://id.example", "stranger");
    let scope = approval_scope(ActionKind::ManagePermission);

    let mut denied = EnterpriseApprovalRequest::new(scope.clone(), maker.clone(), 100, 300, 1)
        .expect("approval request must be valid");
    assert_eq!(
        denied.deny(maker.clone(), 110),
        Err(ApprovalLifecycleError::SelfApproval)
    );
    denied
        .deny(checker.clone(), 110)
        .expect("distinct checker must be able to deny");
    assert_eq!(denied.state(), ApprovalLifecycleState::Denied);
    assert_eq!(denied.decision_actor(), Some(&checker));
    assert_eq!(
        denied.approve(checker.clone(), 120),
        Err(ApprovalLifecycleError::InvalidState(
            ApprovalLifecycleState::Denied
        ))
    );

    let mut withdrawn = EnterpriseApprovalRequest::new(scope.clone(), maker.clone(), 100, 300, 1)
        .expect("approval request must be valid");
    assert_eq!(
        withdrawn.withdraw(&stranger, 110),
        Err(ApprovalLifecycleError::RequesterMismatch)
    );
    withdrawn
        .withdraw(&maker, 110)
        .expect("requester must be able to withdraw pending request");
    assert_eq!(withdrawn.state(), ApprovalLifecycleState::Withdrawn);

    let mut revoked = EnterpriseApprovalRequest::new(scope.clone(), maker, 100, 300, 1)
        .expect("approval request must be valid");
    revoked
        .approve(checker.clone(), 110)
        .expect("approval must succeed");
    assert_eq!(
        revoked.revoke(&stranger, 120),
        Err(ApprovalLifecycleError::DecisionActorMismatch)
    );
    assert_eq!(revoked.state(), ApprovalLifecycleState::Approved);
    revoked
        .revoke(&checker, 120)
        .expect("approving checker must be able to revoke");
    assert_eq!(revoked.state(), ApprovalLifecycleState::Revoked);
    assert_eq!(
        revoked.consume(&scope, 130),
        Err(ApprovalLifecycleError::InvalidState(
            ApprovalLifecycleState::Revoked
        ))
    );
}

#[test]
fn lifecycle_errors_have_stable_display_and_no_hidden_sources() {
    let errors = [
        ApprovalLifecycleError::InvalidValidityWindow,
        ApprovalLifecycleError::InvalidUseLimit,
        ApprovalLifecycleError::NonDelegableAction,
        ApprovalLifecycleError::SelfApproval,
        ApprovalLifecycleError::RequesterMismatch,
        ApprovalLifecycleError::DecisionActorMismatch,
        ApprovalLifecycleError::ScopeMismatch,
        ApprovalLifecycleError::Expired,
        ApprovalLifecycleError::InvalidState(ApprovalLifecycleState::Consumed),
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }
    for error in [
        ApprovalPrincipalRefError::InvalidIssuer,
        ApprovalPrincipalRefError::InvalidSubject,
    ] {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }
}
