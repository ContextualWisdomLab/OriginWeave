//! Purpose-bound sensitive-data disclosure and opaque-handle authority.
//!
//! This module carries authority metadata only. It never stores or exposes the
//! protected value itself, performs no I/O, and grants no authority from ambient
//! session, network, repository, or model state.

use originweave_core::Origin;

const MAX_AUTHORITY_IDENTIFIER_BYTES: usize = 128;

/// Classification applied to one protected field before disclosure policy runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClassification {
    /// Public information that does not require sensitive-data handling.
    PublicData,
    /// Internal information that is not intended for unrestricted disclosure.
    InternalData,
    /// Personal information associated with an identifiable person.
    PersonalData,
    /// Sensitive personal information requiring stronger disclosure controls.
    SensitivePersonalData,
    /// Authentication, authorization, or other credential material.
    CredentialData,
    /// Payment or financial account material.
    PaymentData,
}

/// The strongest disclosure action an exact authority scope permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureDecision {
    /// No disclosure is authorized.
    DenyAccess,
    /// Only an opaque broker handle may cross the policy boundary.
    OpaqueHandleOnly,
    /// Only a derived value may cross the policy boundary.
    DerivedValueOnly,
    /// A bounded subset of the field may be disclosed.
    PartialFieldDisclosure,
    /// The complete field may be disclosed to the exact bound destination.
    FullFieldDisclosure,
    /// Human approval is required before any requested disclosure.
    HumanApprovalRequired,
    /// Two independent controls must authorize the requested disclosure.
    DualControlRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthorityScope {
    tenant_id: String,
    task_id: String,
    field_id: String,
    purpose_id: String,
    destination: Origin,
}

impl AuthorityScope {
    fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: Origin,
    ) -> Self {
        Self {
            tenant_id: tenant_id.to_owned(),
            task_id: task_id.to_owned(),
            field_id: field_id.to_owned(),
            purpose_id: purpose_id.to_owned(),
            destination,
        }
    }

    fn is_complete(&self) -> bool {
        authority_identifier_is_valid(&self.tenant_id)
            && authority_identifier_is_valid(&self.task_id)
            && authority_identifier_is_valid(&self.field_id)
            && authority_identifier_is_valid(&self.purpose_id)
    }
}

fn authority_identifier_is_valid(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier.len() <= MAX_AUTHORITY_IDENTIFIER_BYTES
        && identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

/// One requested disclosure, without carrying the protected field value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDataRequest {
    authority: AuthorityScope,
    classification: DataClassification,
}

impl SensitiveDataRequest {
    /// Build a request bound to an exact tenant, task, field, purpose, validated destination, and class.
    ///
    /// Authority identifiers are admitted only as 1–128 byte ASCII policy tokens using
    /// alphanumeric characters plus `.`, `_`, `:`, and `-`; invalid identifiers remain
    /// fail-closed when the request is evaluated.
    #[must_use]
    pub fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: Origin,
        classification: DataClassification,
    ) -> Self {
        Self {
            authority: AuthorityScope::new(tenant_id, task_id, field_id, purpose_id, destination),
            classification,
        }
    }
}

/// Explicit authority for one requested sensitive-data disclosure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisclosureScope {
    authority: AuthorityScope,
    classification: DataClassification,
    decision: DisclosureDecision,
}

impl DisclosureScope {
    /// Build an exact disclosure authority scope and its maximum permitted outcome.
    ///
    /// Authority identifiers use the same bounded ASCII policy-token contract as
    /// [`SensitiveDataRequest`]; malformed scopes never grant disclosure.
    #[must_use]
    pub fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: Origin,
        classification: DataClassification,
        decision: DisclosureDecision,
    ) -> Self {
        Self {
            authority: AuthorityScope::new(tenant_id, task_id, field_id, purpose_id, destination),
            classification,
            decision,
        }
    }
}

/// Evaluate disclosure only from the exact request and explicit authority scope.
///
/// An incomplete or malformed authority scope fails closed even when both sides
/// contain the same invalid identifier.
#[must_use]
pub fn evaluate_disclosure(
    request: &SensitiveDataRequest,
    scope: &DisclosureScope,
) -> DisclosureDecision {
    if !request.authority.is_complete()
        || !scope.authority.is_complete()
        || request.authority != scope.authority
        || request.classification != scope.classification
    {
        DisclosureDecision::DenyAccess
    } else {
        scope.decision
    }
}

/// Result of evaluating one attempted use of an opaque sensitive-value handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleUseDecision {
    /// The supplied exact scope, expiry, and prior-use count permit broker admission.
    Authorized,
    /// Tenant, task, field, purpose, or destination did not match the handle scope.
    ScopeMismatch,
    /// The handle is no longer valid at the supplied trusted time.
    Expired,
    /// The bounded use count has already been consumed.
    UseLimitReached,
}

/// Authority metadata attached to an opaque sensitive-value handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveValueHandleScope {
    authority: AuthorityScope,
    expires_at_epoch_seconds: u64,
    max_uses: u32,
}

impl SensitiveValueHandleScope {
    /// Build an opaque-handle scope with an exclusive expiry and bounded use count.
    ///
    /// Authority identifiers use the same bounded ASCII policy-token contract as
    /// disclosure requests and scopes.
    #[must_use]
    pub fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: Origin,
        expires_at_epoch_seconds: u64,
        max_uses: u32,
    ) -> Self {
        Self {
            authority: AuthorityScope::new(tenant_id, task_id, field_id, purpose_id, destination),
            expires_at_epoch_seconds,
            max_uses,
        }
    }
}

/// One proposed use of an opaque sensitive-value handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleUseRequest {
    authority: AuthorityScope,
    now_epoch_seconds: u64,
    uses_so_far: u32,
}

impl HandleUseRequest {
    /// Build a handle-use evaluation request from trusted time and authoritative broker state.
    ///
    /// The eventual broker must supply these state values from its own trusted,
    /// caller-unforgeable storage; accepting this struct does not make arbitrary
    /// caller input authoritative. Authority identifiers use the same bounded ASCII
    /// policy-token contract as disclosure requests and scopes.
    #[must_use]
    pub fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: Origin,
        now_epoch_seconds: u64,
        uses_so_far: u32,
    ) -> Self {
        Self {
            authority: AuthorityScope::new(tenant_id, task_id, field_id, purpose_id, destination),
            now_epoch_seconds,
            uses_so_far,
        }
    }
}

/// Evaluate whether authoritative broker state is admissible for one handle use.
///
/// This pure function does not consume a use, mutate broker state, resolve a
/// handle, or release a protected value. It is therefore not standalone
/// enforcement. A trusted broker must obtain trusted time and caller-unforgeable
/// handle state, atomically reserve or increment the use count before value
/// resolution, and recheck the reserved scope immediately before disclosure.
/// Missing or malformed authority identifiers fail closed as a scope mismatch.
/// The caller must supply a destination that has already crossed the canonical
/// [`Origin`] boundary.
#[must_use]
pub fn evaluate_handle_use(
    request: &HandleUseRequest,
    scope: &SensitiveValueHandleScope,
) -> HandleUseDecision {
    if !request.authority.is_complete()
        || !scope.authority.is_complete()
        || request.authority != scope.authority
    {
        HandleUseDecision::ScopeMismatch
    } else if request.now_epoch_seconds >= scope.expires_at_epoch_seconds {
        HandleUseDecision::Expired
    } else if request.uses_so_far >= scope.max_uses {
        HandleUseDecision::UseLimitReached
    } else {
        HandleUseDecision::Authorized
    }
}
