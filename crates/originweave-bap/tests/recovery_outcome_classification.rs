use originweave_bap::{
    BapCommandRecovery, BapExternalSideEffectOutcome, BapRecoveryAction, BapTaskEvent,
    BapTaskLifecycle,
};

fn accepted_receipt() -> originweave_bap::BapCommandReceipt {
    let mut lifecycle = BapTaskLifecycle::new();
    let receipt = lifecycle.apply_with_receipt(
        "retry-key",
        "tenant-a",
        "task-a",
        BapTaskEvent::Admit,
    );
    assert!(receipt.is_ok(), "{receipt:?}");
    let Ok(receipt) = receipt else {
        unreachable!("asserted valid command receipt")
    };
    receipt
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
        let recovery = BapCommandRecovery::new(accepted_receipt(), outcome);
        assert_eq!(recovery.external_outcome(), outcome);
        assert_eq!(recovery.required_action(), expected_action);
        assert_eq!(recovery.permits_redispatch(), expected_redispatch);
        assert_eq!(recovery.receipt().task_id(), "task-a");

        let debug = format!("{recovery:?}");
        assert!(!debug.contains("retry-key"));
        assert!(!debug.contains("tenant-a"));
        assert!(!debug.contains("task-a"));
    }
}
