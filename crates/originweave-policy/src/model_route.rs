//! Exact provider/model/region/retention route admission for sensitive-data workflows.
//!
//! This module evaluates route metadata only. An [`ModelRouteDecision::Authorized`] result does
//! not authorize disclosure of a protected value, authenticate a provider, prove the provider's
//! physical region, invoke a model, or choose a fallback. A trusted broker/orchestrator must
//! independently authorize the permitted value form and derive the actual route identity from
//! trusted runtime configuration.

use crate::sensitive_data::{
    DisclosureDecision, DisclosureScope, SensitiveDataAuthority, SensitiveDataRequest,
    evaluate_disclosure,
};

const MAX_ROUTE_IDENTIFIER_BYTES: usize = 128;

/// Result of comparing one requested model route with its exact sensitive-data route authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRouteDecision {
    /// The exact sensitive-data authority and every route identifier match.
    Authorized,
    /// The sensitive-data authority is malformed or does not match the route scope.
    AuthorityMismatch,
    /// Provider, model, region, or retention-policy route metadata is malformed or does not match.
    RouteMismatch,
}

/// One proposed provider/model/region/retention route for an already classified sensitive field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRouteRequest {
    authority: SensitiveDataAuthority,
    provider_id: String,
    model_id: String,
    region_id: String,
    retention_policy_id: String,
}

impl ModelRouteRequest {
    /// Build a requested model route without granting disclosure authority.
    #[must_use]
    pub fn new(
        authority: SensitiveDataAuthority,
        provider_id: &str,
        model_id: &str,
        region_id: &str,
        retention_policy_id: &str,
    ) -> Self {
        Self {
            authority,
            provider_id: provider_id.to_owned(),
            model_id: model_id.to_owned(),
            region_id: region_id.to_owned(),
            retention_policy_id: retention_policy_id.to_owned(),
        }
    }

    fn route_identifiers_are_valid(&self) -> bool {
        route_identifier_is_valid(&self.provider_id)
            && route_identifier_is_valid(&self.model_id)
            && route_identifier_is_valid(&self.region_id)
            && route_identifier_is_valid(&self.retention_policy_id)
    }
}

/// Exact model-route authority for one existing sensitive-data authority tuple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRouteScope {
    authority: SensitiveDataAuthority,
    provider_id: String,
    model_id: String,
    region_id: String,
    retention_policy_id: String,
}

impl ModelRouteScope {
    /// Build a route scope from trusted policy metadata.
    ///
    /// Route identifiers are validated when the scope is evaluated so malformed policy state
    /// remains fail-closed rather than becoming authority merely because request and scope match.
    #[must_use]
    pub fn new(
        authority: SensitiveDataAuthority,
        provider_id: &str,
        model_id: &str,
        region_id: &str,
        retention_policy_id: &str,
    ) -> Self {
        Self {
            authority,
            provider_id: provider_id.to_owned(),
            model_id: model_id.to_owned(),
            region_id: region_id.to_owned(),
            retention_policy_id: retention_policy_id.to_owned(),
        }
    }

    fn route_identifiers_are_valid(&self) -> bool {
        route_identifier_is_valid(&self.provider_id)
            && route_identifier_is_valid(&self.model_id)
            && route_identifier_is_valid(&self.region_id)
            && route_identifier_is_valid(&self.retention_policy_id)
    }
}

fn route_identifier_is_valid(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier.len() <= MAX_ROUTE_IDENTIFIER_BYTES
        && identifier.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

/// Evaluate exact model-route admission without authorizing protected-value disclosure.
///
/// The existing sensitive-data disclosure comparator is reused only to validate that request and
/// scope carry the same complete [`SensitiveDataAuthority`]. Route admission then independently
/// requires valid, exact provider, model, region, and retention-policy identifiers. All route
/// identifiers use bounded 1–128 byte ASCII policy tokens containing alphanumeric characters plus
/// `.`, `_`, `:`, and `-`.
#[must_use]
pub fn evaluate_model_route(
    request: &ModelRouteRequest,
    scope: &ModelRouteScope,
) -> ModelRouteDecision {
    let authority_decision = evaluate_disclosure(
        &SensitiveDataRequest::new(request.authority.clone()),
        &DisclosureScope::new(scope.authority.clone(), DisclosureDecision::OpaqueHandleOnly),
    );
    if authority_decision == DisclosureDecision::DenyAccess {
        return ModelRouteDecision::AuthorityMismatch;
    }

    if !request.route_identifiers_are_valid()
        || !scope.route_identifiers_are_valid()
        || request.provider_id != scope.provider_id
        || request.model_id != scope.model_id
        || request.region_id != scope.region_id
        || request.retention_policy_id != scope.retention_policy_id
    {
        ModelRouteDecision::RouteMismatch
    } else {
        ModelRouteDecision::Authorized
    }
}
