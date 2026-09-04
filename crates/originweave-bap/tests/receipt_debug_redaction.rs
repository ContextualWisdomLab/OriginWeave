#![allow(clippy::expect_used)]

use originweave_bap::{BapTaskEvent, BapTaskLifecycle};

#[test]
fn command_receipt_debug_does_not_disclose_retry_tenant_or_task_identifiers() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_with_receipt(
            "retry-secret-marker",
            "private-tenant-marker",
            "private-task-marker",
            BapTaskEvent::Admit,
        )
        .expect("receipt");

    let debug = format!("{receipt:?}");
    assert!(debug.contains("idempotency_key_byte_count"));
    assert!(!debug.contains("retry-secret-marker"));
    assert!(!debug.contains("private-tenant-marker"));
    assert!(!debug.contains("private-task-marker"));
}
