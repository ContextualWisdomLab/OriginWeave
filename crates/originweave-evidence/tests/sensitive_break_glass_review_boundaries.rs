#![allow(clippy::expect_used)]

//! Boundary regressions for credential-free break-glass review evidence.

use originweave_core::Origin;
use originweave_evidence::{
    SensitiveAccessClass, SensitiveAccessOutcome, SensitiveBreakGlassApprovalMode,
    SensitiveBreakGlassEvidence, SensitiveBreakGlassEvidenceInput,
    SensitiveBreakGlassReviewEvidence, SensitiveBreakGlassReviewEvidenceError,
    SensitiveBreakGlassReviewEvidenceInput, SensitiveBreakGlassReviewOutcome,
};

fn receipt() -> SensitiveBreakGlassEvidence {
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
        disclosure_epoch_seconds: 150,
        monitoring_reference: "monitoring-session-42".to_owned(),
        post_event_review_reference: "review-record-42".to_owned(),
        post_event_review_due_epoch_seconds: 250,
        retention_deadline_epoch_seconds: 1_000,
    })
    .expect("valid break-glass receipt")
}

fn input(reviewer_id: &str) -> SensitiveBreakGlassReviewEvidenceInput {
    SensitiveBreakGlassReviewEvidenceInput {
        reviewer_id: reviewer_id.to_owned(),
        completed_epoch_seconds: 250,
        outcome: SensitiveBreakGlassReviewOutcome::ConfirmedCompliant,
        finding_count: 0,
        remediation_reference: None,
    }
}

#[test]
fn empty_oversized_and_punctuation_only_reviewers_fail_closed() {
    let oversized = "a".repeat(129);
    for reviewer_id in ["", oversized.as_str(), "---"] {
        assert_eq!(
            SensitiveBreakGlassReviewEvidence::try_from_receipt(
                &receipt(),
                input(reviewer_id),
            ),
            Err(SensitiveBreakGlassReviewEvidenceError::InvalidIdentifier)
        );
    }
}

#[test]
fn valid_incident_escalation_retains_only_bounded_metadata() {
    let evidence = SensitiveBreakGlassReviewEvidence::try_from_receipt(
        &receipt(),
        SensitiveBreakGlassReviewEvidenceInput {
            reviewer_id: "security-reviewer-42".to_owned(),
            completed_epoch_seconds: 251,
            outcome: SensitiveBreakGlassReviewOutcome::IncidentEscalated,
            finding_count: u32::MAX,
            remediation_reference: Some("incident-record-42".to_owned()),
        },
    )
    .expect("bounded incident escalation metadata should be accepted");

    assert_eq!(
        evidence.outcome(),
        SensitiveBreakGlassReviewOutcome::IncidentEscalated
    );
    assert_eq!(evidence.finding_count(), u32::MAX);
    assert_eq!(evidence.remediation_reference(), Some("incident-record-42"));
}
