#![allow(clippy::expect_used)]

//! Fail-closed policy contracts for model-route admission of sensitive data.
//!
//! Route admission is intentionally separate from disclosure authority: authorizing a
//! provider/model/region/retention/training tuple must never imply that raw protected values may
//! be disclosed. A later broker/orchestrator must independently authorize the value form and
//! derive the actual route identity from trusted runtime configuration.

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, ModelRouteDecision, ModelRouteRequest, ModelRouteScope,
    SensitiveDataAuthority, evaluate_model_route,
};

fn authority(destination: &str) -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        "tenant-alpha",
        "task-42",
        "customer-email",
        "case-resolution",
        Origin::parse(destination).expect("valid destination origin"),
        DataClassification::PersonalData,
    )
}

fn scope() -> ModelRouteScope {
    ModelRouteScope::new(
        authority("https://model-gateway.example"),
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
    )
}

fn request() -> ModelRouteRequest {
    ModelRouteRequest::new(
        authority("https://model-gateway.example"),
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
    )
}

#[test]
fn exact_sensitive_authority_and_model_route_are_admitted() {
    assert_eq!(
        evaluate_model_route(&request(), &scope()),
        ModelRouteDecision::Authorized
    );
}

#[test]
fn sensitive_authority_mismatch_is_distinct_from_route_mismatch() {
    let wrong_authority = ModelRouteRequest::new(
        authority("https://different-gateway.example"),
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
    );

    assert_eq!(
        evaluate_model_route(&wrong_authority, &scope()),
        ModelRouteDecision::AuthorityMismatch
    );
}

#[test]
fn every_model_route_dimension_is_exact_and_non_transferable() {
    let cases = [
        ModelRouteRequest::new(
            authority("https://model-gateway.example"),
            "provider-other",
            "model-reviewed-v1",
            "kr-central",
            "ephemeral-retention",
            "no-training",
        ),
        ModelRouteRequest::new(
            authority("https://model-gateway.example"),
            "provider-private",
            "model-other-v2",
            "kr-central",
            "ephemeral-retention",
            "no-training",
        ),
        ModelRouteRequest::new(
            authority("https://model-gateway.example"),
            "provider-private",
            "model-reviewed-v1",
            "us-east",
            "ephemeral-retention",
            "no-training",
        ),
        ModelRouteRequest::new(
            authority("https://model-gateway.example"),
            "provider-private",
            "model-reviewed-v1",
            "kr-central",
            "provider-default-retention",
            "no-training",
        ),
        ModelRouteRequest::new(
            authority("https://model-gateway.example"),
            "provider-private",
            "model-reviewed-v1",
            "kr-central",
            "ephemeral-retention",
            "training-allowed",
        ),
    ];

    for candidate in cases {
        assert_eq!(
            evaluate_model_route(&candidate, &scope()),
            ModelRouteDecision::RouteMismatch
        );
    }
}

#[test]
fn malformed_request_route_identifiers_fail_closed() {
    let malformed_values = ["", "contains space", "🚫", &"x".repeat(129)];

    for malformed in malformed_values {
        let candidates = [
            ModelRouteRequest::new(
                authority("https://model-gateway.example"),
                malformed,
                "model-reviewed-v1",
                "kr-central",
                "ephemeral-retention",
                "no-training",
            ),
            ModelRouteRequest::new(
                authority("https://model-gateway.example"),
                "provider-private",
                malformed,
                "kr-central",
                "ephemeral-retention",
                "no-training",
            ),
            ModelRouteRequest::new(
                authority("https://model-gateway.example"),
                "provider-private",
                "model-reviewed-v1",
                malformed,
                "ephemeral-retention",
                "no-training",
            ),
            ModelRouteRequest::new(
                authority("https://model-gateway.example"),
                "provider-private",
                "model-reviewed-v1",
                "kr-central",
                malformed,
                "no-training",
            ),
            ModelRouteRequest::new(
                authority("https://model-gateway.example"),
                "provider-private",
                "model-reviewed-v1",
                "kr-central",
                "ephemeral-retention",
                malformed,
            ),
        ];
        for candidate in candidates {
            assert_eq!(
                evaluate_model_route(&candidate, &scope()),
                ModelRouteDecision::RouteMismatch
            );
        }
    }
}

#[test]
fn malformed_scope_route_identifiers_fail_closed_against_a_valid_request() {
    let cases = [
        ModelRouteScope::new(
            authority("https://model-gateway.example"),
            "invalid provider",
            "model-reviewed-v1",
            "kr-central",
            "ephemeral-retention",
            "no-training",
        ),
        ModelRouteScope::new(
            authority("https://model-gateway.example"),
            "provider-private",
            "invalid model",
            "kr-central",
            "ephemeral-retention",
            "no-training",
        ),
        ModelRouteScope::new(
            authority("https://model-gateway.example"),
            "provider-private",
            "model-reviewed-v1",
            "invalid region",
            "ephemeral-retention",
            "no-training",
        ),
        ModelRouteScope::new(
            authority("https://model-gateway.example"),
            "provider-private",
            "model-reviewed-v1",
            "kr-central",
            "invalid retention",
            "no-training",
        ),
        ModelRouteScope::new(
            authority("https://model-gateway.example"),
            "provider-private",
            "model-reviewed-v1",
            "kr-central",
            "ephemeral-retention",
            "invalid training",
        ),
    ];

    for malformed_scope in cases {
        assert_eq!(
            evaluate_model_route(&request(), &malformed_scope),
            ModelRouteDecision::RouteMismatch
        );
    }
}

#[test]
fn malformed_sensitive_authority_fails_closed_even_when_both_sides_match() {
    let invalid_authority = SensitiveDataAuthority::new(
        "invalid tenant",
        "task-42",
        "customer-email",
        "case-resolution",
        Origin::parse("https://model-gateway.example").expect("valid destination origin"),
        DataClassification::PersonalData,
    );
    let invalid_request = ModelRouteRequest::new(
        invalid_authority.clone(),
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
    );
    let invalid_scope = ModelRouteScope::new(
        invalid_authority,
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
    );

    assert_eq!(
        evaluate_model_route(&invalid_request, &invalid_scope),
        ModelRouteDecision::AuthorityMismatch
    );
}
