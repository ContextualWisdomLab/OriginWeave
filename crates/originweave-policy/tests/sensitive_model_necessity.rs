#![allow(clippy::expect_used)]

//! Fail-first contract requiring a fresh lower-disclosure-path check before raw model input.

use originweave_core::Origin;
use originweave_policy::{
    DataClassification, DisclosureDecision, DisclosureScope, ModelDisclosureAlternative,
    ModelDisclosureDecision, ModelDisclosureNecessity, ModelDisclosureNecessityEvidence,
    ModelInvocationRequest, ModelInvocationScope, ModelRouteRequest, ModelRouteScope,
    SensitiveDataAuthority, SensitiveDataRequest, evaluate_full_field_model_disclosure,
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

fn invocation_request(authority: SensitiveDataAuthority) -> ModelInvocationRequest {
    ModelInvocationRequest::new(
        ModelRouteRequest::new(
            authority,
            "provider-private",
            "model-reviewed-v1",
            "kr-central",
            "ephemeral-retention",
            "no-training",
            "subprocessors-reviewed-v1",
        ),
        "case-resolution-prompt-v1",
        "customer-email-summary-v1",
        4_096,
        1_024,
        0,
    )
}

fn invocation_scope(authority: SensitiveDataAuthority) -> ModelInvocationScope {
    ModelInvocationScope::new(
        ModelRouteScope::new(
            authority,
            "provider-private",
            "model-reviewed-v1",
            "kr-central",
            "ephemeral-retention",
            "no-training",
            "subprocessors-reviewed-v1",
        ),
        "case-resolution-prompt-v1",
        "customer-email-summary-v1",
        8_192,
        2_048,
        1_001,
    )
}

fn necessity_evidence(necessity: ModelDisclosureNecessity) -> ModelDisclosureNecessityEvidence {
    ModelDisclosureNecessityEvidence::new(necessity, 1_000)
}

#[test]
fn exact_full_field_model_disclosure_requires_no_lower_disclosure_path() {
    let exact_authority = authority();
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope = DisclosureScope::new(
        exact_authority.clone(),
        DisclosureDecision::FullFieldDisclosure,
    );
    let invocation_request = invocation_request(exact_authority.clone());
    let invocation_scope = invocation_scope(exact_authority);
    let necessity = necessity_evidence(ModelDisclosureNecessity::NoLowerDisclosurePath);

    assert_eq!(
        necessity.necessity(),
        ModelDisclosureNecessity::NoLowerDisclosurePath
    );
    assert_eq!(necessity.valid_until(), 1_000);
    assert_eq!(
        evaluate_full_field_model_disclosure(
            &disclosure_request,
            &disclosure_scope,
            &necessity,
            &invocation_request,
            &invocation_scope,
            999,
        ),
        ModelDisclosureDecision::Authorized
    );
}

#[test]
fn any_available_lower_disclosure_path_blocks_raw_model_input() {
    for alternative in [
        ModelDisclosureAlternative::OpaqueHandle,
        ModelDisclosureAlternative::DeterministicTransform,
        ModelDisclosureAlternative::LocalRule,
        ModelDisclosureAlternative::StructuredTool,
        ModelDisclosureAlternative::ApprovedDerivedValue,
    ] {
        let exact_authority = authority();
        let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
        let disclosure_scope = DisclosureScope::new(
            exact_authority.clone(),
            DisclosureDecision::FullFieldDisclosure,
        );
        let invocation_request = invocation_request(exact_authority.clone());
        let invocation_scope = invocation_scope(exact_authority);

        assert_eq!(
            evaluate_full_field_model_disclosure(
                &disclosure_request,
                &disclosure_scope,
                &necessity_evidence(ModelDisclosureNecessity::LowerDisclosurePathAvailable(
                    alternative,
                )),
                &invocation_request,
                &invocation_scope,
                999,
            ),
            ModelDisclosureDecision::FullFieldNotNecessary(alternative)
        );
    }
}

#[test]
fn necessity_never_upgrades_a_weaker_disclosure_decision() {
    let exact_authority = authority();
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope = DisclosureScope::new(
        exact_authority.clone(),
        DisclosureDecision::OpaqueHandleOnly,
    );
    let invocation_request = invocation_request(exact_authority.clone());
    let invocation_scope = invocation_scope(exact_authority);

    assert_eq!(
        evaluate_full_field_model_disclosure(
            &disclosure_request,
            &disclosure_scope,
            &ModelDisclosureNecessityEvidence::new(
                ModelDisclosureNecessity::NoLowerDisclosurePath,
                0,
            ),
            &invocation_request,
            &invocation_scope,
            999,
        ),
        ModelDisclosureDecision::DisclosureNotAuthorized(DisclosureDecision::OpaqueHandleOnly)
    );
}

#[test]
fn zero_necessity_horizon_fails_closed_after_existing_authority_passes() {
    let exact_authority = authority();
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope = DisclosureScope::new(
        exact_authority.clone(),
        DisclosureDecision::FullFieldDisclosure,
    );
    let invocation_request = invocation_request(exact_authority.clone());
    let invocation_scope = invocation_scope(exact_authority);

    assert_eq!(
        evaluate_full_field_model_disclosure(
            &disclosure_request,
            &disclosure_scope,
            &ModelDisclosureNecessityEvidence::new(
                ModelDisclosureNecessity::NoLowerDisclosurePath,
                0,
            ),
            &invocation_request,
            &invocation_scope,
            999,
        ),
        ModelDisclosureDecision::NecessityEvidenceInvalid
    );
}

#[test]
fn necessity_horizon_is_exclusive_and_cannot_be_replayed_at_expiry() {
    let exact_authority = authority();
    let disclosure_request = SensitiveDataRequest::new(exact_authority.clone());
    let disclosure_scope = DisclosureScope::new(
        exact_authority.clone(),
        DisclosureDecision::FullFieldDisclosure,
    );
    let invocation_request = invocation_request(exact_authority.clone());
    let invocation_scope = invocation_scope(exact_authority);

    assert_eq!(
        evaluate_full_field_model_disclosure(
            &disclosure_request,
            &disclosure_scope,
            &ModelDisclosureNecessityEvidence::new(
                ModelDisclosureNecessity::NoLowerDisclosurePath,
                1_000,
            ),
            &invocation_request,
            &invocation_scope,
            1_000,
        ),
        ModelDisclosureDecision::NecessityEvidenceExpired
    );
}
