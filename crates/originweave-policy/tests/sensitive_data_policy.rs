#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, DisclosureDecision, DisclosureScope, HandleUseDecision, HandleUseRequest,
    SensitiveDataAuthority, SensitiveDataRequest, SensitiveValueHandleScope, evaluate_disclosure,
    evaluate_handle_use,
};

const TENANT: &str = "tenant_alpha";
const TASK: &str = "task_ship_order";
const FIELD: &str = "shipping_address";
const PURPOSE: &str = "fulfill_order";
const DESTINATION: &str = "https://shipping.example";

#[derive(Clone, Copy)]
struct AuthorityCase<'a> {
    tenant: &'a str,
    task: &'a str,
    field: &'a str,
    purpose: &'a str,
    destination: &'a str,
}

fn authority_case<'a>(
    tenant: &'a str,
    task: &'a str,
    field: &'a str,
    purpose: &'a str,
    destination: &'a str,
) -> AuthorityCase<'a> {
    AuthorityCase {
        tenant,
        task,
        field,
        purpose,
        destination,
    }
}

fn exact_authority() -> AuthorityCase<'static> {
    authority_case(TENANT, TASK, FIELD, PURPOSE, DESTINATION)
}

fn origin(input: &str) -> Origin {
    Origin::parse(input).expect("test origin must be valid")
}

fn sensitive_authority(
    authority: AuthorityCase<'_>,
    classification: DataClassification,
) -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        authority.tenant,
        authority.task,
        authority.field,
        authority.purpose,
        origin(authority.destination),
        classification,
    )
}

fn disclosure_request(
    authority: AuthorityCase<'_>,
    classification: DataClassification,
) -> SensitiveDataRequest {
    SensitiveDataRequest::new(sensitive_authority(authority, classification))
}

fn disclosure_scope(
    authority: AuthorityCase<'_>,
    classification: DataClassification,
    decision: DisclosureDecision,
) -> DisclosureScope {
    DisclosureScope::new(sensitive_authority(authority, classification), decision)
}

fn handle_scope(
    authority: AuthorityCase<'_>,
    classification: DataClassification,
) -> SensitiveValueHandleScope {
    SensitiveValueHandleScope::new(sensitive_authority(authority, classification), 2_000, 2)
}

fn handle_use(
    authority: AuthorityCase<'_>,
    classification: DataClassification,
    now: u64,
    uses: u32,
) -> HandleUseRequest {
    HandleUseRequest::new(sensitive_authority(authority, classification), now, uses)
}

fn assert_disclosure_denied(authority: AuthorityCase<'_>, classification: DataClassification) {
    let permitted = disclosure_scope(
        exact_authority(),
        DataClassification::PersonalData,
        DisclosureDecision::FullFieldDisclosure,
    );
    assert_eq!(
        evaluate_disclosure(&disclosure_request(authority, classification), &permitted),
        DisclosureDecision::DenyAccess
    );
}

fn assert_handle_scope_mismatch(authority: AuthorityCase<'_>, classification: DataClassification) {
    let scope = handle_scope(exact_authority(), DataClassification::PersonalData);
    assert_eq!(
        evaluate_handle_use(&handle_use(authority, classification, 1_999, 0), &scope),
        HandleUseDecision::ScopeMismatch
    );
}

