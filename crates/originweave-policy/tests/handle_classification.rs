#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, HandleUseDecision, SensitiveDataAuthority, SensitiveHandleUseState,
    SensitiveValueHandleScope,
};

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
    let scope =
        SensitiveValueHandleScope::new(authority(DataClassification::PersonalData), 2_000, 2);
    let mut state = SensitiveHandleUseState::new(scope);

    assert_eq!(
        state.reserve_use(authority(DataClassification::PersonalData), 1_999),
        HandleUseDecision::Authorized
    );
    assert_eq!(
        state.reserve_use(authority(DataClassification::SensitivePersonalData), 1_999,),
        HandleUseDecision::ScopeMismatch
    );
}
