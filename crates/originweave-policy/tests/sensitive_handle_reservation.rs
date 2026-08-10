#![allow(clippy::expect_used)]

use std::sync::{Arc, Barrier, Mutex};
use std::thread;

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, HandleRevocationReason, HandleUseDecision, SensitiveDataAuthority,
    SensitiveHandleUseState, SensitiveValueHandleScope,
};

const TENANT: &str = "tenant_alpha";
const TASK: &str = "task_ship_order";
const FIELD: &str = "shipping_address";
const PURPOSE: &str = "fulfill_order";
const DESTINATION: &str = "https://shipping.example";
const AUDIENCE: &str = "trusted_browser_adapter";

fn authority(destination: &str) -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        TENANT,
        TASK,
        FIELD,
        PURPOSE,
        Origin::parse(destination).expect("test origin must be valid"),
        DataClassification::PersonalData,
    )
}

fn scope(max_uses: u32) -> SensitiveValueHandleScope {
    SensitiveValueHandleScope::new(authority(DESTINATION), AUDIENCE, 2_000, max_uses)
}

#[test]
fn reservation_state_consumes_each_authorized_use_exactly_once() {
    let mut state = SensitiveHandleUseState::new(scope(2));

    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::Authorized
    );
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::Authorized
    );
    assert_eq!(state.reserved_uses(), 2);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::UseLimitReached
    );
    assert_eq!(state.reserved_uses(), 2);
}

#[test]
fn concurrent_reservations_share_one_count_and_never_transfer_audience_authority() {
    let state = Arc::new(Mutex::new(SensitiveHandleUseState::new(scope(1))));
    let start = Arc::new(Barrier::new(4));
    let mut workers = Vec::new();

    for audience in [AUDIENCE, AUDIENCE, "other_service"] {
        let state = Arc::clone(&state);
        let start = Arc::clone(&start);
        workers.push(thread::spawn(move || {
            start.wait();
            state
                .lock()
                .expect("test mutex must remain healthy")
                .reserve_use(authority(DESTINATION), audience, 1_999)
        }));
    }

    start.wait();
    let decisions: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().expect("reservation worker must complete"))
        .collect();

    assert_eq!(
        decisions
            .iter()
            .filter(|decision| **decision == HandleUseDecision::Authorized)
            .count(),
        1
    );
    assert_eq!(
        decisions
            .iter()
            .filter(|decision| **decision == HandleUseDecision::UseLimitReached)
            .count(),
        1
    );
    assert_eq!(
        decisions
            .iter()
            .filter(|decision| **decision == HandleUseDecision::AudienceMismatch)
            .count(),
        1
    );
    assert_eq!(
        state
            .lock()
            .expect("test mutex must remain healthy")
            .reserved_uses(),
        1
    );
}

#[test]
fn denied_reservations_do_not_consume_the_authoritative_count() {
    let mut state = SensitiveHandleUseState::new(scope(2));

    assert_eq!(
        state.reserve_use(authority("https://other.example"), AUDIENCE, 1_999),
        HandleUseDecision::ScopeMismatch
    );
    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), "other_service", 1_999),
        HandleUseDecision::AudienceMismatch
    );
    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 2_000),
        HandleUseDecision::Expired
    );
    assert_eq!(state.reserved_uses(), 0);
}

#[test]
fn invalid_or_empty_audience_never_receives_handle_authority() {
    let mut state = SensitiveHandleUseState::new(scope(1));

    for audience in ["", "browser adapter", "브라우저", "_-_"] {
        assert_eq!(
            state.reserve_use(authority(DESTINATION), audience, 1_999),
            HandleUseDecision::AudienceMismatch
        );
        assert_eq!(state.reserved_uses(), 0);
    }
}

#[test]
fn zero_use_scope_never_reserves_or_wraps_the_counter() {
    let mut state = SensitiveHandleUseState::new(scope(0));

    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::UseLimitReached
    );
    assert_eq!(state.reserved_uses(), 0);
}

#[test]
fn revocation_is_authoritative_idempotent_and_blocks_future_use() {
    let mut state = SensitiveHandleUseState::new(scope(3));

    assert_eq!(state.revocation_reason(), None);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::Authorized
    );
    assert_eq!(state.reserved_uses(), 1);

    assert!(state.revoke(HandleRevocationReason::TaskCompleted));
    assert_eq!(
        state.revocation_reason(),
        Some(HandleRevocationReason::TaskCompleted)
    );
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::Revoked
    );
    assert_eq!(state.reserved_uses(), 1);

    assert!(!state.revoke(HandleRevocationReason::PolicyChanged));
    assert_eq!(
        state.revocation_reason(),
        Some(HandleRevocationReason::TaskCompleted)
    );
}

#[test]
fn revocation_precedes_request_mismatch_without_leaking_later_policy_state() {
    let mut state = SensitiveHandleUseState::new(scope(3));
    assert!(state.revoke(HandleRevocationReason::SuspiciousUse));

    for (request_authority, audience, now) in [
        (authority("https://other.example"), AUDIENCE, 1_999),
        (authority(DESTINATION), "other_service", 1_999),
        (authority(DESTINATION), AUDIENCE, 2_000),
    ] {
        assert_eq!(
            state.reserve_use(request_authority, audience, now),
            HandleUseDecision::Revoked
        );
        assert_eq!(state.reserved_uses(), 0);
        assert_eq!(
            state.revocation_reason(),
            Some(HandleRevocationReason::SuspiciousUse)
        );
    }
}

#[test]
fn every_required_revocation_cause_can_be_recorded() {
    for reason in [
        HandleRevocationReason::TaskCompleted,
        HandleRevocationReason::PolicyChanged,
        HandleRevocationReason::KeyRotated,
        HandleRevocationReason::SessionTerminated,
        HandleRevocationReason::SuspiciousUse,
    ] {
        let mut state = SensitiveHandleUseState::new(scope(1));
        assert!(state.revoke(reason));
        assert_eq!(state.revocation_reason(), Some(reason));
        assert_eq!(
            state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
            HandleUseDecision::Revoked
        );
        assert_eq!(state.reserved_uses(), 0);
    }
}
