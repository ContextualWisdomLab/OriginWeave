#![allow(clippy::expect_used)]

//! Boundary regressions for credential-free break-glass policy identifiers.

use originweave_core::Origin;
use originweave_policy::{
    BreakGlassActorBinding, BreakGlassApprovalEvidence, BreakGlassApproverBinding,
    BreakGlassIdentityBindings, BreakGlassValidityPolicy, DataClassification, DisclosureDecision,
    DisclosureScope, SensitiveBreakGlassDecision, SensitiveBreakGlassRequest,
    SensitiveBreakGlassScope, SensitiveDataAuthority, SensitiveDataRequest,
    evaluate_sensitive_break_glass,
};

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
    request: SensitiveBreakGlassRequest,
    scope: SensitiveBreakGlassScope,
) -> SensitiveBreakGlassDecision {
    let exact_authority = authority();
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope =
        DisclosureScope::new(exact_authority, DisclosureDecision::DualControlRequired);
    let identity_bindings = BreakGlassIdentityBindings::new(
        BreakGlassActorBinding::new("support-operator-42", "support-operator-42"),
        BreakGlassApproverBinding::dual_control("support-approver-7", "security-approver-9"),
    );
    let validity_policy = BreakGlassValidityPolicy::new(100);

    evaluate_sensitive_break_glass(
        &disclosure_request,
        &disclosure_scope,
        &request,
        &scope,
        &identity_bindings,
        &validity_policy,
        150,
    )
}

fn valid_scope() -> SensitiveBreakGlassScope {
    SensitiveBreakGlassScope::new(
        authority(),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval-human-2"),
        100,
        200,
        true,
        true,
    )
}

#[test]
fn empty_and_oversized_break_glass_reasons_fail_closed() {
    let oversized_reason = "a".repeat(129);
    for reason_id in ["", oversized_reason.as_str()] {
        let request = SensitiveBreakGlassRequest::new(authority(), reason_id);
        assert_eq!(
            evaluate(request, valid_scope()),
            SensitiveBreakGlassDecision::InvalidRequest
        );
    }
}

#[test]
fn either_invalid_dual_control_approval_identifier_fails_closed() {
    for approval in [
        BreakGlassApprovalEvidence::dual_control("approval id with spaces", "approval-human-2"),
        BreakGlassApprovalEvidence::dual_control("approval-human-1", "approval id with spaces"),
    ] {
        let scope = SensitiveBreakGlassScope::new(
            authority(),
            "incident-ticket-42",
            approval,
            100,
            200,
            true,
            true,
        );
        assert_eq!(
            evaluate(
                SensitiveBreakGlassRequest::new(authority(), "incident-ticket-42"),
                scope,
            ),
            SensitiveBreakGlassDecision::InvalidScope
        );
    }
}
