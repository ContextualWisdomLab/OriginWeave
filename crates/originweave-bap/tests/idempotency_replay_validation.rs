#![allow(clippy::expect_used)]

use originweave_bap::{
    BapCommandReceiptError, BapTaskEvent, BapTaskLifecycle, BapTaskState,
    MAX_BAP_IDEMPOTENCY_KEY_BYTES, MAX_BAP_TASK_ID_BYTES,
};

#[test]
fn replay_validates_retry_identifiers_before_receipt_comparison() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_or_replay(None, "request-1", "tenant-1", "task-1", BapTaskEvent::Admit)
        .expect("initial receipt");

    let oversized_key = "x".repeat(MAX_BAP_IDEMPOTENCY_KEY_BYTES + 1);
    assert_eq!(
        task.apply_or_replay(
            Some(&receipt),
            &oversized_key,
            "tenant-1",
            "task-1",
            BapTaskEvent::Admit,
        ),
        Err(BapCommandReceiptError::IdempotencyKeyLimitExceeded),
    );

    assert_eq!(
        task.apply_or_replay(
            Some(&receipt),
            "request-1",
            "tenant with space",
            "task-1",
            BapTaskEvent::Admit,
        ),
        Err(BapCommandReceiptError::InvalidTenantId),
    );

    let oversized_task_id = "x".repeat(MAX_BAP_TASK_ID_BYTES + 1);
    assert_eq!(
        task.apply_or_replay(
            Some(&receipt),
            "request-1",
            "tenant-1",
            &oversized_task_id,
            BapTaskEvent::Admit,
        ),
        Err(BapCommandReceiptError::TaskIdLimitExceeded),
    );

    assert_eq!(task.state(), BapTaskState::Admitted);
    assert_eq!(task.transition_sequence(), 1);
}

#[test]
fn exact_receipt_can_be_validated_without_mutable_lifecycle_access() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_with_receipt("request-1", "tenant-1", "task-1", BapTaskEvent::Admit)
        .expect("initial receipt");
    let task = task;

    assert_eq!(
        task.validate_replay(
            &receipt,
            "request-1",
            "tenant-1",
            "task-1",
            BapTaskEvent::Admit,
        ),
        Ok(()),
    );
    assert_eq!(task.state(), BapTaskState::Admitted);
    assert_eq!(task.transition_sequence(), 1);
}
