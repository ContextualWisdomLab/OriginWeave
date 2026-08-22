#![allow(clippy::expect_used)]

use originweave_bap::{
    BapCommandReceipt, BapTaskEvent, BapTaskLifecycle, BapTaskState, BapTaskTransition,
    BapTaskTransitionError,
};

#[test]
fn reconciliation_receipt_replays_after_transition_backed_restore() {
    let mut lifecycle = BapTaskLifecycle::new();
    lifecycle.apply(BapTaskEvent::Admit).expect("admit");
    lifecycle.apply(BapTaskEvent::Start).expect("start");
    let receipt = lifecycle
        .apply_with_receipt(
            "reconcile-1",
            "tenant-a",
            "task-a",
            BapTaskEvent::RequireReconciliation,
        )
        .expect("require reconciliation");

    assert_eq!(lifecycle.state(), BapTaskState::ReconciliationRequired);
    let transition = BapTaskTransition::restore(
        BapTaskState::Running,
        BapTaskState::ReconciliationRequired,
        3,
        BapTaskEvent::RequireReconciliation,
    )
    .expect("restore reconciliation transition");
    assert_eq!(transition, receipt.transition());

    let restored_receipt =
        BapCommandReceipt::restore("reconcile-1", "tenant-a", "task-a", transition)
            .expect("restore receipt");
    let mut restored = BapTaskLifecycle::restore_with_transition(
        BapTaskState::ReconciliationRequired,
        3,
        Some(transition),
    )
    .expect("restore lifecycle");

    let replay = restored
        .apply_or_replay(
            Some(&restored_receipt),
            "reconcile-1",
            "tenant-a",
            "task-a",
            BapTaskEvent::RequireReconciliation,
        )
        .expect("replay reconciliation command");
    assert_eq!(replay, restored_receipt);
    assert_eq!(restored.transition_sequence(), 3);

    let resolution = restored
        .apply(BapTaskEvent::ResolveReconciliation)
        .expect("resolve reconciliation");
    assert_eq!(resolution.previous_state(), BapTaskState::ReconciliationRequired);
    assert_eq!(resolution.current_state(), BapTaskState::Running);
    assert_eq!(resolution.sequence(), 4);
}

#[test]
fn dead_letter_receipt_replays_but_terminal_state_stays_closed() {
    let mut lifecycle = BapTaskLifecycle::new();
    lifecycle.apply(BapTaskEvent::Admit).expect("admit");
    lifecycle.apply(BapTaskEvent::Start).expect("start");
    lifecycle
        .apply(BapTaskEvent::RequireReconciliation)
        .expect("require reconciliation");
    let receipt = lifecycle
        .apply_with_receipt(
            "dead-letter-1",
            "tenant-a",
            "task-a",
            BapTaskEvent::DeadLetter,
        )
        .expect("dead letter");

    assert_eq!(lifecycle.state(), BapTaskState::DeadLettered);
    let transition = BapTaskTransition::restore(
        BapTaskState::ReconciliationRequired,
        BapTaskState::DeadLettered,
        4,
        BapTaskEvent::DeadLetter,
    )
    .expect("restore dead-letter transition");
    assert_eq!(transition, receipt.transition());

    let restored_receipt =
        BapCommandReceipt::restore("dead-letter-1", "tenant-a", "task-a", transition)
            .expect("restore receipt");
    let mut restored = BapTaskLifecycle::restore_with_transition(
        BapTaskState::DeadLettered,
        4,
        Some(transition),
    )
    .expect("restore lifecycle");
    let replay = restored
        .apply_or_replay(
            Some(&restored_receipt),
            "dead-letter-1",
            "tenant-a",
            "task-a",
            BapTaskEvent::DeadLetter,
        )
        .expect("replay dead-letter command");
    assert_eq!(replay, restored_receipt);

    assert_eq!(
        restored.apply(BapTaskEvent::ResolveReconciliation),
        Err(BapTaskTransitionError::TerminalState {
            state: BapTaskState::DeadLettered,
        })
    );
}
