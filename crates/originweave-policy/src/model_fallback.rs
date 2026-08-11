//! Fail-closed fallback selection layered on exact sensitive-model route authority.
//!
//! This module consumes caller-supplied provider availability evidence only after the primary route
//! itself passes [`crate::evaluate_model_route`]. Availability evidence is bound to the exact route it
//! describes, carries an exclusive validity horizon, and is evaluated against trusted time supplied
//! by the broker/orchestrator. This module performs no provider health check, clock attestation,
//! retry, network I/O, protected-value disclosure, model invocation, or execution of the selected
//! route. A trusted broker/orchestrator must derive availability from an authoritative runtime
//! boundary and may execute only the exact route authorized by this deterministic policy.

use crate::{ModelRouteDecision, ModelRouteRequest, ModelRouteScope, evaluate_model_route};

/// Trusted runtime availability classification for the exact reviewed primary model route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRouteAvailability {
    /// The trusted runtime boundary reports that the exact primary route is available.
    Available,
    /// The trusted runtime boundary reports that the exact primary route is unavailable.
    Unavailable,
    /// Availability is missing, contradictory, or otherwise not trustworthy enough to use.
    Unknown,
}

/// Availability evidence for one exact primary route with an exclusive validity horizon.
///
/// The embedded route identifies the exact provider/model/region/retention/training/subprocessor/
/// export authority tuple observed by the trusted runtime boundary. `valid_until` belongs to the same
/// trusted time domain supplied later to [`evaluate_model_fallback`]. A zero horizon is intentionally
/// invalid, and evidence is expired when evaluation time is greater than or equal to the horizon.
/// Constructing this value does not attest the route identity, clock, or provider health.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRouteAvailabilityEvidence {
    route: ModelRouteRequest,
    state: ModelRouteAvailability,
    valid_until: u64,
}

impl ModelRouteAvailabilityEvidence {
    /// Build route-bound availability evidence with an exclusive validity horizon.
    #[must_use]
    pub fn new(route: ModelRouteRequest, state: ModelRouteAvailability, valid_until: u64) -> Self {
        Self {
            route,
            state,
            valid_until,
        }
    }
}

/// One proposed primary route and optional fallback after trusted availability observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelFallbackRequest {
    primary_route: ModelRouteRequest,
    primary_availability: ModelRouteAvailabilityEvidence,
    fallback_route: Option<ModelRouteRequest>,
}

impl ModelFallbackRequest {
    /// Build a request for one primary route without an alternate route.
    ///
    /// `primary_availability` is evidence supplied by a trusted runtime boundary; constructing this
    /// value does not prove provider health or grant permission to disclose protected data.
    #[must_use]
    pub fn new(
        primary_route: ModelRouteRequest,
        primary_availability: ModelRouteAvailabilityEvidence,
    ) -> Self {
        Self {
            primary_route,
            primary_availability,
            fallback_route: None,
        }
    }

    /// Attach one explicitly proposed fallback route.
    ///
    /// The route is still denied unless trusted policy independently contains the exact same reviewed
    /// fallback scope and the existing route evaluator authorizes it.
    #[must_use]
    pub fn with_fallback(mut self, fallback_route: ModelRouteRequest) -> Self {
        self.fallback_route = Some(fallback_route);
        self
    }
}

/// Trusted primary route and optional separately reviewed fallback route authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelFallbackScope {
    primary_route: ModelRouteScope,
    fallback_route: Option<ModelRouteScope>,
}

impl ModelFallbackScope {
    /// Build trusted fallback policy with no alternate route authorized.
    #[must_use]
    pub fn new(primary_route: ModelRouteScope) -> Self {
        Self {
            primary_route,
            fallback_route: None,
        }
    }

    /// Attach one exact separately reviewed fallback route scope.
    ///
    /// Presence in this scope does not make the route healthy or execute it; it only permits the
    /// evaluator to consider that exact route after the primary has been authorized then observed
    /// unavailable.
    #[must_use]
    pub fn with_fallback(mut self, fallback_route: ModelRouteScope) -> Self {
        self.fallback_route = Some(fallback_route);
        self
    }
}

