#![allow(clippy::expect_used)]

//! Fail-closed policy contract for exceptional sensitive-data break-glass access.
//!
//! Break-glass does not turn a denied or ordinarily authorized disclosure into a new authority.
//! It may authorize only an existing human-approval or dual-control sensitive-data decision after
//! exact authority/reason binding, bounded freshness, explicit approval, heightened monitoring,
//! and mandatory post-event review all succeed. The values below carry policy metadata only and
//! never contain protected field bytes.

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
const MAXIMUM_WINDOW: u64 = VALID_UNTIL - VALID_FROM;

fn authority(task_id: &str) -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        "tenant-alpha",
        task_id,
        "customer-address",
        "incident-response",
        Origin::parse("https://support-console.example").expect("valid destination origin"),
        DataClassification::SensitivePersonalData,
    )
}

fn default_approvers(disclosure_decision: DisclosureDecision) -> BreakGlassApproverBinding {
    if disclosure_decision == DisclosureDecision::DualControlRequired {
        BreakGlassApproverBinding::dual_control("support-approver-7", "security-approver-9")
    } else {
        BreakGlassApproverBinding::human("support-approver-7")
    }
}

fn evaluate_with_approvers(
    disclosure_decision: DisclosureDecision,
    break_glass_request: SensitiveBreakGlassRequest,
    break_glass_scope: SensitiveBreakGlassScope,
    approver_binding: BreakGlassApproverBinding,
    trusted_time: u64,
) -> SensitiveBreakGlassDecision {
    let exact_authority = authority("task-42");
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope = DisclosureScope::new(exact_authority, disclosure_decision);
    let identity_bindings = BreakGlassIdentityBindings::new(
        BreakGlassActorBinding::new(ACTOR_ID, ACTOR_ID),
        approver_binding,
    );
    let validity_policy = BreakGlassValidityPolicy::new(MAXIMUM_WINDOW);

    evaluate_sensitive_break_glass(
        &disclosure_request,
        &disclosure_scope,
        &break_glass_request,
        &break_glass_scope,
        &identity_bindings,
        &validity_policy,
        trusted_time,
    )
}

fn evaluate(
    disclosure_decision: DisclosureDecision,
    break_glass_request: SensitiveBreakGlassRequest,
    break_glass_scope: SensitiveBreakGlassScope,
    trusted_time: u64,
) -> SensitiveBreakGlassDecision {
    let approver_binding = default_approvers(disclosure_decision);
    evaluate_with_approvers(
        disclosure_decision,
        break_glass_request,
        break_glass_scope,
        approver_binding,
        trusted_time,
    )
}

fn request() -> SensitiveBreakGlassRequest {
    SensitiveBreakGlassRequest::new(authority("task-42"), "incident-ticket-42")
}

fn human_scope() -> SensitiveBreakGlassScope {
    SensitiveBreakGlassScope::new(
        authority("task-42"),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::human("approval-human-1"),
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    )
}

fn dual_scope() -> SensitiveBreakGlassScope {
    SensitiveBreakGlassScope::new(
        authority("task-42"),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    )
}

#[test]
fn human_approval_break_glass_is_authorized_inside_the_exact_window() {
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            human_scope(),
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::Authorized
    );
}

