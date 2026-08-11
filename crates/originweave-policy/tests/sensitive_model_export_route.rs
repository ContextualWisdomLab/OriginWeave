#![allow(clippy::expect_used)]

//! Export-policy binding regressions for sensitive model-route admission.
//!
//! Retention, training, reviewed subprocessors, and export are independent policy
//! dimensions. A route that is otherwise identical must not inherit export
//! authority merely because its provider/model/region tuple is approved.

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, ModelRouteDecision, ModelRouteRequest, ModelRouteScope,
    SensitiveDataAuthority, evaluate_model_route,
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

fn request(export_policy_id: &str) -> ModelRouteRequest {
    ModelRouteRequest::new_with_export_policy(
        authority(),
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
        "subprocessors-reviewed-v1",
        export_policy_id,
    )
}

fn scope(export_policy_id: &str) -> ModelRouteScope {
    ModelRouteScope::new_with_export_policy(
        authority(),
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
        "subprocessors-reviewed-v1",
        export_policy_id,
    )
}

#[test]
fn exact_export_policy_is_part_of_model_route_authority() {
    assert_eq!(
        evaluate_model_route(&request("no-export"), &scope("no-export")),
        ModelRouteDecision::Authorized
    );
    assert_eq!(
        evaluate_model_route(&request("approved-export"), &scope("no-export")),
        ModelRouteDecision::RouteMismatch
    );
}

#[test]
fn malformed_requested_export_policy_fails_closed() {
    assert_eq!(
        evaluate_model_route(&request("invalid export"), &scope("no-export")),
        ModelRouteDecision::RouteMismatch
    );
}

#[test]
fn malformed_scope_export_policy_fails_closed_against_valid_request() {
    assert_eq!(
        evaluate_model_route(&request("no-export"), &scope("invalid export")),
        ModelRouteDecision::RouteMismatch
    );
}
