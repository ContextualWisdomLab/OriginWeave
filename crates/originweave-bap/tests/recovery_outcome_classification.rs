use originweave_bap::{
    BapCommandReceipt, BapCommandReceiptError, BapCommandRecovery, BapCommandRecoveryError,
    BapExternalSideEffectOutcome, BapRecoveryAction, BapRecoveryDisposition,
    BapRecoveryEvidenceDigest, BapRecoveryEvidenceDigestError, BapTaskEvent, BapTaskLifecycle,
    BapTaskState,
};

const RECOVERY_EVIDENCE_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_RECOVERY_EVIDENCE_DIGEST: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

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

fn recovery_evidence_digest() -> BapRecoveryEvidenceDigest {
    let digest = BapRecoveryEvidenceDigest::parse(RECOVERY_EVIDENCE_DIGEST);
    assert!(digest.is_ok(), "{digest:?}");
    let Ok(digest) = digest else {
        unreachable!("asserted valid recovery evidence digest")
    };
    digest
}

fn other_recovery_evidence_digest() -> BapRecoveryEvidenceDigest {
    let digest = BapRecoveryEvidenceDigest::parse(OTHER_RECOVERY_EVIDENCE_DIGEST);
    assert!(digest.is_ok(), "{digest:?}");
    let Ok(digest) = digest else {
        unreachable!("asserted valid alternate recovery evidence digest")
    };
    digest
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
        let recovery = BapCommandRecovery::new(receipt, outcome, recovery_evidence_digest());
        assert_eq!(recovery.external_outcome(), outcome);
        assert_eq!(recovery.required_action(), expected_action);
        assert_eq!(
            recovery.permits_redispatch(&lifecycle, recovery.evidence_digest()),
            Ok(expected_redispatch)
        );
        assert_eq!(lifecycle.state(), BapTaskState::Admitted);
        assert_eq!(lifecycle.transition_sequence(), 1);
        assert_eq!(recovery.receipt().task_id(), "task-a");
        assert_eq!(
            recovery.evidence_digest().as_str(),
            RECOVERY_EVIDENCE_DIGEST
        );

        let debug = format!("{recovery:?}");
        assert!(!debug.contains("retry-key"));
        assert!(!debug.contains("tenant-a"));
        assert!(!debug.contains("task-a"));
    }
}