#[test]
fn disclosure_is_bound_to_every_exact_authority_dimension() {
    let exact = exact_authority();
    let permitted = disclosure_scope(
        exact,
        DataClassification::PersonalData,
        DisclosureDecision::FullFieldDisclosure,
    );
    assert_eq!(
        evaluate_disclosure(
            &disclosure_request(exact, DataClassification::PersonalData),
            &permitted,
        ),
        DisclosureDecision::FullFieldDisclosure
    );

    assert_disclosure_denied(
        authority_case("tenant_beta", TASK, FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_disclosure_denied(
        authority_case(TENANT, "task_other", FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_disclosure_denied(
        authority_case(TENANT, TASK, "customer_email", PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_disclosure_denied(
        authority_case(TENANT, TASK, FIELD, "marketing", DESTINATION),
        DataClassification::PersonalData,
    );
    assert_disclosure_denied(
        authority_case(TENANT, TASK, FIELD, PURPOSE, "https://other.example"),
        DataClassification::PersonalData,
    );
    assert_disclosure_denied(exact, DataClassification::SensitivePersonalData);
}

#[test]
fn sensitive_destination_uses_the_canonical_origin_boundary() {
    let canonical = authority_case(TENANT, TASK, FIELD, PURPOSE, "HTTPS://Shipping.Example:443");
    assert_eq!(
        evaluate_disclosure(
            &disclosure_request(canonical, DataClassification::PersonalData),
            &disclosure_scope(
                exact_authority(),
                DataClassification::PersonalData,
                DisclosureDecision::FullFieldDisclosure,
            ),
        ),
        DisclosureDecision::FullFieldDisclosure
    );

    assert_disclosure_denied(
        authority_case(
            TENANT,
            TASK,
            FIELD,
            PURPOSE,
            "https://shipping.example:8443",
        ),
        DataClassification::PersonalData,
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
    let exact = exact_authority();
    let request = disclosure_request(exact, DataClassification::PersonalData);
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
            evaluate_disclosure(
                &request,
                &disclosure_scope(exact, DataClassification::PersonalData, decision),
            ),
            decision
        );
    }
}

#[test]
fn opaque_handle_use_is_bound_to_scope_classification_expiry_and_use_count() {
    let exact = exact_authority();
    let scope = handle_scope(exact, DataClassification::PersonalData);
    assert_eq!(
        evaluate_handle_use(
            &handle_use(exact, DataClassification::PersonalData, 1_999, 1),
            &scope,
        ),
        HandleUseDecision::Authorized
    );
    assert_handle_scope_mismatch(
        authority_case(TENANT, TASK, FIELD, PURPOSE, "https://other.example"),
        DataClassification::PersonalData,
    );
    assert_handle_scope_mismatch(exact, DataClassification::SensitivePersonalData);
    assert_eq!(
        evaluate_handle_use(
            &handle_use(exact, DataClassification::PersonalData, 2_000, 1),
            &scope,
        ),
        HandleUseDecision::Expired
    );
    assert_eq!(
        evaluate_handle_use(
            &handle_use(exact, DataClassification::PersonalData, 1_999, 2),
            &scope,
        ),
        HandleUseDecision::UseLimitReached
    );
}

#[test]
fn handle_scope_mismatch_covers_every_authority_dimension() {
    assert_handle_scope_mismatch(
        authority_case("tenant_beta", TASK, FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_handle_scope_mismatch(
        authority_case(TENANT, "task_other", FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_handle_scope_mismatch(
        authority_case(TENANT, TASK, "customer_email", PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_handle_scope_mismatch(
        authority_case(TENANT, TASK, FIELD, "marketing", DESTINATION),
        DataClassification::PersonalData,
    );
    assert_handle_scope_mismatch(
        authority_case(TENANT, TASK, FIELD, PURPOSE, "https://other.example"),
        DataClassification::PersonalData,
    );
    assert_handle_scope_mismatch(exact_authority(), DataClassification::CredentialData);
}

#[test]
fn incomplete_authority_never_grants_disclosure_or_handle_use() {
    assert_disclosure_denied(
        authority_case("", TASK, FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_disclosure_denied(
        authority_case(TENANT, "", FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_disclosure_denied(
        authority_case(TENANT, TASK, "", PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_disclosure_denied(
        authority_case(TENANT, TASK, FIELD, "", DESTINATION),
        DataClassification::PersonalData,
    );

    let exact = exact_authority();
    let request = disclosure_request(exact, DataClassification::PersonalData);
    let incomplete_scope = disclosure_scope(
        authority_case("", TASK, FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
        DisclosureDecision::FullFieldDisclosure,
    );
    assert_eq!(
        evaluate_disclosure(&request, &incomplete_scope),
        DisclosureDecision::DenyAccess
    );

    let incomplete_handle_scope = handle_scope(
        authority_case("", TASK, FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
    assert_eq!(
        evaluate_handle_use(
            &handle_use(exact, DataClassification::PersonalData, 1_999, 0),
            &incomplete_handle_scope,
        ),
        HandleUseDecision::ScopeMismatch
    );
    assert_handle_scope_mismatch(
        authority_case(TENANT, "", FIELD, PURPOSE, DESTINATION),
        DataClassification::PersonalData,
    );
}

#[test]
fn authority_identifiers_are_bounded_ascii_policy_tokens() {
    let exact_maximum = "a".repeat(128);
    let valid = authority_case(
        &exact_maximum,
        "task.ship-order:v1",
        FIELD,
        "fulfill-order",
        DESTINATION,
    );
    assert_eq!(
        evaluate_disclosure(
            &disclosure_request(valid, DataClassification::PersonalData),
            &disclosure_scope(
                valid,
                DataClassification::PersonalData,
                DisclosureDecision::FullFieldDisclosure,
            ),
        ),
        DisclosureDecision::FullFieldDisclosure
    );

    let oversized = "a".repeat(129);
    for punctuation_only in [":", "...", "_-_"] {
        assert_invalid_equal_authority(authority_case(
            punctuation_only,
            TASK,
            FIELD,
            PURPOSE,
            DESTINATION,
        ));
    }
    assert_invalid_equal_authority(authority_case(
        "tenant alpha",
        TASK,
        FIELD,
        PURPOSE,
        DESTINATION,
    ));
    assert_invalid_equal_authority(authority_case(
        TENANT,
        "task\nship_order",
        FIELD,
        PURPOSE,
        DESTINATION,
    ));
    assert_invalid_equal_authority(authority_case(
        TENANT,
        TASK,
        "배송주소",
        PURPOSE,
        DESTINATION,
    ));
    assert_invalid_equal_authority(authority_case(TENANT, TASK, FIELD, &oversized, DESTINATION));

    let invalid_handle_authority =
        authority_case("tenant alpha", TASK, FIELD, PURPOSE, DESTINATION);
    assert_eq!(
        evaluate_handle_use(
            &handle_use(
                invalid_handle_authority,
                DataClassification::PersonalData,
                1_999,
                0,
            ),
            &handle_scope(invalid_handle_authority, DataClassification::PersonalData),
        ),
        HandleUseDecision::ScopeMismatch
    );
}

fn assert_invalid_equal_authority(authority: AuthorityCase<'_>) {
    let request = disclosure_request(authority, DataClassification::PersonalData);
    let scope = disclosure_scope(
        authority,
        DataClassification::PersonalData,
        DisclosureDecision::FullFieldDisclosure,
    );
    assert_eq!(
        evaluate_disclosure(&request, &scope),
        DisclosureDecision::DenyAccess
    );
}
