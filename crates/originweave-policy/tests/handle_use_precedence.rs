#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, HandleUseDecision, HandleUseRequest, SensitiveDataAuthority,
    SensitiveValueHandleScope, evaluate_handle_use,
};

fn destination() -> Origin {
    Origin::parse("https://shipping.example").expect("canonical destination")
}

fn authority() -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        destination(),
        DataClassification::PersonalData,
    )
}

fn mismatched_authority() -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        "tenant_beta",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        destination(),
        DataClassification::PersonalData,
    )
}

#[test]
fn scope_mismatch_precedes_expiry_and_use_limit() {
    let scope = SensitiveValueHandleScope::new(authority(), 2_000, 2);
    let request = HandleUseRequest::new(mismatched_authority(), 3_000, 99);
    assert_eq!(
        evaluate_handle_use(&request, &scope),
        HandleUseDecision::ScopeMismatch
    );
}

#[test]
fn expiry_precedes_use_limit() {
    let scope = SensitiveValueHandleScope::new(authority(), 2_000, 2);
    let request = HandleUseRequest::new(authority(), 3_000, 99);
    assert_eq!(
        evaluate_handle_use(&request, &scope),
        HandleUseDecision::Expired
    );
}

#[test]
fn zero_max_uses_exhausts_the_handle_before_any_use() {
    let scope = SensitiveValueHandleScope::new(authority(), 2_000, 0);
    let request = HandleUseRequest::new(authority(), 0, 0);
    assert_eq!(
        evaluate_handle_use(&request, &scope),
        HandleUseDecision::UseLimitReached
    );
}
