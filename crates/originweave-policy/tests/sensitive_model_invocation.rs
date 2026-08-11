#![allow(clippy::expect_used)]

//! Fail-closed prompt, output-schema, token-budget, and expiry contracts for sensitive model use.
//!
//! Route admission is a prerequisite, not disclosure authority. This contract additionally binds
//! one reviewed prompt contract, one reviewed output schema, finite input/output token budgets, and
//! one exclusive invocation-policy expiry before a trusted broker/orchestrator may consider a model
//! invocation.

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, ModelInvocationDecision, ModelInvocationRequest, ModelInvocationScope,
    ModelRouteDecision, ModelRouteRequest, ModelRouteScope, SensitiveDataAuthority,
    evaluate_model_invocation,
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

fn route_request(provider_id: &str) -> ModelRouteRequest {
    ModelRouteRequest::new(
        authority("https://model-gateway.example"),
        provider_id,
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
        "subprocessors-reviewed-v1",
    )
}

fn route_scope() -> ModelRouteScope {
    ModelRouteScope::new(
        authority("https://model-gateway.example"),
        "provider-private",
        "model-reviewed-v1",
        "kr-central",
        "ephemeral-retention",
        "no-training",
        "subprocessors-reviewed-v1",
    )
}

fn request(input_tokens: u32, output_tokens: u32) -> ModelInvocationRequest {
    ModelInvocationRequest::new(
        route_request("provider-private"),
        "case-resolution-prompt-v1",
        "customer-email-summary-v1",
        input_tokens,
        output_tokens,
    )
}

fn scope(
    maximum_input_tokens: u32,
    maximum_output_tokens: u32,
    valid_until: u64,
) -> ModelInvocationScope {
    ModelInvocationScope::new(
        route_scope(),
        "case-resolution-prompt-v1",
        "customer-email-summary-v1",
        maximum_input_tokens,
        maximum_output_tokens,
        valid_until,
    )
}

#[test]
fn exact_route_prompt_schema_bounded_tokens_and_fresh_policy_are_authorized() {
    assert_eq!(
        evaluate_model_invocation(&request(4_096, 1_024), &scope(8_192, 2_048, 1_000), 999),
        ModelInvocationDecision::Authorized
    );
}

#[test]
fn route_denial_remains_distinct_from_invocation_policy_mismatch_or_expiry() {
    let request = ModelInvocationRequest::new(
        route_request("provider-other"),
        "case-resolution-prompt-v1",
        "customer-email-summary-v1",
        4_096,
        1_024,
    );

    assert_eq!(
        evaluate_model_invocation(&request, &scope(8_192, 2_048, 1), 1_000),
        ModelInvocationDecision::RouteDenied(ModelRouteDecision::RouteMismatch)
    );
}

#[test]
fn prompt_and_output_schema_contracts_are_exact() {
    let wrong_prompt = ModelInvocationRequest::new(
        route_request("provider-private"),
        "different-prompt-v2",
        "customer-email-summary-v1",
        4_096,
        1_024,
    );
    let wrong_schema = ModelInvocationRequest::new(
        route_request("provider-private"),
        "case-resolution-prompt-v1",
        "different-schema-v2",
        4_096,
        1_024,
    );

    for candidate in [wrong_prompt, wrong_schema] {
        assert_eq!(
            evaluate_model_invocation(&candidate, &scope(8_192, 2_048, 1_000), 999),
            ModelInvocationDecision::InvocationPolicyMismatch
        );
    }
}

#[test]
fn token_budgets_must_be_nonzero_and_within_reviewed_maxima() {
    for candidate in [
        request(0, 1_024),
        request(4_096, 0),
        request(8_193, 1_024),
        request(4_096, 2_049),
    ] {
        assert_eq!(
            evaluate_model_invocation(&candidate, &scope(8_192, 2_048, 1_000), 999),
            ModelInvocationDecision::InvocationPolicyMismatch
        );
    }

    for malformed_scope in [scope(0, 2_048, 1_000), scope(8_192, 0, 1_000)] {
        assert_eq!(
            evaluate_model_invocation(&request(4_096, 1_024), &malformed_scope, 999),
            ModelInvocationDecision::InvocationPolicyMismatch
        );
    }
}

#[test]
fn malformed_prompt_or_schema_policy_identifiers_fail_closed() {
    let malformed_values = ["", "contains space", "🚫", &"x".repeat(129)];

    for malformed in malformed_values {
        let candidates = [
            ModelInvocationRequest::new(
                route_request("provider-private"),
                malformed,
                "customer-email-summary-v1",
                4_096,
                1_024,
            ),
            ModelInvocationRequest::new(
                route_request("provider-private"),
                "case-resolution-prompt-v1",
                malformed,
                4_096,
                1_024,
            ),
        ];
        for candidate in candidates {
            assert_eq!(
                evaluate_model_invocation(&candidate, &scope(8_192, 2_048, 1_000), 999),
                ModelInvocationDecision::InvocationPolicyMismatch
            );
        }

        let scopes = [
            ModelInvocationScope::new(
                route_scope(),
                malformed,
                "customer-email-summary-v1",
                8_192,
                2_048,
                1_000,
            ),
            ModelInvocationScope::new(
                route_scope(),
                "case-resolution-prompt-v1",
                malformed,
                8_192,
                2_048,
                1_000,
            ),
        ];
        for malformed_scope in scopes {
            assert_eq!(
                evaluate_model_invocation(&request(4_096, 1_024), &malformed_scope, 999),
                ModelInvocationDecision::InvocationPolicyMismatch
            );
        }
    }
}

#[test]
fn invocation_policy_expiry_is_exclusive_and_fail_closed() {
    let policy = scope(8_192, 2_048, 1_000);

    assert_eq!(
        evaluate_model_invocation(&request(4_096, 1_024), &policy, 999),
        ModelInvocationDecision::Authorized
    );
    assert_eq!(
        evaluate_model_invocation(&request(4_096, 1_024), &policy, 1_000),
        ModelInvocationDecision::InvocationExpired
    );
    assert_eq!(
        evaluate_model_invocation(&request(4_096, 1_024), &policy, u64::MAX),
        ModelInvocationDecision::InvocationExpired
    );
}

#[test]
fn zero_expiry_is_invalid_but_maximum_epoch_remains_representable() {
    assert_eq!(
        evaluate_model_invocation(&request(4_096, 1_024), &scope(8_192, 2_048, 0), 0),
        ModelInvocationDecision::InvocationPolicyMismatch
    );
    assert_eq!(
        evaluate_model_invocation(
            &request(4_096, 1_024),
            &scope(8_192, 2_048, u64::MAX),
            u64::MAX - 1,
        ),
        ModelInvocationDecision::Authorized
    );
}
