#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, DisclosureDecision, DisclosureScope, HandleUseDecision, HandleUseRequest,
    SensitiveDataRequest, SensitiveValueHandleScope, evaluate_disclosure, evaluate_handle_use,
};

const TENANT: &str = "tenant_alpha";
const TASK: &str = "task_ship_order";
const FIELD: &str = "shipping_address";
const PURPOSE: &str = "fulfill_order";
const DESTINATION: &str = "https://shipping.example";

fn origin(input: &str) -> Origin {
    Origin::parse(input).expect("test origin must be valid")
}

fn disclosure_request(
    tenant: &str,
    task: &str,
    field: &str,
    purpose: &str,
    destination: &str,
    classification: DataClassification,
) -> SensitiveDataRequest {
    SensitiveDataRequest::new(
        tenant,
        task,
        field,
        purpose,
        origin(destination),
        classification,
    )
}

fn disclosure_scope(decision: DisclosureDecision) -> DisclosureScope {
    DisclosureScope::new(
        TENANT,
        TASK,
        FIELD,
        PURPOSE,
        origin(DESTINATION),
        DataClassification::PersonalData,
        decision,
    )
}

fn handle_scope(
    tenant: &str,
    classification: DataClassification,
) -> SensitiveValueHandleScope {
    SensitiveValueHandleScope::new(
        tenant,
        TASK,
        FIELD,
        PURPOSE,
        origin(DESTINATION),
        classification,
        2_000,
        2,
    )
}

#[allow(clippy::too_many_arguments)]
fn handle_use(
    tenant: &str,
    task: &str,
    field: &str,
    purpose: &str,
    destination: &str,
    classification: DataClassification,
    now: u64,
    uses: u32,
) -> HandleUseRequest {
    HandleUseRequest::new(
        tenant,
        task,
        field,
        purpose,
        origin(destination),
        classification,
        now,
        uses,
    )
}

fn valid_handle_use() -> HandleUseRequest {
    handle_use(
        TENANT,
        TASK,
        FIELD,
        PURPOSE,
        DESTINATION,
        DataClassification::PersonalData,
        1_999,
        0,
    )
}

#[test]
fn disclosure_is_bound_to_every_exact_authority_dimension() {
    let permitted = disclosure_scope(DisclosureDecision::FullFieldDisclosure);
    let request = disclosure_request(
        TENANT,
        TASK,
        FIELD,
        PURPOSE,
        DESTINATION,
        DataClassification::PersonalData,
    );
    assert_eq!(
        evaluate_disclosure(&request, &permitted),
        DisclosureDecision::FullFieldDisclosure
    );

    let mismatches = [
        ("tenant_beta", TASK, FIELD, PURPOSE, DESTINATION, DataClassification::PersonalData),
        (TENANT, "task_other", FIELD, PURPOSE, DESTINATION, DataClassification::PersonalData),
        (TENANT, TASK, "customer_email", PURPOSE, DESTINATION, DataClassification::PersonalData),
        (TENANT, TASK, FIELD, "marketing", DESTINATION, DataClassification::PersonalData),
        (TENANT, TASK, FIELD, PURPOSE, "https://other.example", DataClassification::PersonalData),
        (
            TENANT,
            TASK,
            FIELD,
            PURPOSE,
            DESTINATION,
            DataClassification::SensitivePersonalData,
        ),
    ];
    for (tenant, task, field, purpose, destination, classification) in mismatches {
        let request = disclosure_request(
            tenant,
            task,
            field,
            purpose,
            destination,
            classification,
        );
        assert_eq!(
            evaluate_disclosure(&request, &permitted),
            DisclosureDecision::DenyAccess
        );
    }
}

