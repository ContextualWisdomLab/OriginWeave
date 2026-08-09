#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, DisclosureDecision, DisclosureScope, HandleUseDecision, HandleUseRequest,
    SensitiveDataRequest, SensitiveValueHandleScope, evaluate_disclosure, evaluate_handle_use,
};

fn origin(input: &str) -> Origin {
    Origin::parse(input).expect("test origin must be valid")
}

fn shipping_request() -> SensitiveDataRequest {
    SensitiveDataRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        DataClassification::PersonalData,
    )
}

fn shipping_scope(decision: DisclosureDecision) -> DisclosureScope {
    DisclosureScope::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        DataClassification::PersonalData,
        decision,
    )
}

#[test]
fn disclosure_is_bound_to_exact_tenant_task_field_purpose_destination_and_classification() {
    let permitted = shipping_scope(DisclosureDecision::FullFieldDisclosure);
    assert_eq!(
        evaluate_disclosure(&shipping_request(), &permitted),
        DisclosureDecision::FullFieldDisclosure
    );

    let mismatches = [
        SensitiveDataRequest::new(
            "tenant_beta",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_other",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "customer_email",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "marketing",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            origin("https://other.example"),
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::SensitivePersonalData,
        ),
    ];

    for request in mismatches {
        assert_eq!(
            evaluate_disclosure(&request, &permitted),
            DisclosureDecision::DenyAccess
        );
    }
}

#[test]
fn sensitive_destination_uses_the_canonical_origin_boundary() {
    let canonical_request = SensitiveDataRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("HTTPS://Shipping.Example:443"),
        DataClassification::PersonalData,
    );
    assert_eq!(
        evaluate_disclosure(
            &canonical_request,
            &shipping_scope(DisclosureDecision::FullFieldDisclosure),
        ),
        DisclosureDecision::FullFieldDisclosure
    );

    let non_default_port = SensitiveDataRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example:8443"),
        DataClassification::PersonalData,
    );
    assert_eq!(
        evaluate_disclosure(
            &non_default_port,
            &shipping_scope(DisclosureDecision::FullFieldDisclosure),
        ),
        DisclosureDecision::DenyAccess
    );

    for invalid in [
        "https://user@shipping.example",
        "https://shipping.example/path",
        "https://shipping.example\n",
        "https://배송.example",
        "https://127.1",
        "http://shipping.example",
    ] {
        assert!(
            Origin::parse(invalid).is_err(),
            "unexpected origin: {invalid}"
        );
    }

    assert!(Origin::parse("http://127.0.0.1").is_ok());
}

#[test]
fn every_supported_disclosure_outcome_is_preserved_by_exact_scope() {
    for decision in [
        DisclosureDecision::DenyAccess,
        DisclosureDecision::OpaqueHandleOnly,
        DisclosureDecision::DerivedValueOnly,
        DisclosureDecision::PartialFieldDisclosure,
        DisclosureDecision::FullFieldDisclosure,
        DisclosureDecision::HumanApprovalRequired,
        DisclosureDecision::DualControlRequired,
    ] {
        assert_eq!(
            evaluate_disclosure(&shipping_request(), &shipping_scope(decision)),
            decision
        );
    }
}

fn handle_scope() -> SensitiveValueHandleScope {
    SensitiveValueHandleScope::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        2_000,
        2,
    )
}

fn valid_handle_use() -> HandleUseRequest {
    HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        1_999,
        0,
    )
}

#[test]
fn opaque_handle_use_is_bound_to_scope_expiry_and_use_count() {
    let scope = handle_scope();
    let authorized = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        1_999,
        1,
    );
    assert_eq!(
        evaluate_handle_use(&authorized, &scope),
        HandleUseDecision::Authorized
    );

    let wrong_destination = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://other.example"),
        1_999,
        1,
    );
    assert_eq!(
        evaluate_handle_use(&wrong_destination, &scope),
        HandleUseDecision::ScopeMismatch
    );

    let expired = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        2_000,
        1,
    );
    assert_eq!(
        evaluate_handle_use(&expired, &scope),
        HandleUseDecision::Expired
    );

    let exhausted = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        1_999,
        2,
    );
    assert_eq!(
        evaluate_handle_use(&exhausted, &scope),
        HandleUseDecision::UseLimitReached
    );
}

#[test]
fn handle_scope_mismatch_covers_every_authority_dimension() {
    let scope = handle_scope();
    let mismatches = [
        HandleUseRequest::new(
            "tenant_beta",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            1_999,
            0,
        ),
        HandleUseRequest::new(
            "tenant_alpha",
            "task_other",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            1_999,
            0,
        ),
        HandleUseRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "customer_email",
            "fulfill_order",
            origin("https://shipping.example"),
            1_999,
            0,
        ),
        HandleUseRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "marketing",
            origin("https://shipping.example"),
            1_999,
            0,
        ),
        HandleUseRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            origin("https://other.example"),
            1_999,
            0,
        ),
    ];

    for request in mismatches {
        assert_eq!(
            evaluate_handle_use(&request, &scope),
            HandleUseDecision::ScopeMismatch
        );
    }
}

