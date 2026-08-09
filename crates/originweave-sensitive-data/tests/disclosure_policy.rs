use originweave_sensitive_data::{
    DataClassification, DisclosureDecision, DisclosureScope, HandleUseDecision, HandleUseRequest,
    SensitiveDataRequest, SensitiveValueHandleScope, authorize_handle_use, evaluate_disclosure,
};

fn shipping_request() -> SensitiveDataRequest {
    SensitiveDataRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        "https://shipping.example",
        DataClassification::PersonalData,
    )
}

fn shipping_scope(decision: DisclosureDecision) -> DisclosureScope {
    DisclosureScope::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        "https://shipping.example",
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
            "https://shipping.example",
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_other",
            "shipping_address",
            "fulfill_order",
            "https://shipping.example",
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "customer_email",
            "fulfill_order",
            "https://shipping.example",
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "marketing",
            "https://shipping.example",
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            "https://other.example",
            DataClassification::PersonalData,
        ),
        SensitiveDataRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "fulfill_order",
            "https://shipping.example",
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
        "https://shipping.example",
        2_000,
        2,
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
        "https://shipping.example",
        1_999,
        1,
    );
    assert_eq!(
        authorize_handle_use(&authorized, &scope),
        HandleUseDecision::Authorized
    );

    let wrong_audience = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        "https://other.example",
        1_999,
        1,
    );
    assert_eq!(
        authorize_handle_use(&wrong_audience, &scope),
        HandleUseDecision::ScopeMismatch
    );

    let expired = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        "https://shipping.example",
        2_000,
        1,
    );
    assert_eq!(
        authorize_handle_use(&expired, &scope),
        HandleUseDecision::Expired
    );

    let exhausted = HandleUseRequest::new(
        "tenant_alpha",
        "task_ship_order",
        "shipping_address",
        "fulfill_order",
        "https://shipping.example",
        1_999,
        2,
    );
    assert_eq!(
        authorize_handle_use(&exhausted, &scope),
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
            "https://shipping.example",
            1_999,
            0,
        ),
        HandleUseRequest::new(
            "tenant_alpha",
            "task_other",
            "shipping_address",
            "fulfill_order",
            "https://shipping.example",
            1_999,
            0,
        ),
        HandleUseRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "customer_email",
            "fulfill_order",
            "https://shipping.example",
            1_999,
            0,
        ),
        HandleUseRequest::new(
            "tenant_alpha",
            "task_ship_order",
            "shipping_address",
            "marketing",
            "https://shipping.example",
            1_999,
            0,
        ),
    ];

    for request in mismatches {
        assert_eq!(
            authorize_handle_use(&request, &scope),
            HandleUseDecision::ScopeMismatch
        );
    }
}