#[test]
fn lifecycle_aware_disposition_is_the_single_fail_closed_recovery_decision() {
    let cases = [
        (
            BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
            BapRecoveryDisposition::RevalidateBeforeRedispatch,
        ),
        (
            BapExternalSideEffectOutcome::ConfirmedSideEffect,
            BapRecoveryDisposition::VerifyConfirmedSideEffect,
        ),
        (
            BapExternalSideEffectOutcome::UnknownOutcome,
            BapRecoveryDisposition::ReconcileBeforeFurtherAction,
        ),
        (
            BapExternalSideEffectOutcome::ReconciliationRequired,
            BapRecoveryDisposition::ReconcileBeforeFurtherAction,
        ),
    ];

    for (outcome, expected_disposition) in cases {
        let (lifecycle, receipt) = accepted_receipt();
        let recovery = BapCommandRecovery::new(receipt, outcome, recovery_evidence_digest());
        assert_eq!(
            recovery.disposition(&lifecycle, recovery.evidence_digest()),
            Ok(expected_disposition)
        );
    }

    let suspended_cases = [
        (
            BapTaskEvent::WaitForApproval,
            BapTaskState::WaitingForApproval,
        ),
        (
            BapTaskEvent::WaitForExternalInput,
            BapTaskState::WaitingForExternalInput,
        ),
        (BapTaskEvent::Checkpoint, BapTaskState::Checkpointed),
        (
            BapTaskEvent::RequireReconciliation,
            BapTaskState::ReconciliationRequired,
        ),
    ];

    for (event, expected_state) in suspended_cases {
        let mut lifecycle = BapTaskLifecycle::new();
        assert!(lifecycle.apply(BapTaskEvent::Admit).is_ok());
        assert!(lifecycle.apply(BapTaskEvent::Start).is_ok());
        let receipt =
            lifecycle.apply_with_receipt("blocked-retry-key", "tenant-a", "task-a", event);
        assert!(receipt.is_ok(), "{receipt:?}");
        let Ok(receipt) = receipt else {
            unreachable!("asserted valid blocked-state command receipt")
        };
        let recovery = BapCommandRecovery::new(
            receipt,
            BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
            recovery_evidence_digest(),
        );
        assert_eq!(
            recovery.disposition(&lifecycle, recovery.evidence_digest()),
            Ok(BapRecoveryDisposition::RedispatchBlockedByLifecycle {
                state: expected_state
            })
        );
        assert_eq!(
            recovery.permits_redispatch(&lifecycle, recovery.evidence_digest()),
            Ok(false)
        );
    }

    let mut terminal_lifecycle = BapTaskLifecycle::new();
    assert!(terminal_lifecycle.apply(BapTaskEvent::Admit).is_ok());
    assert!(terminal_lifecycle.apply(BapTaskEvent::Start).is_ok());
    let terminal_receipt = terminal_lifecycle.apply_with_receipt(
        "terminal-decision-key",
        "tenant-a",
        "task-a",
        BapTaskEvent::Fail,
    );
    assert!(terminal_receipt.is_ok(), "{terminal_receipt:?}");
    let Ok(terminal_receipt) = terminal_receipt else {
        unreachable!("asserted valid terminal command receipt")
    };
    let terminal_recovery = BapCommandRecovery::new(
        terminal_receipt,
        BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
        recovery_evidence_digest(),
    );
    assert_eq!(
        terminal_recovery.disposition(&terminal_lifecycle, terminal_recovery.evidence_digest()),
        Ok(BapRecoveryDisposition::RedispatchBlockedByLifecycle {
            state: BapTaskState::Failed
        })
    );

    let (mut stale_lifecycle, stale_receipt) = accepted_receipt();
    assert!(stale_lifecycle.apply(BapTaskEvent::Start).is_ok());
    let stale_recovery = BapCommandRecovery::new(
        stale_receipt,
        BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
        recovery_evidence_digest(),
    );
    assert_eq!(
        stale_recovery.disposition(&stale_lifecycle, stale_recovery.evidence_digest()),
        Err(BapCommandRecoveryError::ReceiptValidation {
            error: BapCommandReceiptError::ReplayStateMismatch,
        })
    );
}

#[test]
fn recovery_evidence_identity_must_match_before_lifecycle_state_is_considered() {
    let (mut lifecycle, receipt) = accepted_receipt();
    let recovery = BapCommandRecovery::new(
        receipt,
        BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
        recovery_evidence_digest(),
    );
    let alternate_evidence = other_recovery_evidence_digest();

    assert!(lifecycle.apply(BapTaskEvent::Start).is_ok());
    assert_eq!(
        recovery.disposition(&lifecycle, &alternate_evidence),
        Err(BapCommandRecoveryError::EvidenceDigestMismatch)
    );
    assert_eq!(
        recovery.permits_redispatch(&lifecycle, &alternate_evidence),
        Err(BapCommandRecoveryError::EvidenceDigestMismatch)
    );
    assert_eq!(lifecycle.state(), BapTaskState::Running);
    assert_eq!(lifecycle.transition_sequence(), 2);

    let mismatch = BapCommandRecoveryError::EvidenceDigestMismatch;
    assert_eq!(
        mismatch.to_string(),
        "BAP recovery evidence digest does not match the retained recovery classification"
    );
    assert!(std::error::Error::source(&mismatch).is_none());

    let receipt_failure = BapCommandRecoveryError::ReceiptValidation {
        error: BapCommandReceiptError::ReplayStateMismatch,
    };
    assert_eq!(
        receipt_failure.to_string(),
        BapCommandReceiptError::ReplayStateMismatch.to_string()
    );
    assert!(std::error::Error::source(&receipt_failure).is_some());
}

