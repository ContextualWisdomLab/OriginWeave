#![allow(clippy::expect_used)]

use originweave_bap::{BapCommandReceiptError, BapTaskEvent, BapTaskLifecycle, BapTaskState};

#[test]
fn replay_rejects_same_state_and_sequence_from_a_different_transition_path() {
    let mut source_task = BapTaskLifecycle::new();
    source_task.apply(BapTaskEvent::Admit).expect("admit source");
    source_task.apply(BapTaskEvent::Start).expect("start source");
    source_task
        .apply(BapTaskEvent::WaitForApproval)
        .expect("wait source");
    let receipt = source_task
        .apply_or_replay(
            None,
            "request-resume",
            "tenant-1",
            "task-1",
            BapTaskEvent::Resume,
        )
        .expect("source resume receipt");

    let mut other_task = BapTaskLifecycle::new();
    other_task.apply(BapTaskEvent::Admit).expect("admit other");
    other_task.apply(BapTaskEvent::Start).expect("start other");
    other_task
        .apply(BapTaskEvent::WaitForExternalInput)
        .expect("wait other");
    other_task.apply(BapTaskEvent::Resume).expect("resume other");

    assert_eq!(source_task.state(), BapTaskState::Running);
    assert_eq!(other_task.state(), BapTaskState::Running);
    assert_eq!(source_task.transition_sequence(), 4);
    assert_eq!(other_task.transition_sequence(), 4);
    assert_eq!(
        receipt.transition().previous_state(),
        BapTaskState::WaitingForApproval
    );

    assert_eq!(
        other_task.apply_or_replay(
            Some(&receipt),
            "request-resume",
            "tenant-1",
            "task-1",
            BapTaskEvent::Resume,
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(other_task.state(), BapTaskState::Running);
    assert_eq!(other_task.transition_sequence(), 4);
}

#[test]
fn restored_snapshot_without_last_transition_identity_cannot_replay_receipt() {
    let mut source_task = BapTaskLifecycle::new();
    let receipt = source_task
        .apply_or_replay(
            None,
            "request-admit",
            "tenant-1",
            "task-1",
            BapTaskEvent::Admit,
        )
        .expect("source admit receipt");

    let mut restored = BapTaskLifecycle::restore(BapTaskState::Admitted, 1)
        .expect("reachable admitted snapshot");
    assert_eq!(
        restored.apply_or_replay(
            Some(&receipt),
            "request-admit",
            "tenant-1",
            "task-1",
            BapTaskEvent::Admit,
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(restored.state(), BapTaskState::Admitted);
    assert_eq!(restored.transition_sequence(), 1);
}
