#![allow(clippy::expect_used)]

//! Credential-free evidence contract for completed sensitive break-glass post-event review.
//!
//! Review evidence binds to the exact break-glass receipt and records only bounded outcome metadata.
//! It carries no protected value, free-form finding text, approval payload, credential, or model data.

use originweave_core::Origin;
use originweave_evidence::{
    SensitiveAccessClass, SensitiveAccessOutcome, SensitiveBreakGlassApprovalMode,
    SensitiveBreakGlassEvidence, SensitiveBreakGlassEvidenceInput,
    SensitiveBreakGlassReviewEvidence, SensitiveBreakGlassReviewEvidenceError,
    SensitiveBreakGlassReviewEvidenceInput, SensitiveBreakGlassReviewOutcome,
    SensitiveBreakGlassReviewTimeliness,
};

const DISCLOSURE_TIME: u64 = 150;
const REVIEW_DUE_TIME: u64 = 250;
const RETENTION_DEADLINE_TIME: u64 = 1_000;

fn break_glass_receipt() -> SensitiveBreakGlassEvidence {
    SensitiveBreakGlassEvidence::try_from(SensitiveBreakGlassEvidenceInput {
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
        valid_from_epoch_seconds: 100,
        valid_until_epoch_seconds: 200,
        maximum_window_seconds: 100,
        decision_epoch_seconds: 120,
        disclosure_epoch_seconds: DISCLOSURE_TIME,
        monitoring_reference: "monitoring-session-42".to_owned(),
        post_event_review_reference: "review-record-42".to_owned(),
        post_event_review_due_epoch_seconds: REVIEW_DUE_TIME,
        retention_deadline_epoch_seconds: RETENTION_DEADLINE_TIME,
    })
    .expect("valid break-glass receipt")
}

fn compliant_input(completed_epoch_seconds: u64) -> SensitiveBreakGlassReviewEvidenceInput {
    SensitiveBreakGlassReviewEvidenceInput {
        reviewer_id: "security-reviewer-42".to_owned(),
        completed_epoch_seconds,
        outcome: SensitiveBreakGlassReviewOutcome::ConfirmedCompliant,
        finding_count: 0,
        remediation_reference: None,
    }
}

#[test]
fn on_time_compliant_review_is_bound_to_the_exact_break_glass_receipt() {
    let receipt = break_glass_receipt();
    let evidence = SensitiveBreakGlassReviewEvidence::try_from_receipt(
        &receipt,
        compliant_input(REVIEW_DUE_TIME),
    )
    .expect("valid on-time review evidence");

    assert_eq!(evidence.request_id(), "request-42");
    assert_eq!(evidence.decision_id(), "decision-42");
    assert_eq!(evidence.review_reference(), "review-record-42");
    assert_eq!(evidence.reviewer_id(), "security-reviewer-42");
    assert_eq!(evidence.completed_epoch_seconds(), REVIEW_DUE_TIME);
    assert_eq!(
        evidence.timeliness(),
        SensitiveBreakGlassReviewTimeliness::OnTime
    );
    assert_eq!(
        evidence.outcome(),
        SensitiveBreakGlassReviewOutcome::ConfirmedCompliant
    );
    assert_eq!(evidence.finding_count(), 0);
    assert_eq!(evidence.remediation_reference(), None);

    let debug = format!("{evidence:?}");
    for forbidden in [
        "protected-customer-value",
        "opaque-handle-token",
        "approval-payload",
    ] {
        assert!(!debug.contains(forbidden));
    }
}

#[test]
fn late_policy_violation_is_recorded_before_retention_expiry() {
    let receipt = break_glass_receipt();
    let evidence = SensitiveBreakGlassReviewEvidence::try_from_receipt(
        &receipt,
        SensitiveBreakGlassReviewEvidenceInput {
            reviewer_id: "security-reviewer-42".to_owned(),
            completed_epoch_seconds: REVIEW_DUE_TIME + 1,
            outcome: SensitiveBreakGlassReviewOutcome::PolicyViolation,
            finding_count: 2,
            remediation_reference: Some("remediation-ticket-42".to_owned()),
        },
    )
    .expect("late review should remain recordable before retention expiry");

    assert_eq!(
        evidence.timeliness(),
        SensitiveBreakGlassReviewTimeliness::Late
    );
    assert_eq!(
        evidence.outcome(),
        SensitiveBreakGlassReviewOutcome::PolicyViolation
    );
    assert_eq!(evidence.finding_count(), 2);
    assert_eq!(
        evidence.remediation_reference(),
        Some("remediation-ticket-42")
    );
}

