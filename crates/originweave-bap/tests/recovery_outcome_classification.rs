use originweave_bap::{
    BapCommandReceipt, BapCommandReceiptError, BapCommandRecovery, BapExternalSideEffectOutcome,
    BapRecoveryAction, BapRecoveryEvidenceDigest, BapRecoveryEvidenceDigestError, BapTaskEvent,
    BapTaskLifecycle, BapTaskState,
};

const RECOVERY_EVIDENCE_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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
            recovery.permits_redispatch(&lifecycle),
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

    assert_eq!(recovery.permits_redispatch(&lifecycle), Ok(true));
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

        let recovery = BapCommandRecovery::new(
            receipt,
            BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
            recovery_evidence_digest(),
        );
        assert_eq!(recovery.permits_redispatch(&lifecycle), Ok(false));
    }
}