#[test]
fn recovery_evidence_digest_requires_exact_lowercase_sha256_identity() {
    let valid = recovery_evidence_digest();
    assert_eq!(valid.as_str(), RECOVERY_EVIDENCE_DIGEST);

    for invalid in [
        "",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "sha256_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "sha512:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        assert_eq!(
            BapRecoveryEvidenceDigest::parse(invalid),
            Err(BapRecoveryEvidenceDigestError::InvalidFormat)
        );
    }

    let error = BapRecoveryEvidenceDigestError::InvalidFormat;
    assert_eq!(
        error.to_string(),
        "recovery evidence digest must be sha256: followed by 64 lowercase hexadecimal digits"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn recovery_validation_requires_only_read_only_lifecycle_access() {
    let (lifecycle, receipt) = accepted_receipt();
    let recovery = BapCommandRecovery::new(
        receipt,
        BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
        recovery_evidence_digest(),
    );

    assert_eq!(
        recovery.permits_redispatch(&lifecycle, recovery.evidence_digest()),
        Ok(true)
    );
    assert_eq!(lifecycle.state(), BapTaskState::Admitted);
    assert_eq!(lifecycle.transition_sequence(), 1);
}

#[test]
fn stale_recovery_receipt_cannot_signal_redispatch() {
    let (mut lifecycle, receipt) = accepted_receipt();
    let recovery = BapCommandRecovery::new(
        receipt,
        BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
        recovery_evidence_digest(),
    );

    let advance = lifecycle.apply(BapTaskEvent::Start);
    assert!(advance.is_ok(), "{advance:?}");

    assert_eq!(
        recovery.permits_redispatch(&lifecycle, recovery.evidence_digest()),
        Err(BapCommandRecoveryError::ReceiptValidation {
            error: BapCommandReceiptError::ReplayStateMismatch,
        })
    );
    assert_eq!(lifecycle.state(), BapTaskState::Running);
    assert_eq!(lifecycle.transition_sequence(), 2);
}

#[test]
fn reconciliation_hold_never_signals_redispatch_before_explicit_resolution() {
    let mut lifecycle = BapTaskLifecycle::new();
    let admit = lifecycle.apply(BapTaskEvent::Admit);
    assert!(admit.is_ok(), "{admit:?}");
    let start = lifecycle.apply(BapTaskEvent::Start);
    assert!(start.is_ok(), "{start:?}");

    let receipt = lifecycle.apply_with_receipt(
        "reconcile-retry-key",
        "tenant-a",
        "task-a",
        BapTaskEvent::RequireReconciliation,
    );
    assert!(receipt.is_ok(), "{receipt:?}");
    let Ok(receipt) = receipt else {
        unreachable!("asserted valid reconciliation command receipt")
    };
    assert_eq!(lifecycle.state(), BapTaskState::ReconciliationRequired);
    assert_eq!(lifecycle.transition_sequence(), 3);

    let recovery = BapCommandRecovery::new(
        receipt,
        BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
        recovery_evidence_digest(),
    );
    assert_eq!(
        recovery.permits_redispatch(&lifecycle, recovery.evidence_digest()),
        Ok(false)
    );
    assert_eq!(lifecycle.state(), BapTaskState::ReconciliationRequired);
    assert_eq!(lifecycle.transition_sequence(), 3);
}

#[test]
fn suspended_lifecycle_never_signals_redispatch_before_resume() {
    let cases = [
        (
            BapTaskEvent::WaitForApproval,
            BapTaskState::WaitingForApproval,
        ),
        (
            BapTaskEvent::WaitForExternalInput,
            BapTaskState::WaitingForExternalInput,
        ),
        (BapTaskEvent::Checkpoint, BapTaskState::Checkpointed),
    ];

    for (suspend_event, expected_state) in cases {
        let mut lifecycle = BapTaskLifecycle::new();
        let admit = lifecycle.apply(BapTaskEvent::Admit);
        assert!(admit.is_ok(), "{admit:?}");
        let start = lifecycle.apply(BapTaskEvent::Start);
        assert!(start.is_ok(), "{start:?}");

        let receipt = lifecycle.apply_with_receipt(
            "suspended-retry-key",
            "tenant-a",
            "task-a",
            suspend_event,
        );
        assert!(receipt.is_ok(), "{receipt:?}");
        let Ok(receipt) = receipt else {
            unreachable!("asserted valid suspended-state command receipt")
        };
        assert_eq!(lifecycle.state(), expected_state);

        let recovery = BapCommandRecovery::new(
            receipt,
            BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
            recovery_evidence_digest(),
        );
        assert_eq!(
            recovery.permits_redispatch(&lifecycle, recovery.evidence_digest()),
            Ok(false)
        );
        assert_eq!(lifecycle.state(), expected_state);
    }
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

        let recovery = BapCommandRecovery::new(
            receipt,
            BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
            recovery_evidence_digest(),
        );
        assert_eq!(
            recovery.permits_redispatch(&lifecycle, recovery.evidence_digest()),
            Ok(false)
        );
    }
}