#[test]
fn post_event_reviewer_must_be_independent_from_the_disclosing_actor() {
    let receipt = break_glass_receipt();
    for reviewer_id in [receipt.actor_id(), receipt.approved_actor_id()] {
        let mut input = compliant_input(REVIEW_DUE_TIME);
        input.reviewer_id = reviewer_id.to_owned();
        assert_eq!(
            SensitiveBreakGlassReviewEvidence::try_from_receipt(&receipt, input),
            Err(SensitiveBreakGlassReviewEvidenceError::ReviewerConflict)
        );
    }
}

#[test]
fn malformed_reviewer_or_remediation_identifiers_fail_closed() {
    let receipt = break_glass_receipt();
    let mut malformed_reviewer = compliant_input(REVIEW_DUE_TIME);
    malformed_reviewer.reviewer_id = "reviewer id with spaces".to_owned();
    assert_eq!(
        SensitiveBreakGlassReviewEvidence::try_from_receipt(&receipt, malformed_reviewer),
        Err(SensitiveBreakGlassReviewEvidenceError::InvalidIdentifier)
    );

    let malformed_remediation = SensitiveBreakGlassReviewEvidenceInput {
        reviewer_id: "security-reviewer-42".to_owned(),
        completed_epoch_seconds: REVIEW_DUE_TIME,
        outcome: SensitiveBreakGlassReviewOutcome::IncidentEscalated,
        finding_count: 1,
        remediation_reference: Some("---".to_owned()),
    };
    assert_eq!(
        SensitiveBreakGlassReviewEvidence::try_from_receipt(&receipt, malformed_remediation),
        Err(SensitiveBreakGlassReviewEvidenceError::InvalidIdentifier)
    );
}

#[test]
fn completion_must_follow_disclosure_and_not_exceed_retention() {
    let receipt = break_glass_receipt();
    for completed_epoch_seconds in [DISCLOSURE_TIME, RETENTION_DEADLINE_TIME + 1] {
        assert_eq!(
            SensitiveBreakGlassReviewEvidence::try_from_receipt(
                &receipt,
                compliant_input(completed_epoch_seconds),
            ),
            Err(SensitiveBreakGlassReviewEvidenceError::InvalidCompletionTime)
        );
    }

    assert!(
        SensitiveBreakGlassReviewEvidence::try_from_receipt(
            &receipt,
            compliant_input(RETENTION_DEADLINE_TIME),
        )
        .is_ok()
    );
}

#[test]
fn outcome_and_finding_metadata_must_be_consistent() {
    let receipt = break_glass_receipt();
    let cases = [
        SensitiveBreakGlassReviewEvidenceInput {
            reviewer_id: "security-reviewer-42".to_owned(),
            completed_epoch_seconds: REVIEW_DUE_TIME,
            outcome: SensitiveBreakGlassReviewOutcome::ConfirmedCompliant,
            finding_count: 1,
            remediation_reference: None,
        },
        SensitiveBreakGlassReviewEvidenceInput {
            reviewer_id: "security-reviewer-42".to_owned(),
            completed_epoch_seconds: REVIEW_DUE_TIME,
            outcome: SensitiveBreakGlassReviewOutcome::ConfirmedCompliant,
            finding_count: 0,
            remediation_reference: Some("remediation-ticket-42".to_owned()),
        },
        SensitiveBreakGlassReviewEvidenceInput {
            reviewer_id: "security-reviewer-42".to_owned(),
            completed_epoch_seconds: REVIEW_DUE_TIME,
            outcome: SensitiveBreakGlassReviewOutcome::PolicyViolation,
            finding_count: 0,
            remediation_reference: Some("remediation-ticket-42".to_owned()),
        },
        SensitiveBreakGlassReviewEvidenceInput {
            reviewer_id: "security-reviewer-42".to_owned(),
            completed_epoch_seconds: REVIEW_DUE_TIME,
            outcome: SensitiveBreakGlassReviewOutcome::IncidentEscalated,
            finding_count: 1,
            remediation_reference: None,
        },
    ];

    for input in cases {
        assert_eq!(
            SensitiveBreakGlassReviewEvidence::try_from_receipt(&receipt, input),
            Err(SensitiveBreakGlassReviewEvidenceError::InvalidOutcomeEvidence)
        );
    }
}

#[test]
fn error_text_is_stable_and_source_free() {
    let cases = [
        (
            SensitiveBreakGlassReviewEvidenceError::InvalidIdentifier,
            "invalid sensitive break-glass review identifier",
        ),
        (
            SensitiveBreakGlassReviewEvidenceError::ReviewerConflict,
            "sensitive break-glass reviewer conflicts with disclosed actor",
        ),
        (
            SensitiveBreakGlassReviewEvidenceError::InvalidCompletionTime,
            "invalid sensitive break-glass review completion time",
        ),
        (
            SensitiveBreakGlassReviewEvidenceError::InvalidOutcomeEvidence,
            "invalid sensitive break-glass review outcome evidence",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(std::error::Error::source(&error).is_none());
    }
}
