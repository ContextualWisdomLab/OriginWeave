#![allow(clippy::expect_used)]

//! Break-glass approvals must identify approvers independently from the approved actor.

use originweave_core::Origin;
use originweave_policy::{
    BreakGlassActorBinding, BreakGlassApprovalEvidence, BreakGlassApproverBinding,
    BreakGlassIdentityBindings, BreakGlassValidityPolicy, DataClassification, DisclosureDecision,
    DisclosureScope, SensitiveBreakGlassDecision, SensitiveBreakGlassRequest,
    SensitiveBreakGlassScope, SensitiveDataAuthority, SensitiveDataRequest,
    evaluate_sensitive_break_glass,
};

const VALID_FROM: u64 = 100;
const VALID_UNTIL: u64 = 200;
const TRUSTED_TIME: u64 = 150;
const ACTOR_ID: &str = "support-operator-42";

fn authority() -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        "tenant-alpha",
        "task-42",
        "customer-address",
        "incident-response",
        Origin::parse("https://support-console.example").expect("valid destination origin"),
        DataClassification::SensitivePersonalData,
    )
}

fn evaluate(
    disclosure_decision: DisclosureDecision,
    approval: BreakGlassApprovalEvidence,
    approvers: BreakGlassApproverBinding,
) -> SensitiveBreakGlassDecision {
    let exact_authority = authority();
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope = DisclosureScope::new(exact_authority.clone(), disclosure_decision);
    let request = SensitiveBreakGlassRequest::new(exact_authority.clone(), "incident-ticket-42");
    let scope = SensitiveBreakGlassScope::new(
        exact_authority,
        "incident-ticket-42",
        approval,
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    );
    let identities =
        BreakGlassIdentityBindings::new(BreakGlassActorBinding::new(ACTOR_ID, ACTOR_ID), approvers);
    let validity = BreakGlassValidityPolicy::new(VALID_UNTIL - VALID_FROM);

    evaluate_sensitive_break_glass(
        &disclosure_request,
        &disclosure_scope,
        &request,
        &scope,
        &identities,
        &validity,
        TRUSTED_TIME,
    )
}

#[test]
fn human_break_glass_requires_an_approver_distinct_from_the_approved_actor() {
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            BreakGlassApprovalEvidence::human("approval-human-1"),
            BreakGlassApproverBinding::human("approval-human-1", "support-approver-7"),
        ),
        SensitiveBreakGlassDecision::Authorized
    );

    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            BreakGlassApprovalEvidence::human("approval-human-1"),
            BreakGlassApproverBinding::human("approval-human-1", ACTOR_ID),
        ),
        SensitiveBreakGlassDecision::ApproverIndependenceRequired
    );
}

#[test]
fn dual_control_requires_two_distinct_non_beneficiary_approver_identities() {
    assert_eq!(
        evaluate(
            DisclosureDecision::DualControlRequired,
            BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
            BreakGlassApproverBinding::dual_control(
                "approval-human-1",
                "support-approver-7",
                "approval-human-2",
                "security-approver-9",
            ),
        ),
        SensitiveBreakGlassDecision::Authorized
    );

    assert_eq!(
        evaluate(
            DisclosureDecision::DualControlRequired,
            BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
            BreakGlassApproverBinding::dual_control(
                "approval-human-1",
                "support-approver-7",
                "approval-human-2",
                "support-approver-7",
            ),
        ),
        SensitiveBreakGlassDecision::InvalidApproverBinding
    );

    for approvers in [
        BreakGlassApproverBinding::dual_control(
            "approval-human-1",
            ACTOR_ID,
            "approval-human-2",
            "security-approver-9",
        ),
        BreakGlassApproverBinding::dual_control(
            "approval-human-1",
            "support-approver-7",
            "approval-human-2",
            ACTOR_ID,
        ),
    ] {
        assert_eq!(
            evaluate(
                DisclosureDecision::DualControlRequired,
                BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
                approvers,
            ),
            SensitiveBreakGlassDecision::ApproverIndependenceRequired
        );
    }
}

