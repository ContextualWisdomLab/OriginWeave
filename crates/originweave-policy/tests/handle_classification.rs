#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, HandleUseDecision, HandleUseRequest, SensitiveDataAuthority,
    SensitiveValueHandleScope, evaluate_handle_use,
};

const AUDIENCE: &str = "trusted_browser_adapter";

fn destination() -> Origin {
    Origin::parse("https://shipping.example").expect("canonical destination")
}

fn authority(classification: DataClassification) -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        destination(),
        classification,
    )
}

#[test]
fn opaque_handle_use_requires_the_exact_data_classification() {
    let scope = SensitiveValueHandleScope::new(
        authority(DataClassification::PersonalData),
        AUDIENCE,
        2_000,
        2,
    );
    let permitted = HandleUseRequest::new(
        authority(DataClassification::PersonalData),
        AUDIENCE,
        1_999,
        0,
    );
    let reclassified = HandleUseRequest::new(
        authority(DataClassification::SensitivePersonalData),
        AUDIENCE,
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
