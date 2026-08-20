#![allow(clippy::expect_used)]

use std::error::Error as _;

use originweave_bap::{
    BapTaskEvent, BapTaskLifecycle, BapTaskRestoreError, BapTaskState, BapTaskTransitionError,
};

#[test]
fn restored_lifecycle_preserves_state_and_monotonic_sequence() {
    let mut task = BapTaskLifecycle::restore(BapTaskState::Checkpointed, 41)
        .expect("valid checkpoint snapshot");

    assert_eq!(task.state(), BapTaskState::Checkpointed);
    assert_eq!(task.transition_sequence(), 41);

    let resumed = task
        .apply(BapTaskEvent::Resume)
        .expect("resume restored task");
    assert_eq!(resumed.previous_state(), BapTaskState::Checkpointed);
    assert_eq!(resumed.current_state(), BapTaskState::Running);
    assert_eq!(resumed.sequence(), 42);
}

#[test]
fn impossible_restored_snapshots_fail_closed() {
    for (state, sequence) in [
        (BapTaskState::Created, 1),
        (BapTaskState::Admitted, 0),
        (BapTaskState::Admitted, 2),
        (BapTaskState::Running, 1),
        (BapTaskState::Running, 3),
        (BapTaskState::WaitingForApproval, 2),
        (BapTaskState::WaitingForApproval, 4),
        (BapTaskState::WaitingForExternalInput, 2),
        (BapTaskState::WaitingForExternalInput, 4),
        (BapTaskState::Checkpointed, 2),
        (BapTaskState::Checkpointed, 4),
        (BapTaskState::Succeeded, 2),
        (BapTaskState::Succeeded, 4),
        (BapTaskState::Failed, 0),
        (BapTaskState::Cancelled, 0),
        (BapTaskState::Expired, 0),
    ] {
        assert_eq!(
            BapTaskLifecycle::restore(state, sequence),
            Err(BapTaskRestoreError::InvalidSnapshot {
                state,
                transition_sequence: sequence,
            }),
            "state={state:?}, sequence={sequence}",
        );
    }
}

#[test]
fn valid_restored_snapshot_classes_remain_accepted() {
    for (state, sequence) in [
        (BapTaskState::Created, 0),
        (BapTaskState::Admitted, 1),
        (BapTaskState::Running, 2),
        (BapTaskState::Running, 4),
        (BapTaskState::WaitingForApproval, 3),
        (BapTaskState::WaitingForExternalInput, 5),
        (BapTaskState::Checkpointed, 7),
        (BapTaskState::Succeeded, 3),
        (BapTaskState::Failed, 1),
        (BapTaskState::Cancelled, 2),
        (BapTaskState::Expired, 4),
    ] {
        let task = BapTaskLifecycle::restore(state, sequence).expect("reachable snapshot");
        assert_eq!(task.state(), state);
        assert_eq!(task.transition_sequence(), sequence);
    }
}

#[test]
fn exhausted_sequence_fails_closed_without_mutating_state() {
    let mut task = BapTaskLifecycle::restore(BapTaskState::Checkpointed, u64::MAX)
        .expect("valid exhausted checkpoint snapshot");

    assert_eq!(
        task.apply(BapTaskEvent::Resume),
        Err(BapTaskTransitionError::SequenceExhausted),
    );
    assert_eq!(task.state(), BapTaskState::Checkpointed);
    assert_eq!(task.transition_sequence(), u64::MAX);
}

#[test]
fn restored_terminal_lifecycle_remains_terminal() {
    let mut task =
        BapTaskLifecycle::restore(BapTaskState::Succeeded, 9).expect("valid terminal snapshot");

    assert_eq!(
        task.apply(BapTaskEvent::Resume),
        Err(BapTaskTransitionError::TerminalState {
            state: BapTaskState::Succeeded,
        }),
    );
    assert_eq!(task.transition_sequence(), 9);
}

#[test]
fn lifecycle_failures_use_the_standard_rust_error_contract() {
    let mut created = BapTaskLifecycle::new();
    let invalid_transition = created
        .apply(BapTaskEvent::Start)
        .expect_err("created task must reject start");
    assert_eq!(
        invalid_transition.to_string(),
        "BAP task event Start is invalid from state Created"
    );
    assert!(invalid_transition.source().is_none());

    let exhausted = BapTaskTransitionError::SequenceExhausted;
    assert_eq!(
        exhausted.to_string(),
        "BAP task transition sequence is exhausted"
    );
    assert!(exhausted.source().is_none());

    let terminal = BapTaskTransitionError::TerminalState {
        state: BapTaskState::Cancelled,
    };
    assert_eq!(terminal.to_string(), "BAP task state Cancelled is terminal");
    assert!(terminal.source().is_none());

    let restore = BapTaskLifecycle::restore(BapTaskState::Created, 1)
        .expect_err("unreachable snapshot must fail");
    assert_eq!(
        restore.to_string(),
        "BAP task snapshot state Created with transition sequence 1 is unreachable"
    );
    assert!(restore.source().is_none());
}
