//! Necessity-gated composition for exceptional full-field model disclosure.
//!
//! This module narrows the existing sensitive-data and reviewed-model policy composition with one
//! additional fail-closed precondition: raw protected field bytes may be considered for model input
//! only after a trusted broker or orchestrator has determined that no lower-disclosure execution path
//! can satisfy the approved task. Necessity evidence carries an exclusive validity horizon so an old
//! `NoLowerDisclosurePath` observation cannot be replayed indefinitely. The types here carry policy
//! metadata only; they never carry a protected value, inspect task state, authenticate a provider,
//! invoke a model, or prove that a caller-supplied necessity claim is truthful.

use crate::model_route::{
    ModelDisclosureDecision as InvocationDisclosureDecision, ModelInvocationDecision,
    ModelInvocationRequest, ModelInvocationScope,
    evaluate_full_field_model_disclosure as evaluate_disclosure_and_invocation,
};
use crate::sensitive_data::{DisclosureDecision, DisclosureScope, SensitiveDataRequest};

/// A lower-disclosure execution path that can satisfy the approved task without raw model input.
///
/// The trusted broker or orchestrator derives this classification from current task/runtime state.
/// Selecting a variant is evidence that full-field model disclosure is unnecessary, so the policy
/// composition fails closed before returning model-disclosure authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelDisclosureAlternative {
    /// An opaque handle can reach the trusted execution boundary without exposing the raw field to a model.
    OpaqueHandle,
    /// A deterministic transformation can produce the required task value without raw model input.
    DeterministicTransform,
    /// A local deterministic rule can complete the required decision or transformation.
    LocalRule,
    /// A reviewed structured tool can perform the operation without disclosing the raw field to a model.
    StructuredTool,
    /// A separately approved derived value is sufficient for the model-backed portion of the task.
    ApprovedDerivedValue,
}

/// Necessity classification supplied to the full-field model-disclosure composition boundary.
///
/// This pure policy type is not self-authenticating. A trusted broker or orchestrator must derive it
/// from the actual current task, available tools, handle capabilities, deterministic transforms, and
/// approved derived values immediately before protected-value resolution. Untrusted model or page
/// content must never be permitted to assert `NoLowerDisclosurePath` as authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelDisclosureNecessity {
    /// The trusted execution boundary found no lower-disclosure path able to satisfy the approved task.
    NoLowerDisclosurePath,
    /// A lower-disclosure path remains available, so raw protected model input is unnecessary.
    LowerDisclosurePathAvailable(ModelDisclosureAlternative),
}

/// Freshness-bounded necessity evidence for one full-field model-disclosure decision.
///
/// `valid_until` is an exclusive horizon in the same trusted time domain supplied to
/// [`evaluate_full_field_model_disclosure`]. A zero horizon is intentionally invalid and an evidence
/// value is expired when evaluation time is greater than or equal to the horizon. Construction does
/// not attest the clock or prove that the necessity classification was truthfully derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelDisclosureNecessityEvidence {
    necessity: ModelDisclosureNecessity,
    valid_until: u64,
}

impl ModelDisclosureNecessityEvidence {
    /// Build caller-supplied necessity evidence with an exclusive validity horizon.
    #[must_use]
    pub const fn new(necessity: ModelDisclosureNecessity, valid_until: u64) -> Self {
        Self {
            necessity,
            valid_until,
        }
    }

    /// Return the lower-disclosure-path classification carried by this evidence.
    #[must_use]
    pub const fn necessity(&self) -> ModelDisclosureNecessity {
        self.necessity
    }

    /// Return the exclusive trusted-time horizon for this evidence.
    #[must_use]
    pub const fn valid_until(&self) -> u64 {
        self.valid_until
    }
}

