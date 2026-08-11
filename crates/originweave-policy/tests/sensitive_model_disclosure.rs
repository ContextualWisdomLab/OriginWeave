#![allow(clippy::expect_used)]

//! Fail-closed composition contract for one full protected field entering a reviewed model request.
//!
//! The model route and invocation policy are not raw-value disclosure authority. This contract
//! requires the exact sensitive-data authority to authorize full-field disclosure, requires that
//! authority to be the same authority carried by the reviewed model invocation, requires no known
//! lower-disclosure task path, and then requires the invocation policy itself to authorize. It carries
//! no protected bytes and performs no model I/O.

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, DisclosureDecision, DisclosureScope, ModelDisclosureDecision,
    ModelDisclosureNecessity, ModelInvocationDecision, ModelInvocationRequest, ModelInvocationScope,
    ModelRouteDecision, ModelRouteRequest, ModelRouteScope, SensitiveDataAuthority,
    SensitiveDataRequest, evaluate_full_field_model_disclosure,
};

fn authority(task_id: &str) -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        "tenant-alpha",
        task_id,
        "customer-email",
        "case-resolution",
        Origin::parse("https://model-gateway.example").expect("valid destination origin"),
        DataClassification::PersonalData,
    )
}

fn route_request(authority: SensitiveDataAuthority, provider_id: &str) -> ModelRouteRequest {
    ModelRouteRequest::new(
        authority,
        provider_id,
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
        "subprocessors-reviewed-v1",
    )
}

fn route_scope(authority: SensitiveDataAuthority) -> ModelRouteScope {
    ModelRouteScope::new(
        authority,
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
        "subprocessors-reviewed-v1",
    )
}

fn invocation_request(
    authority: SensitiveDataAuthority,
    provider_id: &str,
) -> ModelInvocationRequest {
    ModelInvocationRequest::new(
        route_request(authority, provider_id),
        "case-resolution-prompt-v1",
        "customer-email-summary-v1",
        4_096,
        1_024,
        0,
    )
}

fn invocation_scope(authority: SensitiveDataAuthority) -> ModelInvocationScope {
    ModelInvocationScope::new(
        route_scope(authority),
        "case-resolution-prompt-v1",
        "customer-email-summary-v1",
        8_192,
        2_048,
        1_000,
    )
}

#[test]
fn full_field_disclosure_and_exact_reviewed_invocation_are_authorized() {
    let exact_authority = authority("task-42");
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope = DisclosureScope::new(
        exact_authority.clone(),
        DisclosureDecision::FullFieldDisclosure,
    );
    let invocation_request = invocation_request(exact_authority.clone(), "provider-private");
    let invocation_scope = invocation_scope(exact_authority);

    assert_eq!(
        evaluate_full_field_model_disclosure(
            &disclosure_request,
            &disclosure_scope,
            ModelDisclosureNecessity::NoLowerDisclosurePath,
            &invocation_request,
            &invocation_scope,
            999,
        ),
        ModelDisclosureDecision::Authorized
    );
}

#[test]
fn every_non_full_field_outcome_remains_non_authorizing_for_raw_model_input() {
    for decision in [
        DisclosureDecision::DenyAccess,
        DisclosureDecision::OpaqueHandleOnly,
        DisclosureDecision::DerivedValueOnly,
        DisclosureDecision::PartialFieldDisclosure,
        DisclosureDecision::HumanApprovalRequired,
        DisclosureDecision::DualControlRequired,
    ] {
        let exact_authority = authority("task-42");
        let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
        let disclosure_scope = DisclosureScope::new(exact_authority.clone(), decision);
        let invocation_request = invocation_request(exact_authority.clone(), "provider-private");
        let invocation_scope = invocation_scope(exact_authority);

        assert_eq!(
            evaluate_full_field_model_disclosure(
                &disclosure_request,
                &disclosure_scope,
                ModelDisclosureNecessity::NoLowerDisclosurePath,
                &invocation_request,
                &invocation_scope,
                999,
            ),
            ModelDisclosureDecision::DisclosureNotAuthorized(decision)
        );
    }
}

#[test]
fn disclosure_authority_cannot_be_composed_with_another_tasks_valid_invocation() {
    let disclosed_authority = authority("task-42");
    let invocation_authority = authority("task-99");
    let disclosure_request = SensitiveDataRequest::new(disclosed_authority.clone());
    let disclosure_scope =
        DisclosureScope::new(disclosed_authority, DisclosureDecision::FullFieldDisclosure);
    let invocation_request = invocation_request(invocation_authority.clone(), "provider-private");
    let invocation_scope = invocation_scope(invocation_authority);

    assert_eq!(
        evaluate_full_field_model_disclosure(
            &disclosure_request,
            &disclosure_scope,
            ModelDisclosureNecessity::NoLowerDisclosurePath,
            &invocation_request,
            &invocation_scope,
            999,
        ),
        ModelDisclosureDecision::AuthorityMismatch
    );
}

#[test]
fn invocation_denial_is_preserved_after_full_field_disclosure_authority() {
    let exact_authority = authority("task-42");
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope = DisclosureScope::new(
        exact_authority.clone(),
        DisclosureDecision::FullFieldDisclosure,
    );
    let invocation_request = invocation_request(exact_authority.clone(), "provider-other");
    let invocation_scope = invocation_scope(exact_authority);

    assert_eq!(
        evaluate_full_field_model_disclosure(
            &disclosure_request,
            &disclosure_scope,
            ModelDisclosureNecessity::NoLowerDisclosurePath,
            &invocation_request,
            &invocation_scope,
            999,
        ),
        ModelDisclosureDecision::InvocationDenied(ModelInvocationDecision::RouteDenied(
            ModelRouteDecision::RouteMismatch
        ))
    );
}
