#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, ModelFallbackDecision, ModelFallbackRequest, ModelFallbackScope,
    ModelRouteAvailability, ModelRouteAvailabilityEvidence, ModelRouteDecision, ModelRouteRequest,
    ModelRouteScope, SensitiveDataAuthority, evaluate_model_fallback,
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

fn availability(
    route: &ModelRouteRequest,
    state: ModelRouteAvailability,
    valid_until: u64,
) -> ModelRouteAvailabilityEvidence {
    ModelRouteAvailabilityEvidence::new(route.clone(), state, valid_until)
}

#[test]
fn available_exact_primary_route_is_used_with_fresh_evidence() {
    let primary = primary_request();
    let request = ModelFallbackRequest::new(
        primary.clone(),
        availability(&primary, ModelRouteAvailability::Available, 101),
    );
    let scope = ModelFallbackScope::new(primary_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 100),
        ModelFallbackDecision::PrimaryAuthorized
    );
}

#[test]
fn primary_policy_mismatch_precedes_availability_route_binding() {
    let unreviewed_primary = route_request("provider-unreviewed", "model-primary-v1", "kr-central");
    let unrelated_evidence_route = primary_request();
    let request = ModelFallbackRequest::new(
        unreviewed_primary,
        availability(
            &unrelated_evidence_route,
            ModelRouteAvailability::Unavailable,
            100,
        ),
    )
    .with_fallback(fallback_request());
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 100),
        ModelFallbackDecision::PrimaryRouteDenied(ModelRouteDecision::RouteMismatch)
    );
}

#[test]
fn availability_evidence_for_another_route_fails_closed_before_freshness() {
    let primary = primary_request();
    let other_route = route_request("provider-primary", "model-primary-v2", "kr-central");
    let request = ModelFallbackRequest::new(
        primary,
        availability(&other_route, ModelRouteAvailability::Unavailable, 100),
    )
    .with_fallback(fallback_request());
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 100),
        ModelFallbackDecision::PrimaryAvailabilityRouteMismatch
    );
}

#[test]
fn malformed_availability_lifetime_fails_closed() {
    let primary = primary_request();
    let request = ModelFallbackRequest::new(
        primary.clone(),
        availability(&primary, ModelRouteAvailability::Available, 0),
    );
    let scope = ModelFallbackScope::new(primary_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 0),
        ModelFallbackDecision::PrimaryAvailabilityInvalid
    );
}

#[test]
fn expired_primary_availability_fails_closed_at_exclusive_boundary() {
    let primary = primary_request();
    let request = ModelFallbackRequest::new(
        primary.clone(),
        availability(&primary, ModelRouteAvailability::Available, 100),
    );
    let scope = ModelFallbackScope::new(primary_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 100),
        ModelFallbackDecision::PrimaryAvailabilityExpired
    );
}

#[test]
fn unknown_primary_availability_fails_closed() {
    let primary = primary_request();
    let request = ModelFallbackRequest::new(
        primary.clone(),
        availability(&primary, ModelRouteAvailability::Unknown, 101),
    )
    .with_fallback(fallback_request());
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 100),
        ModelFallbackDecision::PrimaryAvailabilityUnknown
    );
}

#[test]
fn unavailable_primary_without_reviewed_fallback_fails_closed() {
    let primary = primary_request();
    let request = ModelFallbackRequest::new(
        primary.clone(),
        availability(&primary, ModelRouteAvailability::Unavailable, 101),
    );
    let scope = ModelFallbackScope::new(primary_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 100),
        ModelFallbackDecision::PrimaryUnavailableNoReviewedFallback
    );
}

#[test]
fn fallback_must_exist_on_both_request_and_trusted_scope() {
    let request_primary = primary_request();
    let request_only = ModelFallbackRequest::new(
        request_primary.clone(),
        availability(
            &request_primary,
            ModelRouteAvailability::Unavailable,
            101,
        ),
    )
    .with_fallback(fallback_request());
    let no_fallback_scope = ModelFallbackScope::new(primary_scope());
    assert_eq!(
        evaluate_model_fallback(&request_only, &no_fallback_scope, 100),
        ModelFallbackDecision::FallbackPolicyMismatch
    );

    let scope_primary = primary_request();
    let no_fallback_request = ModelFallbackRequest::new(
        scope_primary.clone(),
        availability(&scope_primary, ModelRouteAvailability::Unavailable, 101),
    );
    let scope_only = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());
    assert_eq!(
        evaluate_model_fallback(&no_fallback_request, &scope_only, 100),
        ModelFallbackDecision::FallbackPolicyMismatch
    );
}

#[test]
fn unavailable_primary_can_use_only_an_exact_reviewed_fallback() {
    let primary = primary_request();
    let request = ModelFallbackRequest::new(
        primary.clone(),
        availability(&primary, ModelRouteAvailability::Unavailable, 101),
    )
    .with_fallback(fallback_request());
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 100),
        ModelFallbackDecision::ReviewedFallbackAuthorized
    );
}

#[test]
fn mismatched_reviewed_fallback_is_denied() {
    let primary = primary_request();
    let request = ModelFallbackRequest::new(
        primary.clone(),
        availability(&primary, ModelRouteAvailability::Unavailable, 101),
    )
    .with_fallback(route_request(
        "provider-fallback",
        "model-unreviewed-v2",
        "kr-central",
    ));
    let scope = ModelFallbackScope::new(primary_scope()).with_fallback(fallback_scope());

    assert_eq!(
        evaluate_model_fallback(&request, &scope, 100),
        ModelFallbackDecision::FallbackRouteDenied(ModelRouteDecision::RouteMismatch)
    );
}
