//! Purpose-bound sensitive-data disclosure and opaque-handle authority.
//!
//! This module carries authority metadata only. It never stores or exposes the
//! protected value itself, performs no I/O, and grants no authority from ambient
//! session, network, repository, or model state.

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
    destination: String,
}

impl AuthorityScope {
    fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: &str,
    ) -> Self {
        Self {
            tenant_id: tenant_id.to_owned(),
            task_id: task_id.to_owned(),
            field_id: field_id.to_owned(),
            purpose_id: purpose_id.to_owned(),
            destination: destination.to_owned(),
        }
    }
}

/// One requested disclosure, without carrying the protected field value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDataRequest {
    authority: AuthorityScope,
    classification: DataClassification,
}

impl SensitiveDataRequest {
    /// Build a request bound to an exact tenant, task, field, purpose, destination, and class.
    #[must_use]
    pub fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: &str,
        classification: DataClassification,
    ) -> Self {
        Self {
            authority: AuthorityScope::new(
                tenant_id,
                task_id,
                field_id,
                purpose_id,
                destination,
            ),
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
    #[must_use]
    pub fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: &str,
        classification: DataClassification,
        decision: DisclosureDecision,
    ) -> Self {
        Self {
            authority: AuthorityScope::new(
                tenant_id,
                task_id,
                field_id,
                purpose_id,
                destination,
            ),
            classification,
            decision,
        }
    }
}

/// Evaluate disclosure only from the exact request and explicit authority scope.
#[must_use]
pub fn evaluate_disclosure(
    request: &SensitiveDataRequest,
    scope: &DisclosureScope,
) -> DisclosureDecision {
    if request.authority != scope.authority || request.classification != scope.classification {
        DisclosureDecision::DenyAccess
    } else {
        scope.decision
    }
}

/// Result of validating one attempted use of an opaque sensitive-value handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleUseDecision {
    /// The exact scope, expiry, and use-count limits permit this use.
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
    #[must_use]
    pub fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: &str,
        expires_at_epoch_seconds: u64,
        max_uses: u32,
    ) -> Self {
        Self {
            authority: AuthorityScope::new(
                tenant_id,
                task_id,
                field_id,
                purpose_id,
                destination,
            ),
            expires_at_epoch_seconds,
            max_uses,
        }
    }
}

/// One attempted use of an opaque sensitive-value handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleUseRequest {
    authority: AuthorityScope,
    now_epoch_seconds: u64,
    uses_so_far: u32,
}

impl HandleUseRequest {
    /// Build a handle-use request using trusted time and the broker-recorded prior-use count.
    #[must_use]
    pub fn new(
        tenant_id: &str,
        task_id: &str,
        field_id: &str,
        purpose_id: &str,
        destination: &str,
        now_epoch_seconds: u64,
        uses_so_far: u32,
    ) -> Self {
        Self {
            authority: AuthorityScope::new(
                tenant_id,
                task_id,
                field_id,
                purpose_id,
                destination,
            ),
            now_epoch_seconds,
            uses_so_far,
        }
    }
}

/// Authorize one opaque-handle use without resolving or exposing its protected value.
#[must_use]
pub fn authorize_handle_use(
    request: &HandleUseRequest,
    scope: &SensitiveValueHandleScope,
) -> HandleUseDecision {
    if request.authority != scope.authority {
        HandleUseDecision::ScopeMismatch
    } else if request.now_epoch_seconds >= scope.expires_at_epoch_seconds {
        HandleUseDecision::Expired
    } else if request.uses_so_far >= scope.max_uses {
        HandleUseDecision::UseLimitReached
    } else {
        HandleUseDecision::Authorized
    }
}
