#![allow(clippy::expect_used)]

//! Credential-free audit contract for one actual sensitive-data break-glass disclosure.
//!
//! The receipt records bounded authority, approval, validity, monitoring, and review metadata. It
//! deliberately has no protected-value, opaque-handle, credential, prompt, or provider-token field.

use originweave_core::Origin;
use originweave_evidence::{
    MAX_SENSITIVE_FIELD_COUNT, SensitiveAccessClass, SensitiveAccessOutcome,
    SensitiveBreakGlassApprovalMode, SensitiveBreakGlassEvidence, SensitiveBreakGlassEvidenceError,
    SensitiveBreakGlassEvidenceInput,
};

const VALID_FROM: u64 = 100;
const VALID_UNTIL: u64 = 200;
const MAXIMUM_WINDOW: u64 = 100;
const DECISION_TIME: u64 = 120;
const DISCLOSURE_TIME: u64 = 150;
const REVIEW_DUE_TIME: u64 = 250;

fn valid_input() -> SensitiveBreakGlassEvidenceInput {
    SensitiveBreakGlassEvidenceInput {
        request_id: "request-42".to_owned(),
        decision_id: "decision-42".to_owned(),
        tenant_id: "tenant-alpha".to_owned(),
        actor_id: "support-operator-42".to_owned(),
        approved_actor_id: "support-operator-42".to_owned(),
        task_id: "task-42".to_owned(),
        field_ids: vec!["customer-address".to_owned()],
        purpose_id: "incident-response".to_owned(),
        destination: Origin::parse("https://support-console.example")
            .expect("valid destination origin"),
        classification: SensitiveAccessClass::SensitivePersonalData,
        outcome: SensitiveAccessOutcome::PartialFieldDisclosure,
        policy_version: "sensitive-policy-v7".to_owned(),
        reason_id: "incident-ticket-42".to_owned(),
        approval_mode: SensitiveBreakGlassApprovalMode::DualControl,
        approval_references: vec!["approval-human-1".to_owned(), "approval-human-2".to_owned()],
        valid_from_epoch_seconds: VALID_FROM,
        valid_until_epoch_seconds: VALID_UNTIL,
        maximum_window_seconds: MAXIMUM_WINDOW,
        decision_epoch_seconds: DECISION_TIME,
        disclosure_epoch_seconds: DISCLOSURE_TIME,
        monitoring_reference: "monitoring-session-42".to_owned(),
        post_event_review_reference: "review-record-42".to_owned(),
        post_event_review_due_epoch_seconds: REVIEW_DUE_TIME,
    }
}

#[test]
fn valid_break_glass_disclosure_builds_a_complete_credential_free_receipt() {
    let evidence = SensitiveBreakGlassEvidence::try_from(valid_input())
        .expect("valid break-glass evidence should be accepted");

    assert_eq!(evidence.request_id(), "request-42");
    assert_eq!(evidence.decision_id(), "decision-42");
    assert_eq!(evidence.tenant_id(), "tenant-alpha");
    assert_eq!(evidence.actor_id(), "support-operator-42");
    assert_eq!(evidence.approved_actor_id(), "support-operator-42");
    assert_eq!(evidence.task_id(), "task-42");
    assert_eq!(evidence.field_ids(), ["customer-address"]);
    assert_eq!(evidence.purpose_id(), "incident-response");
    assert_eq!(
        evidence.destination().as_str(),
        "https://support-console.example"
    );
    assert_eq!(
        evidence.classification(),
        SensitiveAccessClass::SensitivePersonalData
    );
    assert_eq!(
        evidence.outcome(),
        SensitiveAccessOutcome::PartialFieldDisclosure
    );
    assert_eq!(evidence.policy_version(), "sensitive-policy-v7");
    assert_eq!(evidence.reason_id(), "incident-ticket-42");
    assert_eq!(
        evidence.approval_mode(),
        SensitiveBreakGlassApprovalMode::DualControl
    );
    assert_eq!(
        evidence.approval_references(),
        ["approval-human-1", "approval-human-2"]
    );
    assert_eq!(evidence.valid_from_epoch_seconds(), VALID_FROM);
    assert_eq!(evidence.valid_until_epoch_seconds(), VALID_UNTIL);
    assert_eq!(evidence.maximum_window_seconds(), MAXIMUM_WINDOW);
    assert_eq!(evidence.decision_epoch_seconds(), DECISION_TIME);
    assert_eq!(evidence.disclosure_epoch_seconds(), DISCLOSURE_TIME);
    assert_eq!(evidence.monitoring_reference(), "monitoring-session-42");
    assert_eq!(evidence.post_event_review_reference(), "review-record-42");
    assert_eq!(
        evidence.post_event_review_due_epoch_seconds(),
        REVIEW_DUE_TIME
    );

    let debug = format!("{evidence:?}");
    for forbidden in [
        "protected-customer-value",
        "opaque-handle-token",
        "provider-secret",
    ] {
        assert!(!debug.contains(forbidden));
    }
}

