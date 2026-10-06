#![allow(clippy::expect_used)]

use originweave_bap::{
    BapCommandReceipt, BapCommandReceiptError, BapTaskEvent, BapTaskLifecycle, BapTaskTransition,
};

/// Characterize caller-trusted reconstruction, not authenticated original issuance.
///
/// Valid replacement metadata is preserved with the supplied validated transition. Replay
/// compares those supplied values and the current last transition without accepting a new event.
#[test]
fn reconstruction_preserves_supplied_metadata_and_replays_without_mutation() {
    let mut task = BapTaskLifecycle::new();
    let issued = task
        .apply_with_receipt("request-1", "tenant-1", "task-1", BapTaskEvent::Admit)
        .expect("normal issuance");
    let accepted = issued.transition();
    let transition = BapTaskTransition::restore(
        accepted.previous_state(),
        accepted.current_state(),
        accepted.sequence(),
        accepted.event(),
    )
    .expect("validated transition reconstruction");
    let reconstructed = BapCommandReceipt::restore("request-2", "tenant-2", "task-2", transition)
        .expect("caller-trusted metadata reconstruction");
    assert_eq!(reconstructed.idempotency_key(), "request-2");
    assert_eq!(reconstructed.tenant_id(), "tenant-2");
    assert_eq!(reconstructed.task_id(), "task-2");
    assert_eq!(reconstructed.transition(), accepted);
    assert_ne!(reconstructed, issued);
    let before = task;
    assert_eq!(
        task.validate_replay(
            &reconstructed,
            "request-2",
            "tenant-2",
            "task-2",
            BapTaskEvent::Admit
        ),
        Ok(())
    );
    assert_eq!(
        task.apply_or_replay(
            Some(&reconstructed),
            "request-2",
            "tenant-2",
            "task-2",
            BapTaskEvent::Admit
        ),
        Ok(reconstructed.clone())
    );
    assert_eq!(task, before);
    for (key, tenant, id, event) in [
        ("request-1", "tenant-2", "task-2", BapTaskEvent::Admit),
        ("request-2", "tenant-1", "task-2", BapTaskEvent::Admit),
        ("request-2", "tenant-2", "task-1", BapTaskEvent::Admit),
        ("request-2", "tenant-2", "task-2", BapTaskEvent::Start),
    ] {
        assert_eq!(
            task.validate_replay(&reconstructed, key, tenant, id, event),
            Err(BapCommandReceiptError::IdempotencyConflict)
        );
        assert_eq!(
            task.apply_or_replay(Some(&reconstructed), key, tenant, id, event),
            Err(BapCommandReceiptError::IdempotencyConflict)
        );
        assert_eq!(task, before);
    }
    let mut state_only = BapTaskLifecycle::restore(before.state(), before.transition_sequence())
        .expect("reachable state-only snapshot");
    let state_only_before = state_only;
    assert_eq!(
        state_only.apply_or_replay(
            Some(&reconstructed),
            "request-2",
            "tenant-2",
            "task-2",
            BapTaskEvent::Admit
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(state_only, state_only_before);
    task.apply(BapTaskEvent::Start).expect("advance lifecycle");
    let advanced = task;
    assert_eq!(
        task.validate_replay(
            &reconstructed,
            "request-2",
            "tenant-2",
            "task-2",
            BapTaskEvent::Admit
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(
        task.apply_or_replay(
            Some(&reconstructed),
            "request-2",
            "tenant-2",
            "task-2",
            BapTaskEvent::Admit
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(task, advanced);
}

/// Pair valid bounded reconstruction with malformed supplied metadata in every identifier slot.
///
/// Syntax/length checks are validation only; they do not authenticate the supplied namespace.
#[test]
fn reconstruction_validates_supplied_identifier_syntax_and_literal_byte_bounds() {
    let mut task = BapTaskLifecycle::new();
    let transition = task
        .apply(BapTaskEvent::Admit)
        .expect("accepted transition");
    let before = task;
    let maximum = "x".repeat(128);
    let oversized = "x".repeat(129);
    let receipt = BapCommandReceipt::restore(&maximum, &maximum, &maximum, transition)
        .expect("literal 128-byte identifiers");
    assert_eq!(receipt.idempotency_key(), maximum);
    assert_eq!(receipt.tenant_id(), maximum);
    assert_eq!(receipt.task_id(), maximum);
    assert_eq!(receipt.transition(), transition);
    assert_eq!(
        task.validate_replay(&receipt, &maximum, &maximum, &maximum, BapTaskEvent::Admit),
        Ok(())
    );
    for invalid in [
        "",
        "with space",
        "slash/path",
        "한글",
        "control\n",
        oversized.as_str(),
    ] {
        let key_error = if invalid.len() > 128 {
            BapCommandReceiptError::IdempotencyKeyLimitExceeded
        } else {
            BapCommandReceiptError::InvalidIdempotencyKey
        };
        let tenant_error = if invalid.len() > 128 {
            BapCommandReceiptError::TenantIdLimitExceeded
        } else {
            BapCommandReceiptError::InvalidTenantId
        };
        let task_error = if invalid.len() > 128 {
            BapCommandReceiptError::TaskIdLimitExceeded
        } else {
            BapCommandReceiptError::InvalidTaskId
        };
        for (key, tenant, id, error) in [
            (invalid, "tenant-1", "task-1", key_error),
            ("request-1", invalid, "task-1", tenant_error),
            ("request-1", "tenant-1", invalid, task_error),
        ] {
            assert_eq!(
                BapCommandReceipt::restore(key, tenant, id, transition),
                Err(error)
            );
            assert_eq!(
                task.validate_replay(&receipt, key, tenant, id, BapTaskEvent::Admit),
                Err(error)
            );
            assert_eq!(
                task.apply_or_replay(Some(&receipt), key, tenant, id, BapTaskEvent::Admit),
                Err(error)
            );
            assert_eq!(task, before);
        }
    }
}

/// Show that equal final transition tuples do not authenticate instances or earlier history.
///
/// Different waits converge before Checkpoint at sequence five. A differing final transition
/// at the same state/sequence is a negative control; no external execution occurs in this test.
#[test]
fn replay_compares_last_transition_not_instance_or_complete_history() {
    let mut source = BapTaskLifecycle::new();
    for event in [
        BapTaskEvent::Admit,
        BapTaskEvent::Start,
        BapTaskEvent::WaitForApproval,
        BapTaskEvent::Resume,
    ] {
        source.apply(event).expect("source history");
    }
    let receipt = source
        .apply_with_receipt("request-5", "tenant-1", "task-1", BapTaskEvent::Checkpoint)
        .expect("source checkpoint");
    for wait in [
        BapTaskEvent::WaitForApproval,
        BapTaskEvent::WaitForExternalInput,
    ] {
        let mut other = BapTaskLifecycle::new();
        for event in [
            BapTaskEvent::Admit,
            BapTaskEvent::Start,
            wait,
            BapTaskEvent::Resume,
            BapTaskEvent::Checkpoint,
        ] {
            other.apply(event).expect("independent convergent history");
        }
        assert_eq!(other, source);
        let before = other;
        assert_eq!(
            other.validate_replay(
                &receipt,
                "request-5",
                "tenant-1",
                "task-1",
                BapTaskEvent::Checkpoint
            ),
            Ok(())
        );
        assert_eq!(
            other.apply_or_replay(
                Some(&receipt),
                "request-5",
                "tenant-1",
                "task-1",
                BapTaskEvent::Checkpoint
            ),
            Ok(receipt.clone())
        );
        assert_eq!(other, before);
    }
    let mut differing = BapTaskLifecycle::new();
    for event in [
        BapTaskEvent::Admit,
        BapTaskEvent::Start,
        BapTaskEvent::WaitForApproval,
        BapTaskEvent::Resume,
        BapTaskEvent::WaitForExternalInput,
    ] {
        differing.apply(event).expect("different last transition");
    }
    let distinct = BapCommandReceipt::restore(
        "request-5",
        "tenant-1",
        "task-1",
        differing.apply(BapTaskEvent::Resume).expect("resume"),
    )
    .expect("valid distinct receipt");
    // Use equal state/sequence with a different final event/previous-state tuple.
    let mut running = BapTaskLifecycle::new();
    for event in [
        BapTaskEvent::Admit,
        BapTaskEvent::Start,
        BapTaskEvent::WaitForApproval,
        BapTaskEvent::Resume,
        BapTaskEvent::RequireReconciliation,
        BapTaskEvent::ResolveReconciliation,
    ] {
        running.apply(event).expect("other running snapshot");
    }
    assert_eq!(running.state(), distinct.transition().current_state());
    assert_eq!(
        running.transition_sequence(),
        distinct.transition().sequence()
    );
    let before = running;
    assert_eq!(
        running.validate_replay(
            &distinct,
            "request-5",
            "tenant-1",
            "task-1",
            BapTaskEvent::Resume
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(
        running.apply_or_replay(
            Some(&distinct),
            "request-5",
            "tenant-1",
            "task-1",
            BapTaskEvent::Resume
        ),
        Err(BapCommandReceiptError::ReplayStateMismatch)
    );
    assert_eq!(running, before);
}
