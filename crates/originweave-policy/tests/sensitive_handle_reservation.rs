#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, HandleUseDecision, SensitiveDataAuthority, SensitiveHandleUseState,
    SensitiveValueHandleScope,
};

const TENANT: &str = "tenant_alpha";
const TASK: &str = "task_ship_order";
const FIELD: &str = "shipping_address";
const PURPOSE: &str = "fulfill_order";
const DESTINATION: &str = "https://shipping.example";
const AUDIENCE: &str = "browser_broker";

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
fn denied_reservations_do_not_consume_the_authoritative_count() {
    let mut state = SensitiveHandleUseState::new(scope(2));

    assert_eq!(
        state.reserve_use(authority("https://other.example"), AUDIENCE, 1_999),
        HandleUseDecision::ScopeMismatch
    );
    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 2_000),
        HandleUseDecision::Expired
    );
    assert_eq!(state.reserved_uses(), 0);
}

#[test]
fn wrong_or_malformed_audience_never_reserves_a_handle_use() {
    let mut state = SensitiveHandleUseState::new(scope(1));

    assert_eq!(
        state.reserve_use(authority(DESTINATION), "other_broker", 1_999),
        HandleUseDecision::AudienceMismatch
    );
    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), "---", 1_999),
        HandleUseDecision::AudienceMismatch
    );
    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::Authorized
    );
    assert_eq!(state.reserved_uses(), 1);
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
fn trusted_time_rollback_cannot_restore_expired_handle_authority() {
    let mut state = SensitiveHandleUseState::new(scope(1));

    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 2_000),
        HandleUseDecision::Expired
    );
    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(
        state.reserve_use(authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::TrustedTimeRollback
    );
    assert_eq!(state.reserved_uses(), 0);
}
