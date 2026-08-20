#![allow(clippy::expect_used)]

use originweave_bap::{BapTaskEvent, BapTaskLifecycle, BapTaskState, BapTaskTransitionError};

#[test]
fn restored_lifecycle_preserves_state_and_monotonic_sequence() {
    let mut task = BapTaskLifecycle::restore(BapTaskState::Checkpointed, 41);

    assert_eq!(task.state(), BapTaskState::Checkpointed);
    assert_eq!(task.transition_sequence(), 41);

    let resumed = task.apply(BapTaskEvent::Resume).expect("resume restored task");
    assert_eq!(resumed.previous_state(), BapTaskState::Checkpointed);
    assert_eq!(resumed.current_state(), BapTaskState::Running);
    assert_eq!(resumed.sequence(), 42);
}

#[test]
fn exhausted_sequence_fails_closed_without_mutating_state() {
    let mut task = BapTaskLifecycle::restore(BapTaskState::Checkpointed, u64::MAX);

    assert_eq!(
        task.apply(BapTaskEvent::Resume),
        Err(BapTaskTransitionError::SequenceExhausted),
    );
    assert_eq!(task.state(), BapTaskState::Checkpointed);
    assert_eq!(task.transition_sequence(), u64::MAX);
}

#[test]
fn restored_terminal_lifecycle_remains_terminal() {
    let mut task = BapTaskLifecycle::restore(BapTaskState::Succeeded, 9);

    assert_eq!(
        task.apply(BapTaskEvent::Resume),
        Err(BapTaskTransitionError::TerminalState {
            state: BapTaskState::Succeeded,
        }),
    );
    assert_eq!(task.transition_sequence(), 9);
}
