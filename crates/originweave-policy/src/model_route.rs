//! Exact provider/model/region/retention/training/subprocessor/export route admission for sensitive-data workflows.
//!
//! This module evaluates route and invocation metadata only. An [`ModelRouteDecision::Authorized`]
//! or [`ModelInvocationDecision::Authorized`] result does not authorize disclosure of a protected
//! value, authenticate a provider, prove the provider's physical region, invoke a model, validate
//! model output, or choose a fallback. A trusted broker/orchestrator must independently authorize
//! the permitted value form, derive actual runtime identities from trusted configuration, derive
//! context-isolation metadata from the actual bounded outgoing message set, and supply invocation
//! time from the same authoritative time domain used to issue policy expiry.

use crate::sensitive_data::{
    DisclosureDecision, DisclosureScope, SensitiveDataAuthority, SensitiveDataRequest,
    evaluate_disclosure,
};

const MAX_ROUTE_IDENTIFIER_BYTES: usize = 128;
const DEFAULT_EXPORT_POLICY_ID: &str = "no-export";

/// Result of comparing one requested model route with its exact sensitive-data route authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRouteDecision {
    /// The exact sensitive-data authority and every route identifier match.
    Authorized,
    /// The sensitive-data authority is malformed or does not match the route scope.
    AuthorityMismatch,
    /// Provider/model/region/retention/training/subprocessor/export metadata is malformed or mismatched.
    RouteMismatch,
}

/// One proposed model route for an already classified sensitive field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRouteRequest {
    authority: SensitiveDataAuthority,
    provider_id: String,
    model_id: String,
    region_id: String,
    retention_policy_id: String,
    training_policy_id: String,
    subprocessor_policy_id: String,
    export_policy_id: String,
}

impl ModelRouteRequest {
    /// Build a requested model route that is explicitly non-exporting.
    ///
    /// Call [`Self::with_export_policy`] when a workflow requests a separately governed export
    /// behavior. Neither constructor nor export-policy selection grants disclosure authority.
    #[must_use]
    pub fn new(
        authority: SensitiveDataAuthority,
        provider_id: &str,
        model_id: &str,
        region_id: &str,
        retention_policy_id: &str,
        training_policy_id: &str,
        subprocessor_policy_id: &str,
    ) -> Self {
        Self {
            authority,
            provider_id: provider_id.to_owned(),
            model_id: model_id.to_owned(),
            region_id: region_id.to_owned(),
            retention_policy_id: retention_policy_id.to_owned(),
            training_policy_id: training_policy_id.to_owned(),
            subprocessor_policy_id: subprocessor_policy_id.to_owned(),
            export_policy_id: DEFAULT_EXPORT_POLICY_ID.to_owned(),
        }
    }

    /// Select one explicit export-policy identifier for this requested route.
    ///
    /// The identifier describes policy intent only. Matching an export-policy identifier does not
    /// execute an export or grant access to protected bytes; the later broker/export boundary must
    /// independently authorize and enforce the actual destination and value form.
    #[must_use]
    pub fn with_export_policy(mut self, export_policy_id: &str) -> Self {
        self.export_policy_id = export_policy_id.to_owned();
        self
    }

