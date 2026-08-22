use originweave_bap::{
    BapCommandReceipt, BapCommandReceiptError, BapCommandRecovery, BapExternalSideEffectOutcome,
    BapRecoveryAction, BapTaskEvent, BapTaskLifecycle, BapTaskState,
};

fn accepted_receipt() -> (BapTaskLifecycle, BapCommandReceipt) {
    let mut lifecycle = BapTaskLifecycle::new();
    let receipt =
        lifecycle.apply_with_receipt("retry-key", "tenant-a", "task-a", BapTaskEvent::Admit);
    assert!(receipt.is_ok(), "{receipt:?}");
    let Ok(receipt) = receipt else {
        unreachable!("asserted valid command receipt")
    };
    (lifecycle, receipt)
}

#[test]
fn crash_recovery_distinguishes_external_side_effect_outcomes_without_unsafe_replay() {
    let cases = [
        (
            BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
            BapRecoveryAction::RevalidateBeforeRedispatch,
            true,
        ),
        (
            BapExternalSideEffectOutcome::ConfirmedSideEffect,
            BapRecoveryAction::VerifyConfirmedSideEffect,
            false,
        ),
        (
            BapExternalSideEffectOutcome::UnknownOutcome,
            BapRecoveryAction::ReconcileBeforeFurtherAction,
            false,
        ),
        (
            BapExternalSideEffectOutcome::ReconciliationRequired,
            BapRecoveryAction::ReconcileBeforeFurtherAction,
            false,
        ),
    ];

    for (outcome, expected_action, expected_redispatch) in cases {
        let (lifecycle, receipt) = accepted_receipt();
        let recovery = BapCommandRecovery::new(receipt, outcome);
        assert_eq!(recovery.external_outcome(), outcome);
        assert_eq!(recovery.required_action(), expected_action);
        assert_eq!(
            recovery.permits_redispatch(&lifecycle),
            Ok(expected_redispatch)
        );
        assert_eq!(lifecycle.state(), BapTaskState::Admitted);
        assert_eq!(lifecycle.transition_sequence(), 1);
        assert_eq!(recovery.receipt().task_id(), "task-a");

        let debug = format!("{recovery:?}");
        assert!(!debug.contains("retry-key"));
        assert!(!debug.contains("tenant-a"));
        assert!(!debug.contains("task-a"));
    }
}

#[test]
fn recovery_validation_requires_only_read_only_lifecycle_access() {
    let (lifecycle, receipt) = accepted_receipt();
    let recovery =
        BapCommandRecovery::new(receipt, BapExternalSideEffectOutcome::ConfirmedNoSideEffect);

    assert_eq!(recovery.permits_redispatch(&lifecycle), Ok(true));
    assert_eq!(lifecycle.state(), BapTaskState::Admitted);
    assert_eq!(lifecycle.transition_sequence(), 1);
}

#[test]
fn stale_recovery_receipt_cannot_signal_redispatch() {
    let (mut lifecycle, receipt) = accepted_receipt();
    let recovery =
        BapCommandRecovery::new(receipt, BapExternalSideEffectOutcome::ConfirmedNoSideEffect);

    let advance = lifecycle.apply(BapTaskEvent::Start);
    assert!(advance.is_ok(), "{advance:?}");

    assert_eq!(
        recovery.permits_redispatch(&lifecycle),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(lifecycle.state(), BapTaskState::Running);
    assert_eq!(lifecycle.transition_sequence(), 2);
}

#[test]
fn terminal_lifecycle_never_signals_redispatch_even_for_confirmed_no_side_effect() {
    for terminal_event in [
        BapTaskEvent::Succeed,
        BapTaskEvent::Fail,
        BapTaskEvent::Cancel,
        BapTaskEvent::Expire,
        BapTaskEvent::DeadLetter,
    ] {
        let mut lifecycle = BapTaskLifecycle::new();
        let admit = lifecycle.apply(BapTaskEvent::Admit);
        assert!(admit.is_ok(), "{admit:?}");
        let start = lifecycle.apply(BapTaskEvent::Start);
        assert!(start.is_ok(), "{start:?}");

        let receipt = lifecycle.apply_with_receipt(
            "terminal-retry-key",
            "tenant-a",
            "task-a",
            terminal_event,
        );
        assert!(receipt.is_ok(), "{receipt:?}");
        let Ok(receipt) = receipt else {
            unreachable!("asserted valid terminal command receipt")
        };
        assert!(lifecycle.state().is_terminal());

        let recovery =
            BapCommandRecovery::new(receipt, BapExternalSideEffectOutcome::ConfirmedNoSideEffect);
        assert_eq!(recovery.permits_redispatch(&lifecycle), Ok(false));
    }
}
