#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, ModelFallbackDecision, ModelFallbackRequest, ModelFallbackScope,
    ModelRouteAvailability, ModelRouteDecision, ModelRouteRequest, ModelRouteScope,
    SensitiveDataAuthority, evaluate_model_fallback,
};

fn authority() -> SensitiveDataAuthority {
    SensitiveDataAuthority::new(
        "tenant-alpha",
        "task-42",
        "customer-email",
        "case-resolution",
        Origin::parse("https://model-gateway.example").expect("valid destination origin"),
        DataClassification::PersonalData,
    )
}

fn route_request(provider: &str, model: &str, region: &str) -> ModelRouteRequest {
    ModelRouteRequest::new(
        authority(),
        provider,
        model,
        region,
        "ephemeral-retention",
        "no-training",
        "subprocessors-reviewed-v1",
    )
}

fn route_scope(provider: &str, model: &str, region: &str) -> ModelRouteScope {
    ModelRouteScope::new(
        authority(),
        provider,
        model,
        region,
        "ephemeral-retention",
        "no-training",
        "subprocessors-reviewed-v1",
    )
}

fn primary_request() -> ModelRouteRequest {
    route_request("provider-primary", "model-primary-v1", "kr-central")
}

fn primary_scope() -> ModelRouteScope {
    route_scope("provider-primary", "model-primary-v1", "kr-central")
}

fn fallback_request() -> ModelRouteRequest {
    route_request("provider-fallback", "model-fallback-v1", "kr-central")
}

fn fallback_scope() -> ModelRouteScope {
    route_scope("provider-fallback", "model-fallback-v1", "kr-central")
}

#[test]
fn available_exact_primary_route_is_used_without_fallback() {
    let request = ModelFallbackRequest::new(primary_request(), ModelRouteAvailability::Available);
    let scope = ModelFallbackScope::new(primary_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope),
        ModelFallbackDecision::PrimaryAuthorized
    );
}

#[test]
fn primary_policy_mismatch_never_falls_back() {
    let request = ModelFallbackRequest::new(
        route_request("provider-unreviewed", "model-primary-v1", "kr-central"),
        ModelRouteAvailability::Unavailable,
    )
    .with_fallback(fallback_request());
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope),
        ModelFallbackDecision::PrimaryRouteDenied(ModelRouteDecision::RouteMismatch)
    );
}

#[test]
fn unknown_primary_availability_fails_closed() {
    let request = ModelFallbackRequest::new(primary_request(), ModelRouteAvailability::Unknown)
        .with_fallback(fallback_request());
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope),
        ModelFallbackDecision::PrimaryAvailabilityUnknown
    );
}

#[test]
fn unavailable_primary_without_reviewed_fallback_fails_closed() {
    let request = ModelFallbackRequest::new(primary_request(), ModelRouteAvailability::Unavailable);
    let scope = ModelFallbackScope::new(primary_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope),
        ModelFallbackDecision::PrimaryUnavailableNoReviewedFallback
    );
}

#[test]
fn fallback_must_exist_on_both_request_and_trusted_scope() {
    let request_only =
        ModelFallbackRequest::new(primary_request(), ModelRouteAvailability::Unavailable)
            .with_fallback(fallback_request());
    let no_fallback_scope = ModelFallbackScope::new(primary_scope());
    assert_eq!(
        evaluate_model_fallback(&request_only, &no_fallback_scope),
        ModelFallbackDecision::FallbackPolicyMismatch
    );

    let no_fallback_request =
        ModelFallbackRequest::new(primary_request(), ModelRouteAvailability::Unavailable);
    let scope_only = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());
    assert_eq!(
        evaluate_model_fallback(&no_fallback_request, &scope_only),
        ModelFallbackDecision::FallbackPolicyMismatch
    );
}

#[test]
fn unavailable_primary_can_use_only_an_exact_reviewed_fallback() {
    let request = ModelFallbackRequest::new(primary_request(), ModelRouteAvailability::Unavailable)
        .with_fallback(fallback_request());
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope),
        ModelFallbackDecision::ReviewedFallbackAuthorized
    );
}

#[test]
fn mismatched_reviewed_fallback_is_denied() {
    let request = ModelFallbackRequest::new(primary_request(), ModelRouteAvailability::Unavailable)
        .with_fallback(route_request(
            "provider-fallback",
            "model-unreviewed-v2",
            "kr-central",
        ));
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope),
        ModelFallbackDecision::FallbackRouteDenied(ModelRouteDecision::RouteMismatch)
    );
}
