#![allow(clippy::expect_used)]

use originweave_bap::{BapTaskEvent, BapTaskLifecycle, BapTaskState, BapTaskTransitionError};

const NON_TERMINAL_STATES: [BapTaskState; 7] = [
    BapTaskState::Created,
    BapTaskState::Admitted,
    BapTaskState::Running,
    BapTaskState::WaitingForApproval,
    BapTaskState::WaitingForExternalInput,
    BapTaskState::Checkpointed,
    BapTaskState::ReconciliationRequired,
];

const ALL_EVENTS: [BapTaskEvent; 13] = [
    BapTaskEvent::Admit,
    BapTaskEvent::Start,
    BapTaskEvent::WaitForApproval,
    BapTaskEvent::WaitForExternalInput,
    BapTaskEvent::Checkpoint,
    BapTaskEvent::Resume,
    BapTaskEvent::RequireReconciliation,
    BapTaskEvent::ResolveReconciliation,
    BapTaskEvent::DeadLetter,
    BapTaskEvent::Succeed,
    BapTaskEvent::Fail,
    BapTaskEvent::Cancel,
    BapTaskEvent::Expire,
];

fn task_in_state(target: BapTaskState) -> BapTaskLifecycle {
    let mut task = BapTaskLifecycle::new();
    if target == BapTaskState::Created {
        return task;
    }

    task.apply(BapTaskEvent::Admit).expect("admit");
    if target == BapTaskState::Admitted {
        return task;
    }

    task.apply(BapTaskEvent::Start).expect("start");
    match target {
        BapTaskState::Running => {}
        BapTaskState::WaitingForApproval => {
            task.apply(BapTaskEvent::WaitForApproval)
                .expect("wait approval");
        }
        BapTaskState::WaitingForExternalInput => {
            task.apply(BapTaskEvent::WaitForExternalInput)
                .expect("wait external");
        }
        BapTaskState::Checkpointed => {
            task.apply(BapTaskEvent::Checkpoint).expect("checkpoint");
        }
        BapTaskState::ReconciliationRequired => {
            task.apply(BapTaskEvent::RequireReconciliation)
                .expect("require reconciliation");
        }
        BapTaskState::Created
        | BapTaskState::Admitted
        | BapTaskState::Succeeded
        | BapTaskState::Failed
        | BapTaskState::Cancelled
        | BapTaskState::Expired
        | BapTaskState::DeadLettered => {
            unreachable!("task_in_state only constructs non-terminal lifecycle states")
        }
    }
    task
}

fn expected_next(state: BapTaskState, event: BapTaskEvent) -> Option<BapTaskState> {
    use BapTaskEvent::{
        Admit, Cancel, Checkpoint, DeadLetter, Expire, Fail, RequireReconciliation,
        ResolveReconciliation, Resume, Start, Succeed, WaitForApproval, WaitForExternalInput,
    };
    use BapTaskState::{
        Admitted, Cancelled, Checkpointed, Created, DeadLettered, Expired, Failed,
        ReconciliationRequired, Running, Succeeded, WaitingForApproval, WaitingForExternalInput,
    };

    match event {
        Fail => Some(Failed),
        Cancel => Some(Cancelled),
        Expire => Some(Expired),
        Admit => (state == Created).then_some(Admitted),
        Start => (state == Admitted).then_some(Running),
        WaitForApproval => (state == Running).then_some(WaitingForApproval),
        WaitForExternalInput => (state == Running).then_some(WaitingForExternalInput),
        Checkpoint => (state == Running).then_some(Checkpointed),
        Resume => matches!(
            state,
            WaitingForApproval | WaitingForExternalInput | Checkpointed
        )
        .then_some(Running),
        RequireReconciliation => (state == Running).then_some(ReconciliationRequired),
        ResolveReconciliation => (state == ReconciliationRequired).then_some(Running),
        DeadLetter => matches!(state, Running | ReconciliationRequired).then_some(DeadLettered),
        Succeed => (state == Running).then_some(Succeeded),
    }
}

#[test]
fn every_non_terminal_state_event_pair_matches_the_reviewed_transition_table() {
    for state in NON_TERMINAL_STATES {
        for event in ALL_EVENTS {
            let mut task = task_in_state(state);
            assert_eq!(task.state(), state);
            let before_sequence = task.transition_sequence();

            match expected_next(state, event) {
                Some(next_state) => {
                    let applied = task.apply(event);
                    assert!(
                        applied.is_ok(),
                        "{state:?} + {event:?} rejected: {applied:?}"
                    );
                    let receipt = applied.expect("reviewed transition must be accepted");
                    assert_eq!(receipt.previous_state(), state);
                    assert_eq!(receipt.current_state(), next_state);
                    assert_eq!(receipt.event(), event);
                    assert_eq!(receipt.sequence(), before_sequence + 1);
                    assert_eq!(task.state(), next_state);
                    assert_eq!(task.transition_sequence(), before_sequence + 1);
                }
                None => {
                    assert_eq!(
                        task.apply(event),
                        Err(BapTaskTransitionError::InvalidTransition { from: state, event }),
                        "{state:?} + {event:?} must fail closed",
                    );
                    assert_eq!(task.state(), state);
                    assert_eq!(task.transition_sequence(), before_sequence);
                }
            }
        }
    }
}

#[test]
fn every_terminal_state_rejects_every_event_without_advancing_history() {
    let entering_events = [
        (BapTaskState::Succeeded, BapTaskEvent::Succeed),
        (BapTaskState::Failed, BapTaskEvent::Fail),
        (BapTaskState::Cancelled, BapTaskEvent::Cancel),
        (BapTaskState::Expired, BapTaskEvent::Expire),
        (BapTaskState::DeadLettered, BapTaskEvent::DeadLetter),
    ];

    for (state, entering_event) in entering_events {
        for event in ALL_EVENTS {
            let mut task = task_in_state(BapTaskState::Running);
            task.apply(entering_event).expect("enter terminal state");
            assert_eq!(task.state(), state, "test fixture must reach {state:?}");
            let terminal_sequence = task.transition_sequence();

            assert_eq!(
                task.apply(event),
                Err(BapTaskTransitionError::TerminalState { state }),
                "{state:?} + {event:?} must stay terminal",
            );
            assert_eq!(task.state(), state);
            assert_eq!(task.transition_sequence(), terminal_sequence);
        }
    }
}