#[test]
fn break_glass_evidence_is_bound_to_the_exact_approved_actor() {
    let mut input = valid_input();
    input.approved_actor_id = "support-operator-other".to_owned();

    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(input),
        Err(SensitiveBreakGlassEvidenceError::ActorMismatch)
    );
}

#[test]
fn approval_mode_requires_the_exact_bounded_approval_set() {
    let mut human = valid_input();
    human.approval_mode = SensitiveBreakGlassApprovalMode::Human;
    human.approval_references = vec!["approval-human-1".to_owned()];
    assert!(SensitiveBreakGlassEvidence::try_from(human).is_ok());

    for approval_references in [
        Vec::new(),
        vec!["approval-human-1".to_owned(), "approval-human-2".to_owned()],
    ] {
        let mut invalid_human = valid_input();
        invalid_human.approval_mode = SensitiveBreakGlassApprovalMode::Human;
        invalid_human.approval_references = approval_references;
        assert_eq!(
            SensitiveBreakGlassEvidence::try_from(invalid_human),
            Err(SensitiveBreakGlassEvidenceError::InvalidApprovalSet)
        );
    }

    for approval_references in [
        vec!["approval-human-1".to_owned()],
        vec!["approval-human-1".to_owned(), "approval-human-1".to_owned()],
        vec![
            "approval-human-1".to_owned(),
            "approval-human-2".to_owned(),
            "approval-human-3".to_owned(),
        ],
    ] {
        let mut invalid_dual = valid_input();
        invalid_dual.approval_references = approval_references;
        assert_eq!(
            SensitiveBreakGlassEvidence::try_from(invalid_dual),
            Err(SensitiveBreakGlassEvidenceError::InvalidApprovalSet)
        );
    }
}

#[test]
fn malformed_identifiers_fail_closed_at_required_and_approval_positions() {
    let mut invalid_reason = valid_input();
    invalid_reason.reason_id = "---".to_owned();
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(invalid_reason),
        Err(SensitiveBreakGlassEvidenceError::InvalidIdentifier)
    );

    let mut invalid_monitoring = valid_input();
    invalid_monitoring.monitoring_reference = "monitoring reference with spaces".to_owned();
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(invalid_monitoring),
        Err(SensitiveBreakGlassEvidenceError::InvalidIdentifier)
    );

    let mut empty_identifier = valid_input();
    empty_identifier.request_id.clear();
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(empty_identifier),
        Err(SensitiveBreakGlassEvidenceError::InvalidIdentifier)
    );

    let mut oversized_identifier = valid_input();
    oversized_identifier.decision_id = "a".repeat(129);
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(oversized_identifier),
        Err(SensitiveBreakGlassEvidenceError::InvalidIdentifier)
    );

    let mut invalid_approval_reference = valid_input();
    invalid_approval_reference.approval_references[0] = "approval reference with spaces".to_owned();
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(invalid_approval_reference),
        Err(SensitiveBreakGlassEvidenceError::InvalidIdentifier)
    );
}

#[test]
fn empty_oversized_invalid_and_duplicate_field_sets_fail_closed() {
    let mut empty_fields = valid_input();
    empty_fields.field_ids.clear();
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(empty_fields),
        Err(SensitiveBreakGlassEvidenceError::InvalidFieldSet)
    );

    let mut oversized_fields = valid_input();
    oversized_fields.field_ids = (0..=MAX_SENSITIVE_FIELD_COUNT)
        .map(|index| format!("customer-field-{index}"))
        .collect();
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(oversized_fields),
        Err(SensitiveBreakGlassEvidenceError::InvalidFieldSet)
    );

    let mut invalid_field = valid_input();
    invalid_field.field_ids = vec!["---".to_owned()];
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(invalid_field),
        Err(SensitiveBreakGlassEvidenceError::InvalidFieldSet)
    );

    let mut duplicate_fields = valid_input();
    duplicate_fields.field_ids = vec!["customer-address".to_owned(), "customer-address".to_owned()];
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(duplicate_fields),
        Err(SensitiveBreakGlassEvidenceError::InvalidFieldSet)
    );
}

