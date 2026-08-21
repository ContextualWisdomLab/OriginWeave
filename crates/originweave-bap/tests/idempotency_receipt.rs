#![allow(clippy::expect_used)]

use std::error::Error as _;

use originweave_bap::{
    BapCommandReceiptError, BapTaskEvent, BapTaskLifecycle, BapTaskState,
    MAX_BAP_IDEMPOTENCY_KEY_BYTES, MAX_BAP_TASK_ID_BYTES,
};

#[test]
fn receipt_binds_task_event_and_transition_for_replay_identification() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_with_receipt("request-1", "task-1", BapTaskEvent::Admit)
        .expect("receipt");

    assert_eq!(receipt.idempotency_key(), "request-1");
    assert_eq!(receipt.task_id(), "task-1");
    assert_eq!(receipt.event(), BapTaskEvent::Admit);
    assert_eq!(receipt.transition().current_state(), BapTaskState::Admitted);
    assert!(receipt.matches("request-1", "task-1", BapTaskEvent::Admit));
    assert!(!receipt.matches("request-2", "task-1", BapTaskEvent::Admit));
    assert!(!receipt.matches("request-1", "task-2", BapTaskEvent::Admit));
    assert!(!receipt.matches("request-1", "task-1", BapTaskEvent::Start));
}

#[test]
fn receipt_cannot_be_minted_from_an_already_accepted_transition() {
    let source = include_str!("../src/lib.rs");
    let receipt_impl = source
        .split("impl BapCommandReceipt {")
        .nth(1)
        .expect("receipt impl")
        .split("/// Deterministic fail-closed BAP task-lifecycle kernel.")
        .next()
        .expect("receipt impl boundary");

    assert!(
        !receipt_impl.contains("pub fn new("),
        "public receipt construction can rebind an accepted transition to arbitrary retry/task metadata",
    );
}

#[test]
fn receipt_rejects_unbounded_or_ambiguous_identifiers() {
    let mut task = BapTaskLifecycle::new();
    for key in [
        "",
        "request with space",
        &"x".repeat(MAX_BAP_IDEMPOTENCY_KEY_BYTES + 1),
    ] {
        assert_eq!(
            task.apply_with_receipt(key, "task-1", BapTaskEvent::Admit),
            if key.len() > MAX_BAP_IDEMPOTENCY_KEY_BYTES {
                Err(BapCommandReceiptError::IdempotencyKeyLimitExceeded)
            } else {
                Err(BapCommandReceiptError::InvalidIdempotencyKey)
            }
        );
    }
    for task_id in [
        "",
        "task with space",
        &"x".repeat(MAX_BAP_TASK_ID_BYTES + 1),
    ] {
        assert_eq!(
            task.apply_with_receipt("request-1", task_id, BapTaskEvent::Admit),
            if task_id.len() > MAX_BAP_TASK_ID_BYTES {
                Err(BapCommandReceiptError::TaskIdLimitExceeded)
            } else {
                Err(BapCommandReceiptError::InvalidTaskId)
            }
        );
    }
}

#[test]
fn receipt_preserves_lifecycle_failure_without_mutating_the_task() {
    let mut task = BapTaskLifecycle::new();
    assert_eq!(
        task.apply_with_receipt("request-1", "task-1", BapTaskEvent::Start),
        Err(BapCommandReceiptError::TransitionRejected {
            error: originweave_bap::BapTaskTransitionError::InvalidTransition {
                from: BapTaskState::Created,
                event: BapTaskEvent::Start,
            },
        })
    );
    assert_eq!(task.state(), BapTaskState::Created);
    assert_eq!(task.transition_sequence(), 0);
}

#[test]
fn receipt_errors_have_standard_error_contracts() {
    let error = BapCommandReceiptError::InvalidTaskId;
    assert_eq!(error.to_string(), "BAP task ID is invalid");
    assert!(error.source().is_none());
    assert_eq!(
        BapCommandReceiptError::InvalidIdempotencyKey.to_string(),
        "BAP idempotency key is invalid"
    );
    assert_eq!(
        BapCommandReceiptError::IdempotencyKeyLimitExceeded.to_string(),
        "BAP idempotency key exceeds its byte limit"
    );
    assert_eq!(
        BapCommandReceiptError::TaskIdLimitExceeded.to_string(),
        "BAP task ID exceeds its byte limit"
    );
    let transition = BapCommandReceiptError::TransitionRejected {
        error: originweave_bap::BapTaskTransitionError::SequenceExhausted,
    };
    assert_eq!(
        transition.to_string(),
        "BAP task transition sequence is exhausted"
    );
    assert!(transition.source().is_some());
}
