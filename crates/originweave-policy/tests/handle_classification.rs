#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, DisclosureDecision, DisclosureScope, HandleUseDecision, HandleUseRequest,
    SensitiveDataAuthority, SensitiveDataRequest, SensitiveValueHandleScope, evaluate_disclosure,
    evaluate_handle_use,
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
    let permitted = HandleUseRequest::new(authority(DataClassification::PersonalData), 1_999, 0);
    let reclassified = HandleUseRequest::new(
        authority(DataClassification::SensitivePersonalData),
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

#[test]
fn every_distinct_classification_pair_fails_closed() {
    let classifications = [
        DataClassification::PublicData,
        DataClassification::InternalData,
        DataClassification::PersonalData,
        DataClassification::SensitivePersonalData,
        DataClassification::CredentialData,
        DataClassification::PaymentData,
    ];

    for scope_classification in classifications {
        let scope = SensitiveValueHandleScope::new(authority(scope_classification), 2_000, 2);
        for request_classification in classifications {
            let request = HandleUseRequest::new(authority(request_classification), 1_999, 0);
            let expected = if request_classification == scope_classification {
                HandleUseDecision::Authorized
            } else {
                HandleUseDecision::ScopeMismatch
            };
            assert_eq!(
                evaluate_handle_use(&request, &scope),
                expected,
                "{scope_classification:?} scope vs {request_classification:?} request",
            );
        }
    }
}

#[test]
fn every_distinct_classification_pair_denies_disclosure() {
    let classifications = [
        DataClassification::PublicData,
        DataClassification::InternalData,
        DataClassification::PersonalData,
        DataClassification::SensitivePersonalData,
        DataClassification::CredentialData,
        DataClassification::PaymentData,
    ];

    for scope_classification in classifications {
        let scope = DisclosureScope::new(
            authority(scope_classification),
            DisclosureDecision::FullFieldDisclosure,
        );
        for request_classification in classifications {
            let request = SensitiveDataRequest::new(authority(request_classification));
            let expected = if request_classification == scope_classification {
                DisclosureDecision::FullFieldDisclosure
            } else {
                DisclosureDecision::DenyAccess
            };
            assert_eq!(
                evaluate_disclosure(&request, &scope),
                expected,
                "{scope_classification:?} scope vs {request_classification:?} request",
            );
        }
    }
}