#[test]
fn dual_control_break_glass_requires_two_distinct_approvals() {
    assert_eq!(
        evaluate(
            DisclosureDecision::DualControlRequired,
            request(),
            dual_scope(),
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::Authorized
    );

    assert_eq!(
        evaluate(
            DisclosureDecision::DualControlRequired,
            request(),
            human_scope(),
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::ApprovalInsufficient
    );

    let duplicate_approval_scope = SensitiveBreakGlassScope::new(
        authority("task-42"),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-1"),
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::DualControlRequired,
            request(),
            duplicate_approval_scope,
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::InvalidScope
    );
}

#[test]
fn dual_control_evidence_satisfies_the_single_human_approval_gate() {
    assert_eq!(
        evaluate_with_approvers(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            dual_scope(),
            BreakGlassApproverBinding::dual_control("support-approver-7", "security-approver-9"),
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::Authorized
    );
}

#[test]
fn break_glass_never_upgrades_non_approval_disclosure_decisions() {
    for decision in [
        DisclosureDecision::DenyAccess,
        DisclosureDecision::OpaqueHandleOnly,
        DisclosureDecision::DerivedValueOnly,
        DisclosureDecision::PartialFieldDisclosure,
        DisclosureDecision::FullFieldDisclosure,
    ] {
        assert_eq!(
            evaluate(decision, request(), dual_scope(), TRUSTED_TIME),
            SensitiveBreakGlassDecision::DisclosureNotApprovalGated(decision)
        );
    }
}

#[test]
fn exact_sensitive_authority_is_required_at_every_break_glass_boundary() {
    let mismatched_request =
        SensitiveBreakGlassRequest::new(authority("task-other"), "incident-ticket-42");
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            mismatched_request,
            human_scope(),
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::AuthorityMismatch
    );

    let mismatched_scope = SensitiveBreakGlassScope::new(
        authority("task-other"),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::human("approval-human-1"),
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            mismatched_scope,
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::AuthorityMismatch
    );
}

#[test]
fn approved_reason_must_match_the_requested_break_glass_reason() {
    let mismatched_reason_scope = SensitiveBreakGlassScope::new(
        authority("task-42"),
        "incident-ticket-other",
        BreakGlassApprovalEvidence::human("approval-human-1"),
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    );

    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            mismatched_reason_scope,
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::ReasonMismatch
    );
}

#[test]
fn malformed_request_and_scope_metadata_fail_closed() {
    let malformed_request = SensitiveBreakGlassRequest::new(authority("task-42"), "---");
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            malformed_request,
            human_scope(),
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::InvalidRequest
    );

    let malformed_reason_scope = SensitiveBreakGlassScope::new(
        authority("task-42"),
        "---",
        BreakGlassApprovalEvidence::human("approval-human-1"),
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            malformed_reason_scope,
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::InvalidScope
    );

    let malformed_approval_scope = SensitiveBreakGlassScope::new(
        authority("task-42"),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::human("approval id with spaces"),
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            malformed_approval_scope,
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::InvalidScope
    );
}

#[test]
fn break_glass_validity_is_positive_half_open_and_time_bounded() {
    for (valid_from, valid_until) in [(100, 100), (101, 100)] {
        let invalid_scope = SensitiveBreakGlassScope::new(
            authority("task-42"),
            "incident-ticket-42",
            BreakGlassApprovalEvidence::human("approval-human-1"),
            valid_from,
            valid_until,
            true,
            true,
        );
        assert_eq!(
            evaluate(
                DisclosureDecision::HumanApprovalRequired,
                request(),
                invalid_scope,
                TRUSTED_TIME,
            ),
            SensitiveBreakGlassDecision::InvalidValidityWindow
        );
    }

    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            human_scope(),
            VALID_FROM - 1,
        ),
        SensitiveBreakGlassDecision::NotYetValid
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            human_scope(),
            VALID_UNTIL,
        ),
        SensitiveBreakGlassDecision::Expired
    );
}

#[test]
fn missing_approval_monitoring_or_post_event_review_fails_closed() {
    let no_approval_scope = SensitiveBreakGlassScope::new(
        authority("task-42"),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::none(),
        VALID_FROM,
        VALID_UNTIL,
        true,
        true,
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            no_approval_scope,
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::ApprovalInsufficient
    );

    let no_monitoring_scope = SensitiveBreakGlassScope::new(
        authority("task-42"),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::human("approval-human-1"),
        VALID_FROM,
        VALID_UNTIL,
        false,
        true,
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            no_monitoring_scope,
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::HeightenedMonitoringRequired
    );

    let no_review_scope = SensitiveBreakGlassScope::new(
        authority("task-42"),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::human("approval-human-1"),
        VALID_FROM,
        VALID_UNTIL,
        true,
        false,
    );
    assert_eq!(
        evaluate(
            DisclosureDecision::HumanApprovalRequired,
            request(),
            no_review_scope,
            TRUSTED_TIME,
        ),
        SensitiveBreakGlassDecision::PostEventReviewRequired
    );
}