    fn route_identifiers_are_valid(&self) -> bool {
        route_identifier_is_valid(&self.provider_id)
            && route_identifier_is_valid(&self.model_id)
            && route_identifier_is_valid(&self.region_id)
            && route_identifier_is_valid(&self.retention_policy_id)
            && route_identifier_is_valid(&self.training_policy_id)
            && route_identifier_is_valid(&self.subprocessor_policy_id)
            && route_identifier_is_valid(&self.export_policy_id)
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
    training_policy_id: String,
    subprocessor_policy_id: String,
    export_policy_id: String,
}

impl ModelRouteScope {
    /// Build a non-exporting route scope from trusted policy metadata.
    ///
    /// Route identifiers are validated when the scope is evaluated so malformed policy state
    /// remains fail-closed rather than becoming authority merely because request and scope match.
    /// Call [`Self::with_export_policy`] when policy explicitly governs another export mode.
    #[must_use]
    pub fn new(
        authority: SensitiveDataAuthority,
        provider_id: &str,
        model_id: &str,
        region_id: &str,
        retention_policy_id: &str,
        training_policy_id: &str,
        subprocessor_policy_id: &str,
    ) -> Self {
        Self {
            authority,
            provider_id: provider_id.to_owned(),
            model_id: model_id.to_owned(),
            region_id: region_id.to_owned(),
            retention_policy_id: retention_policy_id.to_owned(),
            training_policy_id: training_policy_id.to_owned(),
            subprocessor_policy_id: subprocessor_policy_id.to_owned(),
            export_policy_id: DEFAULT_EXPORT_POLICY_ID.to_owned(),
        }
    }

    /// Select one explicit export-policy identifier for this trusted route scope.
    ///
    /// This is route metadata only. The eventual export path must separately verify protected-value
    /// disclosure, destination authority, retention, and any required human or dual-control approval.
    #[must_use]
    pub fn with_export_policy(mut self, export_policy_id: &str) -> Self {
        self.export_policy_id = export_policy_id.to_owned();
        self
    }

    fn route_identifiers_are_valid(&self) -> bool {
        route_identifier_is_valid(&self.provider_id)
            && route_identifier_is_valid(&self.model_id)
            && route_identifier_is_valid(&self.region_id)
            && route_identifier_is_valid(&self.retention_policy_id)
            && route_identifier_is_valid(&self.training_policy_id)
            && route_identifier_is_valid(&self.subprocessor_policy_id)
            && route_identifier_is_valid(&self.export_policy_id)
    }
}

/// Result of composing exact route admission with one reviewed model-invocation policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelInvocationDecision {
    /// Exact route, isolated context, prompt/schema contracts, token budgets, and policy lifetime are authorized.
    Authorized,
    /// Route admission failed before invocation-specific policy could authorize the request.
    RouteDenied(ModelRouteDecision),
    /// The broker detected unrelated conversation history in the sensitive invocation context.
    UnrelatedConversationHistoryDenied,
    /// Prompt/schema metadata, token budgets, or the reviewed expiry are malformed or out of scope.
    InvocationPolicyMismatch,
    /// The otherwise valid invocation policy is no longer fresh at the caller-supplied trusted time.
    InvocationExpired,
}

/// One proposed model invocation after a route has been selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInvocationRequest {
    route: ModelRouteRequest,
    prompt_contract_id: String,
    output_schema_id: String,
    input_tokens: u32,
    output_tokens: u32,
    unrelated_history_items: u32,
}

impl ModelInvocationRequest {
    /// Build one invocation request without authorizing protected-value disclosure or execution.
    ///
    /// `unrelated_history_items` must be derived by the trusted broker/orchestrator from the actual
    /// bounded outgoing message set. A caller-provided zero alone is not proof of isolation; this
    /// pure policy boundary only guarantees that a known nonzero count cannot be authorized.
    #[must_use]
    pub fn new(
        route: ModelRouteRequest,
        prompt_contract_id: &str,
        output_schema_id: &str,
        input_tokens: u32,
        output_tokens: u32,
        unrelated_history_items: u32,
    ) -> Self {
        Self {
            route,
            prompt_contract_id: prompt_contract_id.to_owned(),
            output_schema_id: output_schema_id.to_owned(),
            input_tokens,
            output_tokens,
            unrelated_history_items,
        }
    }
}

/// Reviewed invocation-policy scope layered on one exact model-route scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInvocationScope {
    route: ModelRouteScope,
    prompt_contract_id: String,
    output_schema_id: String,
    maximum_input_tokens: u32,
    maximum_output_tokens: u32,
    valid_until: u64,
}