#[test]
fn sensitive_destination_uses_the_canonical_origin_boundary() {
    let canonical = disclosure_request(
        TENANT,
        TASK,
        FIELD,
        PURPOSE,
        "HTTPS://Shipping.Example:443",
        DataClassification::PersonalData,
    );
    assert_eq!(
        evaluate_disclosure(
            &canonical,
            &disclosure_scope(DisclosureDecision::FullFieldDisclosure),
        ),
        DisclosureDecision::FullFieldDisclosure
    );

    let non_default_port = disclosure_request(
        TENANT,
        TASK,
        FIELD,
        PURPOSE,
        "https://shipping.example:8443",
        DataClassification::PersonalData,
    );
    assert_eq!(
        evaluate_disclosure(
            &non_default_port,
            &disclosure_scope(DisclosureDecision::FullFieldDisclosure),
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
        assert!(Origin::parse(invalid).is_err(), "unexpected origin: {invalid}");
    }
    assert!(Origin::parse("http://127.0.0.1").is_ok());
}

#[test]
fn every_supported_disclosure_outcome_is_preserved_by_exact_scope() {
    let request = disclosure_request(
        TENANT,
        TASK,
        FIELD,
        PURPOSE,
        DESTINATION,
        DataClassification::PersonalData,
    );
    for decision in [
        DisclosureDecision::DenyAccess,
        DisclosureDecision::OpaqueHandleOnly,
        DisclosureDecision::DerivedValueOnly,
        DisclosureDecision::PartialFieldDisclosure,
        DisclosureDecision::FullFieldDisclosure,
        DisclosureDecision::HumanApprovalRequired,
        DisclosureDecision::DualControlRequired,
    ] {
        assert_eq!(evaluate_disclosure(&request, &disclosure_scope(decision)), decision);
    }
}

#[test]
fn opaque_handle_use_is_bound_to_scope_classification_expiry_and_use_count() {
    let scope = handle_scope(TENANT, DataClassification::PersonalData);
    assert_eq!(
        evaluate_handle_use(
            &handle_use(
                TENANT,
                TASK,
                FIELD,
                PURPOSE,
                DESTINATION,
                DataClassification::PersonalData,
                1_999,
                1,
            ),
            &scope,
        ),
        HandleUseDecision::Authorized
    );
    assert_eq!(
        evaluate_handle_use(
            &handle_use(
                TENANT,
                TASK,
                FIELD,
                PURPOSE,
                "https://other.example",
                DataClassification::PersonalData,
                1_999,
                1,
            ),
            &scope,
        ),
        HandleUseDecision::ScopeMismatch
    );
    assert_eq!(
        evaluate_handle_use(
            &handle_use(
                TENANT,
                TASK,
                FIELD,
                PURPOSE,
                DESTINATION,
                DataClassification::SensitivePersonalData,
                1_999,
                1,
            ),
            &scope,
        ),
        HandleUseDecision::ScopeMismatch
    );
    assert_eq!(
        evaluate_handle_use(
            &handle_use(
                TENANT,
                TASK,
                FIELD,
                PURPOSE,
                DESTINATION,
                DataClassification::PersonalData,
                2_000,
                1,
            ),
            &scope,
        ),
        HandleUseDecision::Expired
    );
    assert_eq!(
        evaluate_handle_use(
            &handle_use(
                TENANT,
                TASK,
                FIELD,
                PURPOSE,
                DESTINATION,
                DataClassification::PersonalData,
                1_999,
                2,
            ),
            &scope,
        ),
        HandleUseDecision::UseLimitReached
    );
}

#[test]
fn handle_scope_mismatch_covers_every_authority_dimension() {
    let scope = handle_scope(TENANT, DataClassification::PersonalData);
    let mismatches = [
        ("tenant_beta", TASK, FIELD, PURPOSE, DESTINATION, DataClassification::PersonalData),
        (TENANT, "task_other", FIELD, PURPOSE, DESTINATION, DataClassification::PersonalData),
        (TENANT, TASK, "customer_email", PURPOSE, DESTINATION, DataClassification::PersonalData),
        (TENANT, TASK, FIELD, "marketing", DESTINATION, DataClassification::PersonalData),
        (TENANT, TASK, FIELD, PURPOSE, "https://other.example", DataClassification::PersonalData),
        (TENANT, TASK, FIELD, PURPOSE, DESTINATION, DataClassification::CredentialData),
    ];
    for (tenant, task, field, purpose, destination, classification) in mismatches {
        let request = handle_use(
            tenant,
            task,
            field,
            purpose,
            destination,
            classification,
            1_999,
            0,
        );
        assert_eq!(
            evaluate_handle_use(&request, &scope),
            HandleUseDecision::ScopeMismatch
        );
    }
}

#[test]
fn incomplete_authority_never_grants_disclosure_or_handle_use() {
    let permitted = disclosure_scope(DisclosureDecision::FullFieldDisclosure);
    for (tenant, task, field, purpose) in [
        ("", TASK, FIELD, PURPOSE),
        (TENANT, "", FIELD, PURPOSE),
        (TENANT, TASK, "", PURPOSE),
        (TENANT, TASK, FIELD, ""),
    ] {
        let request = disclosure_request(
            tenant,
            task,
            field,
            purpose,
            DESTINATION,
            DataClassification::PersonalData,
        );
        assert_eq!(
            evaluate_disclosure(&request, &permitted),
            DisclosureDecision::DenyAccess
        );
    }

    let incomplete_disclosure_scope = DisclosureScope::new(
        "",
        TASK,
        FIELD,
        PURPOSE,
        origin(DESTINATION),
        DataClassification::PersonalData,
        DisclosureDecision::FullFieldDisclosure,
    );
    let request = disclosure_request(
        TENANT,
        TASK,
        FIELD,
        PURPOSE,
        DESTINATION,
        DataClassification::PersonalData,
    );
    assert_eq!(
        evaluate_disclosure(&request, &incomplete_disclosure_scope),
        DisclosureDecision::DenyAccess
    );

    assert_eq!(
        evaluate_handle_use(
            &valid_handle_use(),
            &handle_scope("", DataClassification::PersonalData),
        ),
        HandleUseDecision::ScopeMismatch
    );
    assert_eq!(
        evaluate_handle_use(
            &handle_use(
                TENANT,
                "",
                FIELD,
                PURPOSE,
                DESTINATION,
                DataClassification::PersonalData,
                1_999,
                0,
            ),
            &handle_scope(TENANT, DataClassification::PersonalData),
        ),
        HandleUseDecision::ScopeMismatch
    );
}

#[test]
fn authority_identifiers_are_bounded_ascii_policy_tokens() {
    let exact_maximum = "a".repeat(128);
    let valid_request = disclosure_request(
        &exact_maximum,
        "task.ship-order:v1",
        FIELD,
        "fulfill-order",
        DESTINATION,
        DataClassification::PersonalData,
    );
    let valid_scope = DisclosureScope::new(
        &exact_maximum,
        "task.ship-order:v1",
        FIELD,
        "fulfill-order",
        origin(DESTINATION),
        DataClassification::PersonalData,
        DisclosureDecision::FullFieldDisclosure,
    );
    assert_eq!(
        evaluate_disclosure(&valid_request, &valid_scope),
        DisclosureDecision::FullFieldDisclosure
    );

    let oversized = "a".repeat(129);
    for (tenant, task, field, purpose) in [
        ("tenant alpha", TASK, FIELD, PURPOSE),
        (TENANT, "task\nship_order", FIELD, PURPOSE),
        (TENANT, TASK, "배송주소", PURPOSE),
        (TENANT, TASK, FIELD, oversized.as_str()),
    ] {
        let request = disclosure_request(
            tenant,
            task,
            field,
            purpose,
            DESTINATION,
            DataClassification::PersonalData,
        );
        let scope = DisclosureScope::new(
            tenant,
            task,
            field,
            purpose,
            origin(DESTINATION),
            DataClassification::PersonalData,
            DisclosureDecision::FullFieldDisclosure,
        );
        assert_eq!(
            evaluate_disclosure(&request, &scope),
            DisclosureDecision::DenyAccess
        );
    }

    let invalid_scope = handle_scope("tenant alpha", DataClassification::PersonalData);
    let invalid_use = handle_use(
        "tenant alpha",
        TASK,
        FIELD,
        PURPOSE,
        DESTINATION,
        DataClassification::PersonalData,
        1_999,
        0,
    );
    assert_eq!(
        evaluate_handle_use(&invalid_use, &invalid_scope),
        HandleUseDecision::ScopeMismatch
    );
}
