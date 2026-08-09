#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, HandleUseDecision, HandleUseRequest, SensitiveValueHandleScope,
    evaluate_handle_use,
};

fn destination() -> Origin {
    Origin::parse("https://shipping.example").expect("canonical destination")
}

#[test]
fn opaque_handle_use_requires_the_exact_data_classification() {
    let scope = SensitiveValueHandleScope::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        destination(),
        DataClassification::PersonalData,
        2_000,
        2,
    );
    let permitted = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        destination(),
        DataClassification::PersonalData,
        1_999,
        0,
    );
    let reclassified = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        destination(),
        DataClassification::SensitivePersonalData,
        1_999,
        0,
    );

    assert_eq!(
        evaluate_handle_use(&permitted, &scope),
        HandleUseDecision::Authorized
    );
    assert_eq!(
        evaluate_handle_use(&reclassified, &scope),
        HandleUseDecision::ScopeMismatch
    );
}