#[test]
fn approver_identity_is_bound_to_the_exact_approval_reference() {
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            BreakGlassApprovalEvidence::human("approval-human-1"),
            BreakGlassApproverBinding::human("approval-human-other", "support-approver-7"),
        ),
        SensitiveBreakGlassDecision::ApproverBindingMismatch
    );

    assert_eq!(
        evaluate(
            DisclosureDecision::DualControlRequired,
            BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
            BreakGlassApproverBinding::dual_control(
                "approval-human-1",
                "support-approver-7",
                "approval-human-other",
                "security-approver-9",
            ),
        ),
        SensitiveBreakGlassDecision::ApproverBindingMismatch
    );
}

#[test]
fn dual_control_pair_order_does_not_change_reference_to_approver_binding() {
    assert_eq!(
        evaluate(
            DisclosureDecision::DualControlRequired,
            BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
            BreakGlassApproverBinding::dual_control(
                "approval-human-2",
                "security-approver-9",
                "approval-human-1",
                "support-approver-7",
            ),
        ),
        SensitiveBreakGlassDecision::Authorized
    );
}

#[test]
fn malformed_approver_or_bound_approval_reference_fails_closed() {
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            BreakGlassApprovalEvidence::human("approval-human-1"),
            BreakGlassApproverBinding::human("approval-human-1", "approver id with spaces"),
        ),
        SensitiveBreakGlassDecision::InvalidApproverBinding
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            BreakGlassApprovalEvidence::human("approval-human-1"),
            BreakGlassApproverBinding::human("approval id with spaces", "support-approver-7"),
        ),
        SensitiveBreakGlassDecision::InvalidApproverBinding
    );

    for approvers in [
        BreakGlassApproverBinding::dual_control(
            "approval-human-1",
            "approver id with spaces",
            "approval-human-2",
            "security-approver-9",
        ),
        BreakGlassApproverBinding::dual_control(
            "approval-human-1",
            "support-approver-7",
            "approval-human-2",
            "approver id with spaces",
        ),
        BreakGlassApproverBinding::dual_control(
            "approval id with spaces",
            "support-approver-7",
            "approval-human-2",
            "security-approver-9",
        ),
        BreakGlassApproverBinding::dual_control(
            "approval-human-1",
            "support-approver-7",
            "approval id with spaces",
            "security-approver-9",
        ),
    ] {
        assert_eq!(
            evaluate(
                DisclosureDecision::DualControlRequired,
                BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
                approvers,
            ),
            SensitiveBreakGlassDecision::InvalidApproverBinding
        );
    }
}

#[test]
fn duplicate_bound_approval_references_fail_closed() {
    assert_eq!(
        evaluate(
            DisclosureDecision::DualControlRequired,
            BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
            BreakGlassApproverBinding::dual_control(
                "approval-human-1",
                "support-approver-7",
                "approval-human-1",
                "security-approver-9",
            ),
        ),
        SensitiveBreakGlassDecision::InvalidApproverBinding
    );
}

#[test]
fn approver_binding_shape_must_match_the_supplied_approval_evidence() {
    for (decision, approval, approvers) in [
        (
            DisclosureDecision::HumanApprovalRequired,
            BreakGlassApprovalEvidence::human("approval-human-1"),
            BreakGlassApproverBinding::dual_control(
                "approval-human-1",
                "support-approver-7",
                "approval-human-2",
                "security-approver-9",
            ),
        ),
        (
            DisclosureDecision::DualControlRequired,
            BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
            BreakGlassApproverBinding::human("approval-human-1", "support-approver-7"),
        ),
    ] {
        assert_eq!(
            evaluate(decision, approval, approvers),
            SensitiveBreakGlassDecision::ApproverBindingMismatch
        );
    }
}