impl ModelInvocationScope {
    /// Build trusted invocation policy for one prompt/schema pair, token maxima, and expiry.
    ///
    /// Identifiers, token maxima, and the exclusive `valid_until` value are validated during
    /// evaluation so malformed trusted policy remains fail-closed instead of becoming authority
    /// because request and scope happen to match. The expiry is only meaningful when compared with
    /// a trusted time from the same caller-owned authoritative time domain.
    #[must_use]
    pub fn new(
        route: ModelRouteScope,
        prompt_contract_id: &str,
        output_schema_id: &str,
        maximum_input_tokens: u32,
        maximum_output_tokens: u32,
        valid_until: u64,
    ) -> Self {
        Self {
            route,
            prompt_contract_id: prompt_contract_id.to_owned(),
            output_schema_id: output_schema_id.to_owned(),
            maximum_input_tokens,
            maximum_output_tokens,
            valid_until,
        }
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
/// requires valid, exact provider, model, region, retention-policy, training-policy, reviewed
/// subprocessor-policy, and export-policy identifiers. Keeping retention, training, subprocessor,
/// and export authority distinct prevents one permitted provider contract dimension from silently
/// authorizing another. The compatibility constructor defaults export policy to `no-export`, so an
/// omitted export choice never widens route authority. All route identifiers use bounded 1–128 byte
/// ASCII policy tokens containing alphanumeric characters plus `.`, `_`, `:`, and `-`.
#[must_use]
pub fn evaluate_model_route(
    request: &ModelRouteRequest,
    scope: &ModelRouteScope,
) -> ModelRouteDecision {
    let authority_decision = evaluate_disclosure(
        &SensitiveDataRequest::new(request.authority.clone()),
        &DisclosureScope::new(
            scope.authority.clone(),
            DisclosureDecision::OpaqueHandleOnly,
        ),
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
        || request.training_policy_id != scope.training_policy_id
        || request.subprocessor_policy_id != scope.subprocessor_policy_id
        || request.export_policy_id != scope.export_policy_id
    {
        ModelRouteDecision::RouteMismatch
    } else {
        ModelRouteDecision::Authorized
    }
}

/// Evaluate reviewed context isolation, prompt/schema, token limits, and lifetime after exact route admission.
///
/// Route admission remains a separate prerequisite and its failure is preserved in
/// [`ModelInvocationDecision::RouteDenied`]. Once the exact route is admitted, any broker-derived
/// nonzero unrelated-history count fails closed as
/// [`ModelInvocationDecision::UnrelatedConversationHistoryDenied`]. Invocation policy then requires
/// bounded 1–128 byte ASCII prompt/schema identifiers, exact identifier matches, nonzero requested
/// and trusted token budgets, request budgets no larger than the reviewed maxima, and a nonzero
/// exclusive expiry. After those static checks pass, `trusted_time >= valid_until` returns
/// [`ModelInvocationDecision::InvocationExpired`]. The caller must source `trusted_time` from the
/// same authoritative time domain used to issue `valid_until`; this pure policy function neither
/// reads a clock nor attests clock provenance. Authorization remains metadata-only: it does not
/// inspect messages, prove context isolation, disclose a protected value, invoke a provider,
/// validate output, retain/export data, or select a fallback route.
#[must_use]
pub fn evaluate_model_invocation(
    request: &ModelInvocationRequest,
    scope: &ModelInvocationScope,
    trusted_time: u64,
) -> ModelInvocationDecision {
    let route_decision = evaluate_model_route(&request.route, &scope.route);
    if route_decision != ModelRouteDecision::Authorized {
        return ModelInvocationDecision::RouteDenied(route_decision);
    }

    if request.unrelated_history_items != 0 {
        return ModelInvocationDecision::UnrelatedConversationHistoryDenied;
    }

    if !route_identifier_is_valid(&request.prompt_contract_id)
        || !route_identifier_is_valid(&request.output_schema_id)
        || !route_identifier_is_valid(&scope.prompt_contract_id)
        || !route_identifier_is_valid(&scope.output_schema_id)
        || request.prompt_contract_id != scope.prompt_contract_id
        || request.output_schema_id != scope.output_schema_id
        || request.input_tokens == 0
        || request.output_tokens == 0
        || scope.maximum_input_tokens == 0
        || scope.maximum_output_tokens == 0
        || scope.valid_until == 0
        || request.input_tokens > scope.maximum_input_tokens
        || request.output_tokens > scope.maximum_output_tokens
    {
        return ModelInvocationDecision::InvocationPolicyMismatch;
    }

    if trusted_time >= scope.valid_until {
        ModelInvocationDecision::InvocationExpired
    } else {
        ModelInvocationDecision::Authorized
    }
}

/// Result of composing an explicit full-field disclosure with a reviewed model invocation.
///
/// This decision authorizes only the metadata composition boundary. It carries no protected field
/// bytes and does not authenticate or invoke a provider, resolve an opaque handle, validate model
/// output, enforce retention, or prove that a non-model execution path was unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelDisclosureDecision {
    /// The exact full-field disclosure and reviewed model invocation both authorize the same field authority.
    Authorized,
    /// Sensitive-data policy did not explicitly authorize complete-field disclosure.
    DisclosureNotAuthorized(DisclosureDecision),
    /// The disclosure authority and model invocation belong to different exact authority tuples.
    AuthorityMismatch,
    /// Model-route or invocation policy denied the otherwise full-field-authorized request.
    InvocationDenied(ModelInvocationDecision),
}

/// Compose full-field sensitive-data disclosure with one exact reviewed model invocation.
///
/// Full-field disclosure is evaluated first so every weaker, approval-pending, malformed, or denied
/// sensitive-data outcome remains non-authorizing for raw model input. The disclosure request must
/// then carry the same complete [`SensitiveDataAuthority`] as the model invocation request; this
/// prevents independently valid policy metadata for different tenant/task/field/purpose/destination/
/// classification authorities from being combined. Only after those boundaries pass is the existing
/// route/context/prompt/schema/token/expiry evaluator consulted. `trusted_time` must come from the
/// same authoritative time domain as the reviewed invocation policy expiry.
///
/// An [`ModelDisclosureDecision::Authorized`] result still does not carry or disclose protected
/// bytes. A trusted broker must independently prove that full-field model disclosure is necessary,
/// authenticate the runtime/provider, resolve the value inside its transaction boundary, execute only
/// the authorized route, validate output, and enforce retention/export policy.
#[must_use]
pub fn evaluate_full_field_model_disclosure(
    disclosure_request: &SensitiveDataRequest,
    disclosure_scope: &DisclosureScope,
    invocation_request: &ModelInvocationRequest,
    invocation_scope: &ModelInvocationScope,
    trusted_time: u64,
) -> ModelDisclosureDecision {
    let disclosure_decision = evaluate_disclosure(disclosure_request, disclosure_scope);
    if disclosure_decision != DisclosureDecision::FullFieldDisclosure {
        return ModelDisclosureDecision::DisclosureNotAuthorized(disclosure_decision);
    }

    let invocation_authority_decision = evaluate_disclosure(
        disclosure_request,
        &DisclosureScope::new(
            invocation_request.route.authority.clone(),
            DisclosureDecision::FullFieldDisclosure,
        ),
    );
    if invocation_authority_decision != DisclosureDecision::FullFieldDisclosure {
        return ModelDisclosureDecision::AuthorityMismatch;
    }

    let invocation_decision =
        evaluate_model_invocation(invocation_request, invocation_scope, trusted_time);
    if invocation_decision == ModelInvocationDecision::Authorized {
        ModelDisclosureDecision::Authorized
    } else {
        ModelDisclosureDecision::InvocationDenied(invocation_decision)
    }
}