#[test]
fn only_an_actual_disclosure_can_be_recorded_as_break_glass_evidence() {
    for outcome in [
        SensitiveAccessOutcome::DenyAccess,
        SensitiveAccessOutcome::OpaqueHandleOnly,
        SensitiveAccessOutcome::HumanApprovalRequired,
        SensitiveAccessOutcome::DualControlRequired,
    ] {
        let mut input = valid_input();
        input.outcome = outcome;
        assert_eq!(
            SensitiveBreakGlassEvidence::try_from(input),
            Err(SensitiveBreakGlassEvidenceError::DisclosureNotRecorded)
        );
    }
}

#[test]
fn validity_window_and_event_lifecycle_are_fail_closed() {
    let mut zero_maximum = valid_input();
    zero_maximum.maximum_window_seconds = 0;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(zero_maximum),
        Err(SensitiveBreakGlassEvidenceError::InvalidValidityWindow)
    );

    let mut empty_window = valid_input();
    empty_window.valid_until_epoch_seconds = VALID_FROM;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(empty_window),
        Err(SensitiveBreakGlassEvidenceError::InvalidValidityWindow)
    );

    let mut overlong_window = valid_input();
    overlong_window.valid_until_epoch_seconds = VALID_UNTIL + 1;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(overlong_window),
        Err(SensitiveBreakGlassEvidenceError::WindowExceedsMaximum)
    );

    let mut zero_decision_time = valid_input();
    zero_decision_time.decision_epoch_seconds = 0;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(zero_decision_time),
        Err(SensitiveBreakGlassEvidenceError::InvalidLifecycle)
    );

    let mut decision_before_window = valid_input();
    decision_before_window.decision_epoch_seconds = VALID_FROM - 1;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(decision_before_window),
        Err(SensitiveBreakGlassEvidenceError::InvalidLifecycle)
    );

    let mut decision_at_expiry = valid_input();
    decision_at_expiry.decision_epoch_seconds = VALID_UNTIL;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(decision_at_expiry),
        Err(SensitiveBreakGlassEvidenceError::InvalidLifecycle)
    );

    let mut disclosure_before_decision = valid_input();
    disclosure_before_decision.disclosure_epoch_seconds = DECISION_TIME - 1;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(disclosure_before_decision),
        Err(SensitiveBreakGlassEvidenceError::InvalidLifecycle)
    );

    let mut disclosure_at_expiry = valid_input();
    disclosure_at_expiry.disclosure_epoch_seconds = VALID_UNTIL;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(disclosure_at_expiry),
        Err(SensitiveBreakGlassEvidenceError::InvalidLifecycle)
    );

    let mut review_not_after_disclosure = valid_input();
    review_not_after_disclosure.post_event_review_due_epoch_seconds = DISCLOSURE_TIME;
    assert_eq!(
        SensitiveBreakGlassEvidence::try_from(review_not_after_disclosure),
        Err(SensitiveBreakGlassEvidenceError::InvalidLifecycle)
    );
}

#[test]
fn error_text_is_stable_and_has_no_nested_sensitive_source() {
    let cases = [
        (
            SensitiveBreakGlassEvidenceError::InvalidIdentifier,
            "invalid sensitive break-glass identifier",
        ),
        (
            SensitiveBreakGlassEvidenceError::InvalidFieldSet,
            "invalid sensitive break-glass field set",
        ),
        (
            SensitiveBreakGlassEvidenceError::ActorMismatch,
            "sensitive break-glass actor mismatch",
        ),
        (
            SensitiveBreakGlassEvidenceError::InvalidApprovalSet,
            "invalid sensitive break-glass approval set",
        ),
        (
            SensitiveBreakGlassEvidenceError::DisclosureNotRecorded,
            "sensitive break-glass evidence did not record a disclosure",
        ),
        (
            SensitiveBreakGlassEvidenceError::InvalidValidityWindow,
            "invalid sensitive break-glass validity window",
        ),
        (
            SensitiveBreakGlassEvidenceError::WindowExceedsMaximum,
            "sensitive break-glass validity window exceeds local maximum",
        ),
        (
            SensitiveBreakGlassEvidenceError::InvalidLifecycle,
            "invalid sensitive break-glass evidence lifecycle",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(std::error::Error::source(&error).is_none());
    }
}
