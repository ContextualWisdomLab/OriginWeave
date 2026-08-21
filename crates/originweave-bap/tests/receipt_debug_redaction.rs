#![allow(clippy::expect_used)]

use originweave_bap::{BapTaskEvent, BapTaskLifecycle};

#[test]
fn command_receipt_debug_does_not_disclose_retry_or_tenant_identifiers() {
    let mut task = BapTaskLifecycle::new();
    let receipt = task
        .apply_with_receipt(
            "retry-secret-marker",
            "private-tenant-marker",
            "task-1",
            BapTaskEvent::Admit,
        )
        .expect("receipt");

    let debug = format!("{receipt:?}");
    assert!(debug.contains("idempotency_key_byte_count"));
    assert!(!debug.contains("retry-secret-marker"));
    assert!(!debug.contains("private-tenant-marker"));
}
