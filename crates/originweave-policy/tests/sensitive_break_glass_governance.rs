#![allow(clippy::expect_used)]

//! Governance regressions for non-transferable, short-lived break-glass access.

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
const MAXIMUM_WINDOW: u64 = 100;

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

fn request() -> SensitiveBreakGlassRequest {
    SensitiveBreakGlassRequest::new(authority(), "incident-ticket-42")
}

fn scope(valid_from: u64, valid_until: u64) -> SensitiveBreakGlassScope {
    SensitiveBreakGlassScope::new(
        authority(),
        "incident-ticket-42",
        BreakGlassApprovalEvidence::human("approval-human-1"),
        valid_from,
        valid_until,
        true,
        true,
    )
}

fn evaluate(
    request: SensitiveBreakGlassRequest,
    scope: SensitiveBreakGlassScope,
    actor_binding: BreakGlassActorBinding,
    validity_policy: BreakGlassValidityPolicy,
) -> SensitiveBreakGlassDecision {
    let exact_authority = authority();
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope =
        DisclosureScope::new(exact_authority, DisclosureDecision::HumanApprovalRequired);
    let identity_bindings = BreakGlassIdentityBindings::new(
        actor_binding,
        BreakGlassApproverBinding::human("support-approver-7"),
    );

    evaluate_sensitive_break_glass(
        &disclosure_request,
        &disclosure_scope,
        &request,
        &scope,
        &identity_bindings,
        &validity_policy,
        TRUSTED_TIME,
    )
}

#[test]
fn exact_actor_and_reviewed_short_window_are_authorized() {
    assert_eq!(
        evaluate(
            request(),
            scope(VALID_FROM, VALID_UNTIL),
            BreakGlassActorBinding::new("support-operator-42", "support-operator-42"),
            BreakGlassValidityPolicy::new(MAXIMUM_WINDOW),
        ),
        SensitiveBreakGlassDecision::Authorized
    );
}

#[test]
fn break_glass_actor_identity_is_non_transferable() {
    assert_eq!(
        evaluate(
            request(),
            scope(VALID_FROM, VALID_UNTIL),
            BreakGlassActorBinding::new("support-operator-42", "support-operator-other"),
            BreakGlassValidityPolicy::new(MAXIMUM_WINDOW),
        ),
        SensitiveBreakGlassDecision::ActorMismatch
    );
}

#[test]
fn malformed_actor_binding_fails_closed() {
    assert_eq!(
        evaluate(
            request(),
            scope(VALID_FROM, VALID_UNTIL),
            BreakGlassActorBinding::new("", "support-operator-42"),
            BreakGlassValidityPolicy::new(MAXIMUM_WINDOW),
        ),
        SensitiveBreakGlassDecision::InvalidActorBinding
    );

    let oversized_actor = "a".repeat(129);
    assert_eq!(
        evaluate(
            request(),
            scope(VALID_FROM, VALID_UNTIL),
            BreakGlassActorBinding::new("support-operator-42", oversized_actor.as_str()),
            BreakGlassValidityPolicy::new(MAXIMUM_WINDOW),
        ),
        SensitiveBreakGlassDecision::InvalidActorBinding
    );
}

#[test]
fn invalid_or_overlong_break_glass_windows_fail_closed() {
    assert_eq!(
        evaluate(
            request(),
            scope(VALID_FROM, VALID_UNTIL),
            BreakGlassActorBinding::new("support-operator-42", "support-operator-42"),
            BreakGlassValidityPolicy::new(0),
        ),
        SensitiveBreakGlassDecision::InvalidValidityPolicy
    );

    assert_eq!(
        evaluate(
            request(),
            scope(VALID_FROM, VALID_UNTIL + 1),
            BreakGlassActorBinding::new("support-operator-42", "support-operator-42"),
            BreakGlassValidityPolicy::new(MAXIMUM_WINDOW),
        ),
        SensitiveBreakGlassDecision::ValidityWindowTooLong
    );
}
