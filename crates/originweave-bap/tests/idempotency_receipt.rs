#![allow(clippy::expect_used)]

use std::error::Error as _;

use originweave_bap::{
    BapCommandReceiptError, BapTaskEvent, BapTaskLifecycle, BapTaskState,
    MAX_BAP_IDEMPOTENCY_KEY_BYTES, MAX_BAP_TASK_ID_BYTES, MAX_BAP_TENANT_ID_BYTES,
};

#[test]
fn receipt_binds_tenant_task_event_and_transition_for_replay_identification() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_with_receipt("request-1", "tenant-1", "task-1", BapTaskEvent::Admit)
        .expect("receipt");

    assert_eq!(receipt.idempotency_key(), "request-1");
    assert_eq!(receipt.tenant_id(), "tenant-1");
    assert_eq!(receipt.task_id(), "task-1");
    assert_eq!(receipt.event(), BapTaskEvent::Admit);
    assert_eq!(receipt.transition().current_state(), BapTaskState::Admitted);
    assert!(receipt.matches("request-1", "tenant-1", "task-1", BapTaskEvent::Admit));
    assert!(!receipt.matches("request-2", "tenant-1", "task-1", BapTaskEvent::Admit));
    assert!(!receipt.matches("request-1", "tenant-2", "task-1", BapTaskEvent::Admit));
    assert!(!receipt.matches("request-1", "tenant-1", "task-2", BapTaskEvent::Admit));
    assert!(!receipt.matches("request-1", "tenant-1", "task-1", BapTaskEvent::Start));
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
            task.apply_with_receipt(key, "tenant-1", "task-1", BapTaskEvent::Admit),
            if key.len() > MAX_BAP_IDEMPOTENCY_KEY_BYTES {
                Err(BapCommandReceiptError::IdempotencyKeyLimitExceeded)
            } else {
                Err(BapCommandReceiptError::InvalidIdempotencyKey)
            }
        );
    }
    for tenant_id in [
        "",
        "tenant with space",
        &"x".repeat(MAX_BAP_TENANT_ID_BYTES + 1),
    ] {
        assert_eq!(
            task.apply_with_receipt("request-1", tenant_id, "task-1", BapTaskEvent::Admit),
            if tenant_id.len() > MAX_BAP_TENANT_ID_BYTES {
                Err(BapCommandReceiptError::TenantIdLimitExceeded)
            } else {
                Err(BapCommandReceiptError::InvalidTenantId)
            }
        );
    }
    for task_id in [
        "",
        "task with space",
        &"x".repeat(MAX_BAP_TASK_ID_BYTES + 1),
    ] {
        assert_eq!(
            task.apply_with_receipt("request-1", "tenant-1", task_id, BapTaskEvent::Admit),
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
        task.apply_with_receipt("request-1", "tenant-1", "task-1", BapTaskEvent::Start),
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
fn exact_retry_replays_retained_receipt_without_reapplying_transition() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_or_replay(None, "request-1", "tenant-1", "task-1", BapTaskEvent::Admit)
        .expect("initial receipt");

    assert_eq!(task.state(), BapTaskState::Admitted);
    assert_eq!(task.transition_sequence(), 1);

    for _ in 0..100 {
        let replay = task
            .apply_or_replay(
                Some(&receipt),
                "request-1",
                "tenant-1",
                "task-1",
                BapTaskEvent::Admit,
            )
            .expect("exact retry");
        assert_eq!(replay, receipt);
        assert_eq!(task.state(), BapTaskState::Admitted);
        assert_eq!(task.transition_sequence(), 1);
    }
}

#[test]
fn retained_receipt_from_a_different_lifecycle_fails_closed() {
    let mut source_task = BapTaskLifecycle::new();
    let receipt = source_task
        .apply_or_replay(None, "request-1", "tenant-1", "task-1", BapTaskEvent::Admit)
        .expect("source receipt");

    let mut unrelated_task = BapTaskLifecycle::new();
    assert_eq!(
        unrelated_task.apply_or_replay(
            Some(&receipt),
            "request-1",
            "tenant-1",
            "task-1",
            BapTaskEvent::Admit,
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(unrelated_task.state(), BapTaskState::Created);
    assert_eq!(unrelated_task.transition_sequence(), 0);
}

#[test]
fn stale_receipt_after_lifecycle_advances_fails_closed() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_or_replay(None, "request-1", "tenant-1", "task-1", BapTaskEvent::Admit)
        .expect("initial receipt");
    task.apply(BapTaskEvent::Start).expect("start task");

    assert_eq!(
        task.apply_or_replay(
            Some(&receipt),
            "request-1",
            "tenant-1",
            "task-1",
            BapTaskEvent::Admit,
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(task.state(), BapTaskState::Running);
    assert_eq!(task.transition_sequence(), 2);
}

#[test]
fn replay_requires_exact_sequence_even_when_current_state_matches() {
    let mut task = BapTaskLifecycle::new();
    task.apply(BapTaskEvent::Admit).expect("admit task");
    let receipt = task
        .apply_or_replay(None, "request-2", "tenant-1", "task-1", BapTaskEvent::Start)
        .expect("start receipt");
    task.apply(BapTaskEvent::WaitForApproval)
        .expect("wait for approval");
    task.apply(BapTaskEvent::Resume).expect("resume task");

    assert_eq!(task.state(), BapTaskState::Running);
    assert_eq!(task.transition_sequence(), 4);
    assert_eq!(receipt.transition().current_state(), BapTaskState::Running);
    assert_eq!(receipt.transition().sequence(), 2);
    assert_eq!(
        task.apply_or_replay(
            Some(&receipt),
            "request-2",
            "tenant-1",
            "task-1",
            BapTaskEvent::Start,
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(task.state(), BapTaskState::Running);
    assert_eq!(task.transition_sequence(), 4);
}

#[test]
fn conflicting_retry_fails_closed_without_mutating_lifecycle() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_or_replay(None, "request-1", "tenant-1", "task-1", BapTaskEvent::Admit)
        .expect("initial receipt");

    for (idempotency_key, tenant_id, task_id, event) in [
        ("request-2", "tenant-1", "task-1", BapTaskEvent::Admit),
        ("request-1", "tenant-2", "task-1", BapTaskEvent::Admit),
        ("request-1", "tenant-1", "task-2", BapTaskEvent::Admit),
        ("request-1", "tenant-1", "task-1", BapTaskEvent::Start),
    ] {
        assert_eq!(
            task.apply_or_replay(Some(&receipt), idempotency_key, tenant_id, task_id, event,),
            Err(BapCommandReceiptError::IdempotencyConflict)
        );
        assert_eq!(task.state(), BapTaskState::Admitted);
        assert_eq!(task.transition_sequence(), 1);
    }
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
        BapCommandReceiptError::InvalidTenantId.to_string(),
        "BAP tenant ID is invalid"
    );
    assert_eq!(
        BapCommandReceiptError::TenantIdLimitExceeded.to_string(),
        "BAP tenant ID exceeds its byte limit"
    );
    assert_eq!(
        BapCommandReceiptError::TaskIdLimitExceeded.to_string(),
        "BAP task ID exceeds its byte limit"
    );
    assert_eq!(
        BapCommandReceiptError::IdempotencyConflict.to_string(),
        "BAP idempotency key conflicts with the retained command receipt"
    );
    assert!(
        BapCommandReceiptError::IdempotencyConflict
            .source()
            .is_none()
    );
    assert_eq!(
        BapCommandReceiptError::ReplayStateMismatch.to_string(),
        "BAP retained command receipt does not match the current lifecycle state"
    );
    assert!(
        BapCommandReceiptError::ReplayStateMismatch
            .source()
            .is_none()
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
