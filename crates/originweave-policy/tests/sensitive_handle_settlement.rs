#![allow(clippy::expect_used)]

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
fn compensating_exact_failed_reservation_restores_capacity_without_replay() {
    let mut state = SensitiveHandleUseState::new(scope(1));

    let first = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999)
        .expect("first reservation must be authorized");
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.outstanding_reservations(), 1);
    assert_eq!(state.completed_uses(), 0);
    assert_eq!(
        state.reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999),
        Err(HandleUseDecision::UseLimitReached)
    );

    assert!(state.compensate_reservation(&first));
    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(state.outstanding_reservations(), 0);
    assert_eq!(state.completed_uses(), 0);

    let replacement = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999)
        .expect("compensation must restore one use of capacity");
    assert_ne!(replacement, first);
    assert!(!state.compensate_reservation(&first));
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.outstanding_reservations(), 1);
}

#[test]
fn committed_reservation_remains_consumed_and_cannot_be_compensated() {
    let mut state = SensitiveHandleUseState::new(scope(1));
    let reservation = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999)
        .expect("reservation must be authorized");

    assert!(state.commit_reservation(&reservation));
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.outstanding_reservations(), 0);
    assert_eq!(state.completed_uses(), 1);
    assert!(!state.commit_reservation(&reservation));
    assert!(!state.compensate_reservation(&reservation));
    assert_eq!(
        state.reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999),
        Err(HandleUseDecision::UseLimitReached)
    );
}

#[test]
fn settlement_is_identity_bound_when_multiple_reservations_are_outstanding() {
    let mut state = SensitiveHandleUseState::new(scope(3));
    let first = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999)
        .expect("first reservation must be authorized");
    let second = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999)
        .expect("second reservation must be authorized");

    assert_ne!(first, second);
    assert!(state.compensate_reservation(&first));
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.outstanding_reservations(), 1);
    assert!(state.commit_reservation(&second));
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.outstanding_reservations(), 0);
    assert_eq!(state.completed_uses(), 1);
}

#[test]
fn denied_or_revoked_state_never_creates_tracked_reservation() {
    let mut state = SensitiveHandleUseState::new(scope(2));

    assert_eq!(
        state.reserve_tracked_use(authority("https://other.example"), AUDIENCE, 1_999),
        Err(HandleUseDecision::ScopeMismatch)
    );
    assert_eq!(state.outstanding_reservations(), 0);
    assert!(state.revoke(HandleRevocationReason::PolicyChanged));
    assert_eq!(
        state.reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999),
        Err(HandleUseDecision::Revoked)
    );
    assert_eq!(state.reserved_uses(), 0);
}

#[test]
fn revocation_does_not_prevent_compensating_an_undisclosed_reservation() {
    let mut state = SensitiveHandleUseState::new(scope(1));
    let reservation = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_999)
        .expect("reservation must be authorized before revocation");

    assert!(state.revoke(HandleRevocationReason::SessionTerminated));
    assert!(state.compensate_reservation(&reservation));
    assert_eq!(state.reserved_uses(), 0);
    assert_eq!(state.completed_uses(), 0);
    assert_eq!(state.outstanding_reservations(), 0);
}