#[test]
fn incomplete_authority_never_grants_disclosure_or_handle_use() {
    for incomplete_request in [
        SensitiveDataRequest::new(
            "",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
        ),
    ] {
        assert_eq!(
            evaluate_disclosure(
                &incomplete_request,
                &shipping_scope(DisclosureDecision::FullFieldDisclosure),
            ),
            DisclosureDecision::DenyAccess
        );
    }

    for incomplete_scope in [
        DisclosureScope::new(
            "",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
            DisclosureDecision::FullFieldDisclosure,
        ),
        DisclosureScope::new(
            "tenant_alpha",
            "",
            "shipping_address",
            "fulfill_order",
            origin("https://shipping.example"),
            DataClassification::PersonalData,
            DisclosureDecision::FullFieldDisclosure,
        ),
    ] {
        assert_eq!(
            evaluate_disclosure(&shipping_request(), &incomplete_scope),
            DisclosureDecision::DenyAccess
        );
    }

    let incomplete_handle_scope = SensitiveValueHandleScope::new(
        "",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        2_000,
        2,
    );
    assert_eq!(
        evaluate_handle_use(&valid_handle_use(), &incomplete_handle_scope),
        HandleUseDecision::ScopeMismatch
    );

    let incomplete_handle_use = HandleUseRequest::new(
        "tenant_alpha",
        "",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        1_999,
        0,
    );
    assert_eq!(
        evaluate_handle_use(&incomplete_handle_use, &handle_scope()),
        HandleUseDecision::ScopeMismatch
    );
}

#[test]
fn authority_identifiers_are_bounded_ascii_policy_tokens() {
    let exact_maximum = "a".repeat(128);
    let valid_request = SensitiveDataRequest::new(
        &exact_maximum,
        "task.ship-order:v1",
        "shipping_address",
        "fulfill-order",
        origin("https://shipping.example"),
        DataClassification::PersonalData,
    );
    let valid_scope = DisclosureScope::new(
        &exact_maximum,
        "task.ship-order:v1",
        "shipping_address",
        "fulfill-order",
        origin("https://shipping.example"),
        DataClassification::PersonalData,
        DisclosureDecision::FullFieldDisclosure,
    );
    assert_eq!(
        evaluate_disclosure(&valid_request, &valid_scope),
        DisclosureDecision::FullFieldDisclosure
    );

    let oversized = "a".repeat(129);
    let invalid_pairs = [
        (
            SensitiveDataRequest::new(
                "tenant alpha",
                "task_ship_order",
                "shipping_address",
                "fulfill_order",
                origin("https://shipping.example"),
                DataClassification::PersonalData,
            ),
            DisclosureScope::new(
                "tenant alpha",
                "task_ship_order",
                "shipping_address",
                "fulfill_order",
                origin("https://shipping.example"),
                DataClassification::PersonalData,
                DisclosureDecision::FullFieldDisclosure,
            ),
        ),
        (
            SensitiveDataRequest::new(
                "tenant_alpha",
                "task\nship_order",
                "shipping_address",
                "fulfill_order",
                origin("https://shipping.example"),
                DataClassification::PersonalData,
            ),
            DisclosureScope::new(
                "tenant_alpha",
                "task\nship_order",
                "shipping_address",
                "fulfill_order",
                origin("https://shipping.example"),
                DataClassification::PersonalData,
                DisclosureDecision::FullFieldDisclosure,
            ),
        ),
        (
            SensitiveDataRequest::new(
                "tenant_alpha",
                "task_ship_order",
                "배송주소",
                "fulfill_order",
                origin("https://shipping.example"),
                DataClassification::PersonalData,
            ),
            DisclosureScope::new(
                "tenant_alpha",
                "task_ship_order",
                "배송주소",
                "fulfill_order",
                origin("https://shipping.example"),
                DataClassification::PersonalData,
                DisclosureDecision::FullFieldDisclosure,
            ),
        ),
        (
            SensitiveDataRequest::new(
                "tenant_alpha",
                "task_ship_order",
                "shipping_address",
                &oversized,
                origin("https://shipping.example"),
                DataClassification::PersonalData,
            ),
            DisclosureScope::new(
                "tenant_alpha",
                "task_ship_order",
                "shipping_address",
                &oversized,
                origin("https://shipping.example"),
                DataClassification::PersonalData,
                DisclosureDecision::FullFieldDisclosure,
            ),
        ),
    ];

    for (request, scope) in invalid_pairs {
        assert_eq!(
            evaluate_disclosure(&request, &scope),
            DisclosureDecision::DenyAccess
        );
    }

    let invalid_handle_scope = SensitiveValueHandleScope::new(
        "tenant alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        2_000,
        2,
    );
    let invalid_handle_use = HandleUseRequest::new(
        "tenant alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        origin("https://shipping.example"),
        1_999,
        0,
    );
    assert_eq!(
        evaluate_handle_use(&invalid_handle_use, &invalid_handle_scope),
        HandleUseDecision::ScopeMismatch
    );
}
