#![allow(clippy::expect_used)]

use originweave_core::{
    BapTaskEvent, BapTaskLifecycle, BapTaskState, BapTaskTransitionError,
};

#[test]
fn bap_task_lifecycle_follows_the_reviewed_resumable_path() {
    let mut task = BapTaskLifecycle::new();
    assert_eq!(task.state(), BapTaskState::Created);
    assert_eq!(task.transition_sequence(), 0);

    let admitted = task.apply(BapTaskEvent::Admit).expect("admit");
    assert_eq!(admitted.previous_state(), BapTaskState::Created);
    assert_eq!(admitted.current_state(), BapTaskState::Admitted);
    assert_eq!(admitted.sequence(), 1);

    task.apply(BapTaskEvent::Start).expect("start");
    task.apply(BapTaskEvent::WaitForApproval)
        .expect("wait for approval");
    assert_eq!(task.state(), BapTaskState::WaitingForApproval);

    task.apply(BapTaskEvent::Resume).expect("resume approval");
    task.apply(BapTaskEvent::Checkpoint).expect("checkpoint");
    assert_eq!(task.state(), BapTaskState::Checkpointed);

    task.apply(BapTaskEvent::Resume).expect("resume checkpoint");
    let succeeded = task.apply(BapTaskEvent::Succeed).expect("succeed");
    assert_eq!(succeeded.current_state(), BapTaskState::Succeeded);
    assert!(task.state().is_terminal());
    assert_eq!(task.transition_sequence(), 7);
}

#[test]
fn waiting_for_external_input_can_resume_but_cannot_succeed_directly() {
    let mut task = running_task();
    task.apply(BapTaskEvent::WaitForExternalInput)
        .expect("wait for input");

    let error = task
        .apply(BapTaskEvent::Succeed)
        .expect_err("waiting task must not skip resume and post-condition work");
    assert_eq!(
        error,
        BapTaskTransitionError::InvalidTransition {
            from: BapTaskState::WaitingForExternalInput,
            event: BapTaskEvent::Succeed,
        }
    );
    assert_eq!(task.state(), BapTaskState::WaitingForExternalInput);
    assert_eq!(task.transition_sequence(), 3);

    task.apply(BapTaskEvent::Resume).expect("resume input");
    assert_eq!(task.state(), BapTaskState::Running);
}

#[test]
fn invalid_transition_is_fail_closed_and_does_not_advance_history() {
    let mut task = BapTaskLifecycle::new();

    let error = task
        .apply(BapTaskEvent::Start)
        .expect_err("created task must be admitted first");
    assert_eq!(
        error,
        BapTaskTransitionError::InvalidTransition {
            from: BapTaskState::Created,
            event: BapTaskEvent::Start,
        }
    );
    assert_eq!(task.state(), BapTaskState::Created);
    assert_eq!(task.transition_sequence(), 0);
}

#[test]
fn terminal_task_never_reopens_or_advances_history() {
    for terminal_event in [
        BapTaskEvent::Succeed,
        BapTaskEvent::Fail,
        BapTaskEvent::Cancel,
        BapTaskEvent::Expire,
    ] {
        let mut task = if terminal_event == BapTaskEvent::Succeed {
            running_task()
        } else {
            BapTaskLifecycle::new()
        };
        task.apply(terminal_event).expect("enter terminal state");
        let terminal_state = task.state();
        let terminal_sequence = task.transition_sequence();

        for later_event in [
            BapTaskEvent::Admit,
            BapTaskEvent::Start,
            BapTaskEvent::Resume,
            BapTaskEvent::Cancel,
        ] {
            assert_eq!(
                task.apply(later_event),
                Err(BapTaskTransitionError::TerminalState {
                    state: terminal_state,
                })
            );
            assert_eq!(task.state(), terminal_state);
            assert_eq!(task.transition_sequence(), terminal_sequence);
        }
    }
}

#[test]
fn cancellation_and_expiry_cover_pre_dispatch_and_suspended_states() {
    for (state, setup) in [
        (BapTaskState::Created, 0_u8),
        (BapTaskState::Admitted, 1),
        (BapTaskState::Running, 2),
        (BapTaskState::WaitingForApproval, 3),
        (BapTaskState::WaitingForExternalInput, 4),
        (BapTaskState::Checkpointed, 5),
    ] {
        for terminal_event in [BapTaskEvent::Cancel, BapTaskEvent::Expire] {
            let mut task = task_in_state(setup);
            assert_eq!(task.state(), state);
            task.apply(terminal_event).expect("terminal interruption");
            assert!(task.state().is_terminal());
        }
    }
}

fn running_task() -> BapTaskLifecycle {
    let mut task = BapTaskLifecycle::new();
    task.apply(BapTaskEvent::Admit).expect("admit");
    task.apply(BapTaskEvent::Start).expect("start");
    task
}

fn task_in_state(setup: u8) -> BapTaskLifecycle {
    let mut task = BapTaskLifecycle::new();
    if setup >= 1 {
        task.apply(BapTaskEvent::Admit).expect("admit");
    }
    if setup >= 2 {
        task.apply(BapTaskEvent::Start).expect("start");
    }
    match setup {
        3 => {
            task.apply(BapTaskEvent::WaitForApproval)
                .expect("wait approval");
        }
        4 => {
            task.apply(BapTaskEvent::WaitForExternalInput)
                .expect("wait external");
        }
        5 => {
            task.apply(BapTaskEvent::Checkpoint).expect("checkpoint");
        }
        _ => {}
    }
    task
}
