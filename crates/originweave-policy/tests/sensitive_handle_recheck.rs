#![allow(clippy::expect_used)]

use std::cell::Cell;

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

thread_local! {
    static DISPATCH_COUNT: Cell<u32> = const { Cell::new(0) };
}

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

fn authority_with_tenant(tenant_id: &str) -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        tenant_id,
        TASK,
        FIELD,
        PURPOSE,
        Origin::parse(DESTINATION).expect("test origin must be valid"),
        DataClassification::PersonalData,
    )
}

fn scope(max_uses: u32) -> SensitiveValueHandleScope {
    SensitiveValueHandleScope::new(authority(DESTINATION), AUDIENCE, 2_000, max_uses)
}

fn record_disclosure() -> &'static str {
    DISPATCH_COUNT.set(DISPATCH_COUNT.get() + 1);
    "disclosure-attempted"
}

fn reset_dispatch_count() {
    DISPATCH_COUNT.set(0);
}

fn dispatch_count() -> u32 {
    DISPATCH_COUNT.get()
}

#[test]
fn outstanding_reservation_can_be_rechecked_without_consuming_another_use() {
    let mut state = SensitiveHandleUseState::new(scope(1));
    let reservation = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_900)
        .expect("reservation must be authorized");

    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(
        state.recheck_reservation(&reservation, authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::Authorized
    );
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.outstanding_reservations(), 1);
}

#[test]
fn current_reservation_gates_one_same_call_disclosure_callback() {
    let mut state = SensitiveHandleUseState::new(scope(1));
    let reservation = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_900)
        .expect("reservation must be authorized");
    reset_dispatch_count();

    let result = state
        .dispatch_if_reservation_current(
            &reservation,
            authority(DESTINATION),
            AUDIENCE,
            1_999,
            record_disclosure as fn() -> &'static str,
        )
        .expect("current reservation must permit callback dispatch");

    assert_eq!(result, "disclosure-attempted");
    assert_eq!(dispatch_count(), 1);
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.completed_uses(), 0);
    assert_eq!(state.outstanding_reservations(), 1);
}

#[test]
fn denied_reservation_never_invokes_disclosure_callback() {
    let mut state = SensitiveHandleUseState::new(scope(1));
    let reservation = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_900)
        .expect("reservation must be authorized");
    reset_dispatch_count();

    let result = state.dispatch_if_reservation_current(
        &reservation,
        authority(DESTINATION),
        AUDIENCE,
        2_000,
        record_disclosure as fn() -> &'static str,
    );

    assert_eq!(result, Err(HandleUseDecision::Expired));
    assert_eq!(dispatch_count(), 0);
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.completed_uses(), 0);
    assert_eq!(state.outstanding_reservations(), 1);
}

#[test]
fn foreign_or_settled_reservation_cannot_be_rechecked() {
    let mut first_state = SensitiveHandleUseState::new(scope(2));
    let mut second_state = SensitiveHandleUseState::new(scope(2));
    let first = first_state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_900)
        .expect("first reservation must be authorized");
    let second = second_state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_900)
        .expect("second reservation must be authorized");

    assert_eq!(
        second_state.recheck_reservation(&first, authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::ReservationNotOutstanding
    );
    assert!(first_state.compensate_reservation(&first));
    assert_eq!(
        first_state.recheck_reservation(&first, authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::ReservationNotOutstanding
    );
    assert!(second_state.commit_reservation(&second));
    assert_eq!(
        second_state.recheck_reservation(&second, authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::ReservationNotOutstanding
    );
}

#[test]
fn recheck_revalidates_scope_audience_and_expiry_without_poisoning_time() {
    let mut state = SensitiveHandleUseState::new(scope(1));
    let reservation = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_900)
        .expect("reservation must be authorized");

    assert_eq!(
        state.recheck_reservation(
            &reservation,
            authority("https://other.example"),
            AUDIENCE,
            9_999,
        ),
        HandleUseDecision::ScopeMismatch
    );
    assert_eq!(
        state.recheck_reservation(&reservation, authority_with_tenant(""), AUDIENCE, 9_999),
        HandleUseDecision::ScopeMismatch
    );
    assert_eq!(
        state.recheck_reservation(
            &reservation,
            authority(DESTINATION),
            "other_browser_adapter",
            9_999,
        ),
        HandleUseDecision::AudienceMismatch
    );
    assert_eq!(
        state.recheck_reservation(&reservation, authority(DESTINATION), "", 9_999),
        HandleUseDecision::AudienceMismatch
    );
    assert_eq!(
        state.recheck_reservation(&reservation, authority(DESTINATION), AUDIENCE, 1_999),
        HandleUseDecision::Authorized
    );
    assert_eq!(
        state.recheck_reservation(&reservation, authority(DESTINATION), AUDIENCE, 2_000),
        HandleUseDecision::Expired
    );
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.outstanding_reservations(), 1);
}

#[test]
fn binding_mismatch_does_not_expose_revocation_state_on_recheck() {
    let mut active_state = SensitiveHandleUseState::new(scope(1));
    let foreign = active_state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_900)
        .expect("foreign reservation must be authorized");
    let mut revoked_state = SensitiveHandleUseState::new(scope(1));
    assert!(revoked_state.revoke(HandleRevocationReason::PolicyChanged));

    assert_eq!(
        revoked_state.recheck_reservation(
            &foreign,
            authority("https://other.example"),
            AUDIENCE,
            2_001,
        ),
        HandleUseDecision::ScopeMismatch
    );
    assert_eq!(
        revoked_state.recheck_reservation(
            &foreign,
            authority(DESTINATION),
            "other_browser_adapter",
            2_001,
        ),
        HandleUseDecision::AudienceMismatch
    );
    assert_eq!(
        revoked_state.recheck_reservation(
            &foreign,
            authority(DESTINATION),
            AUDIENCE,
            2_001,
        ),
        HandleUseDecision::Revoked
    );
}

#[test]
fn recheck_rejects_trusted_time_rollback_and_denied_dispatch_stays_closed() {
    let mut state = SensitiveHandleUseState::new(scope(1));
    let reservation = state
        .reserve_tracked_use(authority(DESTINATION), AUDIENCE, 1_900)
        .expect("reservation must be authorized");

    assert_eq!(
        state.recheck_reservation(&reservation, authority(DESTINATION), AUDIENCE, 1_950),
        HandleUseDecision::Authorized
    );
    assert_eq!(
        state.recheck_reservation(&reservation, authority(DESTINATION), AUDIENCE, 1_949),
        HandleUseDecision::TrustedTimeRollback
    );

    reset_dispatch_count();
    assert_eq!(
        state.dispatch_if_reservation_current(
            &reservation,
            authority(DESTINATION),
            AUDIENCE,
            1_949,
            record_disclosure as fn() -> &'static str,
        ),
        Err(HandleUseDecision::TrustedTimeRollback)
    );
    assert_eq!(dispatch_count(), 0);
    assert_eq!(state.reserved_uses(), 1);
    assert_eq!(state.completed_uses(), 0);
    assert_eq!(state.outstanding_reservations(), 1);
}
