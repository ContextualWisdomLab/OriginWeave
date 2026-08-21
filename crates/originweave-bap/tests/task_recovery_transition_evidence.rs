#![allow(clippy::expect_used)]

use originweave_bap::{
    BapCommandReceipt, BapTaskEvent, BapTaskLifecycle, BapTaskRestoreError, BapTaskState,
    BapTaskTransition,
};

#[test]
fn exact_transition_evidence_restores_receipt_replay_without_second_mutation() {
    let mut lifecycle = BapTaskLifecycle::new();
    let receipt = lifecycle
        .apply_with_receipt("retry-1", "tenant-a", "task-a", BapTaskEvent::Admit)
        .expect("initial command must be accepted");
    let accepted = receipt.transition();

    let restored_transition = BapTaskTransition::restore(
        accepted.previous_state(),
        accepted.current_state(),
        accepted.sequence(),
        accepted.event(),
    )
    .expect("exact persisted transition evidence must restore");
    let mut restored = BapTaskLifecycle::restore_with_transition(
        accepted.current_state(),
        accepted.sequence(),
        Some(restored_transition),
    )
    .expect("exact state and transition evidence must restore the lifecycle");

    let replay = restored
        .apply_or_replay(
            Some(&receipt),
            "retry-1",
            "tenant-a",
            "task-a",
            BapTaskEvent::Admit,
        )
        .expect("exact retry must replay after authenticated transition recovery");

    assert_eq!(replay, receipt);
    assert_eq!(restored.state(), BapTaskState::Admitted);
    assert_eq!(restored.transition_sequence(), 1);
}

#[test]
fn persisted_receipt_fields_can_be_reconstructed_for_cross_process_replay() {
    let mut lifecycle = BapTaskLifecycle::new();
    let issued = lifecycle
        .apply_with_receipt("retry-1", "tenant-a", "task-a", BapTaskEvent::Admit)
        .expect("initial command must be accepted");
    let accepted = issued.transition();

    let restored_transition = BapTaskTransition::restore(
        accepted.previous_state(),
        accepted.current_state(),
        accepted.sequence(),
        accepted.event(),
    )
    .expect("persisted transition evidence must restore");
    let restored_receipt = BapCommandReceipt::restore(
        issued.idempotency_key(),
        issued.tenant_id(),
        issued.task_id(),
        restored_transition,
    )
    .expect("persisted receipt fields must restore after process loss");
    let mut restored_lifecycle = BapTaskLifecycle::restore_with_transition(
        accepted.current_state(),
        accepted.sequence(),
        Some(restored_transition),
    )
    .expect("persisted lifecycle evidence must restore");

    let replay = restored_lifecycle
        .apply_or_replay(
            Some(&restored_receipt),
            "retry-1",
            "tenant-a",
            "task-a",
            BapTaskEvent::Admit,
        )
        .expect("restored receipt must replay without repeating the transition");

    assert_eq!(replay, restored_receipt);
    assert_eq!(restored_lifecycle.state(), BapTaskState::Admitted);
    assert_eq!(restored_lifecycle.transition_sequence(), 1);
}

#[test]
fn recovery_requires_transition_evidence_after_any_accepted_transition() {
    let invalid_snapshot = BapTaskRestoreError::InvalidSnapshot {
        state: BapTaskState::Created,
        transition_sequence: 1,
    };
    assert_eq!(
        BapTaskLifecycle::restore_with_transition(BapTaskState::Created, 1, None),
        Err(invalid_snapshot)
    );

    let missing = BapTaskRestoreError::MissingTransitionEvidence {
        state: BapTaskState::Admitted,
        transition_sequence: 1,
    };
    assert_eq!(
        missing.to_string(),
        "BAP task snapshot state Admitted with transition sequence 1 is missing last-transition evidence"
    );
    assert_eq!(
        BapTaskLifecycle::restore_with_transition(BapTaskState::Admitted, 1, None),
        Err(missing)
    );

    assert_eq!(
        BapTaskLifecycle::restore_with_transition(BapTaskState::Created, 0, None)
            .expect("created state needs no prior transition")
            .transition_sequence(),
        0
    );
}

#[test]
fn transition_restore_rejects_zero_unreachable_invalid_and_mismatched_evidence() {
    let invalid = |state, transition_sequence| BapTaskRestoreError::InvalidTransitionEvidence {
        state,
        transition_sequence,
    };
    assert_eq!(
        invalid(BapTaskState::Running, 2).to_string(),
        "BAP task transition evidence for state Running with transition sequence 2 is invalid"
    );

    assert_eq!(
        BapTaskTransition::restore(
            BapTaskState::Created,
            BapTaskState::Admitted,
            0,
            BapTaskEvent::Admit,
        ),
        Err(invalid(BapTaskState::Admitted, 0))
    );
    assert_eq!(
        BapTaskTransition::restore(
            BapTaskState::Admitted,
            BapTaskState::Running,
            4,
            BapTaskEvent::Start,
        ),
        Err(invalid(BapTaskState::Running, 4))
    );
    assert_eq!(
        BapTaskTransition::restore(
            BapTaskState::Created,
            BapTaskState::Running,
            1,
            BapTaskEvent::Start,
        ),
        Err(invalid(BapTaskState::Running, 1))
    );
    assert_eq!(
        BapTaskTransition::restore(
            BapTaskState::Created,
            BapTaskState::Running,
            1,
            BapTaskEvent::Admit,
        ),
        Err(invalid(BapTaskState::Running, 1))
    );
}

#[test]
fn lifecycle_restore_rejects_transition_from_a_different_snapshot() {
    let mut lifecycle = BapTaskLifecycle::new();
    let admitted = lifecycle
        .apply(BapTaskEvent::Admit)
        .expect("admit must succeed");

    assert_eq!(
        BapTaskLifecycle::restore_with_transition(BapTaskState::Created, 0, Some(admitted)),
        Err(BapTaskRestoreError::InvalidTransitionEvidence {
            state: BapTaskState::Created,
            transition_sequence: 0,
        })
    );

    lifecycle
        .apply(BapTaskEvent::Start)
        .expect("start must succeed");
    assert_eq!(
        BapTaskLifecycle::restore_with_transition(BapTaskState::Running, 2, Some(admitted)),
        Err(BapTaskRestoreError::InvalidTransitionEvidence {
            state: BapTaskState::Running,
            transition_sequence: 2,
        })
    );

    lifecycle
        .apply(BapTaskEvent::WaitForApproval)
        .expect("wait must succeed");
    let resumed = lifecycle
        .apply(BapTaskEvent::Resume)
        .expect("resume must succeed");
    assert_eq!(
        BapTaskLifecycle::restore_with_transition(BapTaskState::Running, 2, Some(resumed)),
        Err(BapTaskRestoreError::InvalidTransitionEvidence {
            state: BapTaskState::Running,
            transition_sequence: 2,
        })
    );
}