/// Result of composing exact primary route authority, trusted availability, and reviewed fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFallbackDecision {
    /// The exact primary route is policy-authorized and reported available by fresh evidence.
    PrimaryAuthorized,
    /// Primary route policy failed; availability and fallback are intentionally not considered.
    PrimaryRouteDenied(ModelRouteDecision),
    /// Availability evidence belongs to a different route than the authorized primary request.
    PrimaryAvailabilityRouteMismatch,
    /// The primary route is authorized but availability evidence has an invalid lifetime.
    PrimaryAvailabilityInvalid,
    /// The primary route is authorized but its availability evidence is no longer fresh.
    PrimaryAvailabilityExpired,
    /// The primary route is authorized but its runtime availability cannot be trusted.
    PrimaryAvailabilityUnknown,
    /// The primary route is unavailable and policy contains no reviewed fallback route.
    PrimaryUnavailableNoReviewedFallback,
    /// Request and trusted scope disagree about whether a fallback route exists.
    FallbackPolicyMismatch,
    /// The primary route is unavailable and the exact separately reviewed fallback route is authorized.
    ReviewedFallbackAuthorized,
    /// A proposed fallback exists in both request and scope but its exact route policy denied it.
    FallbackRouteDenied(ModelRouteDecision),
}

/// Evaluate fail-closed sensitive-model fallback selection without executing a model route.
///
/// `trusted_time` must come from the same authoritative time domain as the availability horizon.
/// Primary route policy is always evaluated first, so malformed or mismatched primary authority can
/// never become a fallback trigger. After primary authorization, availability evidence must describe
/// the exact same primary route before lifetime or state is considered. A zero horizon is invalid and
/// an exclusive horizon at or before `trusted_time` is expired. Unknown fresh availability also fails
/// closed. Only fresh explicit `Unavailable` evidence permits fallback consideration, and only when
/// request and trusted scope both carry a fallback that independently passes the existing exact route
/// evaluator.
#[must_use]
pub fn evaluate_model_fallback(
    request: &ModelFallbackRequest,
    scope: &ModelFallbackScope,
    trusted_time: u64,
) -> ModelFallbackDecision {
    let primary_decision = evaluate_model_route(&request.primary_route, &scope.primary_route);
    if primary_decision != ModelRouteDecision::Authorized {
        return ModelFallbackDecision::PrimaryRouteDenied(primary_decision);
    }

    if request.primary_availability.route != request.primary_route {
        return ModelFallbackDecision::PrimaryAvailabilityRouteMismatch;
    }
    if request.primary_availability.valid_until == 0 {
        return ModelFallbackDecision::PrimaryAvailabilityInvalid;
    }
    if trusted_time >= request.primary_availability.valid_until {
        return ModelFallbackDecision::PrimaryAvailabilityExpired;
    }

    match request.primary_availability.state {
        ModelRouteAvailability::Available => ModelFallbackDecision::PrimaryAuthorized,
        ModelRouteAvailability::Unknown => ModelFallbackDecision::PrimaryAvailabilityUnknown,
        ModelRouteAvailability::Unavailable => {
            match (&request.fallback_route, &scope.fallback_route) {
                (None, None) => ModelFallbackDecision::PrimaryUnavailableNoReviewedFallback,
                (Some(_), None) | (None, Some(_)) => ModelFallbackDecision::FallbackPolicyMismatch,
                (Some(fallback_request), Some(fallback_scope)) => {
                    let fallback_decision = evaluate_model_route(fallback_request, fallback_scope);
                    if fallback_decision == ModelRouteDecision::Authorized {
                        ModelFallbackDecision::ReviewedFallbackAuthorized
                    } else {
                        ModelFallbackDecision::FallbackRouteDenied(fallback_decision)
                    }
                }
            }
        }
    }
}