/// Result of composing sensitive disclosure, fresh necessity, and one reviewed model invocation.
///
/// Authorization is metadata-only. Even [`Self::Authorized`] does not resolve a protected value,
/// authenticate or contact a provider, validate output, attest runtime region/time, enforce retention,
/// or prove necessity independently of the trusted caller that derived [`ModelDisclosureNecessity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelDisclosureDecision {
    /// Full-field disclosure, fresh necessity, exact authority, and reviewed invocation policy authorize.
    Authorized,
    /// Sensitive-data policy did not explicitly authorize complete-field disclosure.
    DisclosureNotAuthorized(DisclosureDecision),
    /// Disclosure and invocation metadata belong to different exact sensitive-data authority tuples.
    AuthorityMismatch,
    /// The otherwise-authorized necessity evidence has no usable validity horizon.
    NecessityEvidenceInvalid,
    /// The otherwise-authorized necessity evidence is no longer fresh at trusted evaluation time.
    NecessityEvidenceExpired,
    /// A lower-disclosure path remains available, so raw protected model input is not necessary.
    FullFieldNotNecessary(ModelDisclosureAlternative),
    /// Model-route or invocation policy denied the otherwise full-field-authorized request.
    InvocationDenied(ModelInvocationDecision),
}

/// Compose an exceptional full-field disclosure with fresh necessity and one reviewed invocation.
///
/// The existing disclosure/invocation composition runs first so malformed, weaker, mismatched, expired,
/// or otherwise denied policy cannot be upgraded by a necessity claim. Only an otherwise authorized
/// composition reaches the necessity-evidence gate. A zero horizon fails as
/// [`ModelDisclosureDecision::NecessityEvidenceInvalid`]; an exclusive horizon at or before
/// `trusted_time` fails as [`ModelDisclosureDecision::NecessityEvidenceExpired`]. Only fresh evidence
/// is then evaluated for a known lower-disclosure alternative.
///
/// `necessity_evidence` must be derived by a trusted broker/orchestrator from current executable
/// alternatives and use the same trusted time domain as `trusted_time`. A fresh
/// `ModelDisclosureNecessity::NoLowerDisclosurePath` is still not proof merely because a caller
/// supplied it. The trusted value-resolution boundary must revalidate policy and lifecycle state
/// immediately before releasing protected bytes, execute only the exact authorized route, and enforce
/// output, retention, export, audit, and revocation controls.
#[must_use]
pub fn evaluate_full_field_model_disclosure(
    disclosure_request: &SensitiveDataRequest,
    disclosure_scope: &DisclosureScope,
    necessity_evidence: &ModelDisclosureNecessityEvidence,
    invocation_request: &ModelInvocationRequest,
    invocation_scope: &ModelInvocationScope,
    trusted_time: u64,
) -> ModelDisclosureDecision {
    match evaluate_disclosure_and_invocation(
        disclosure_request,
        disclosure_scope,
        invocation_request,
        invocation_scope,
        trusted_time,
    ) {
        InvocationDisclosureDecision::DisclosureNotAuthorized(decision) => {
            ModelDisclosureDecision::DisclosureNotAuthorized(decision)
        }
        InvocationDisclosureDecision::AuthorityMismatch => {
            ModelDisclosureDecision::AuthorityMismatch
        }
        InvocationDisclosureDecision::InvocationDenied(decision) => {
            ModelDisclosureDecision::InvocationDenied(decision)
        }
        InvocationDisclosureDecision::Authorized => {
            if necessity_evidence.valid_until == 0 {
                return ModelDisclosureDecision::NecessityEvidenceInvalid;
            }
            if trusted_time >= necessity_evidence.valid_until {
                return ModelDisclosureDecision::NecessityEvidenceExpired;
            }
            match necessity_evidence.necessity {
                ModelDisclosureNecessity::NoLowerDisclosurePath => {
                    ModelDisclosureDecision::Authorized
                }
                ModelDisclosureNecessity::LowerDisclosurePathAvailable(alternative) => {
                    ModelDisclosureDecision::FullFieldNotNecessary(alternative)
                }
            }
        }
    }
}
