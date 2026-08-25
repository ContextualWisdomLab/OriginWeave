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

/// Exact authority metadata for one classified sensitive-data field use.
///
/// The value contains no protected field bytes. It combines the tenant, task,
/// field, business purpose, canonical destination, and data classification so
/// disclosure, opaque-handle issuance, and opaque-handle use cannot silently
/// diverge on one of those authority dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDataAuthority {
    tenant_id: String,
    task_id: String,
    field_id: String,
    purpose_id: String,
    destination: Origin,
    classification: DataClassification,
}

impl SensitiveDataAuthority {
    /// Build one exact classified authority value without carrying protected data.
    ///
    /// Tenant, task, field, and purpose identifiers are admitted only as 1–128
    /// byte ASCII policy tokens using alphanumeric characters plus `.`, `_`, `:`,
    /// and `-`. Each token must contain at least one alphanumeric character.
    /// Invalid identifiers remain fail-closed when the authority is used.
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
            tenant_id: tenant_id.to_owned(),
            task_id: task_id.to_owned(),
            field_id: field_id.to_owned(),
            purpose_id: purpose_id.to_owned(),
            destination,
            classification,
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
        && identifier.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

/// One requested disclosure, without carrying the protected field value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveDataRequest {
    authority: SensitiveDataAuthority,
}

impl SensitiveDataRequest {
    /// Build a disclosure request from one exact classified authority value.
    #[must_use]
    pub const fn new(authority: SensitiveDataAuthority) -> Self {
        Self { authority }
    }
}

/// Explicit authority for one requested sensitive-data disclosure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisclosureScope {
    authority: SensitiveDataAuthority,
    decision: DisclosureDecision,
}

impl DisclosureScope {
    /// Build an exact disclosure authority scope and its maximum permitted outcome.
    #[must_use]
    pub const fn new(authority: SensitiveDataAuthority, decision: DisclosureDecision) -> Self {
        Self {
            authority,
            decision,
        }
    }
}

/// Evaluate disclosure only from the exact request and explicit authority scope.
///
/// An incomplete or malformed authority fails closed even when both sides contain
/// the same invalid identifier.
#[must_use]
pub fn evaluate_disclosure(
    request: &SensitiveDataRequest,
    scope: &DisclosureScope,
) -> DisclosureDecision {
    if !request.authority.is_complete()
        || !scope.authority.is_complete()
        || request.authority != scope.authority
    {
        DisclosureDecision::DenyAccess
    } else {
        scope.decision
    }
}

/// Result of evaluating one attempted use of an opaque sensitive-value handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleUseDecision {
    /// Exact scope, audience, expiry, and prior-use count permit broker admission.
    Authorized,
    /// Tenant, task, field, purpose, destination, or classification did not match the handle scope.
    ScopeMismatch,
    /// Authenticated workload or adapter audience did not match the handle scope.
    AudienceMismatch,
    /// The handle is no longer valid at the supplied trusted time.
    Expired,
    /// The supplied trusted time predates time already observed by the reservation state.
    TrustedTimeRollback,
    /// The bounded use count has already been consumed.
    UseLimitReached,
}

/// Authority metadata attached to an opaque sensitive-value handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveValueHandleScope {
    authority: SensitiveDataAuthority,
    audience_id: String,
    expires_at_epoch_seconds: u64,
    max_uses: u32,
}

impl SensitiveValueHandleScope {
    /// Build an opaque-handle scope with exact authority, non-transferable audience,
    /// exclusive expiry, and bounded use count.
    ///
    /// `audience_id` identifies the authenticated workload or trusted adapter that
    /// may present this handle. It uses the same bounded ASCII policy-token grammar
    /// as other authority identifiers and is checked fail-closed at use time. A
    /// later field reclassification or audience change therefore requires a newly
    /// authorized handle.
    #[must_use]
    pub fn new(
        authority: SensitiveDataAuthority,
        audience_id: &str,
        expires_at_epoch_seconds: u64,
        max_uses: u32,
    ) -> Self {
        Self {
            authority,
            audience_id: audience_id.to_owned(),
            expires_at_epoch_seconds,
            max_uses,
        }
    }
}

/// One proposed use of an opaque sensitive-value handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleUseRequest {
    authority: SensitiveDataAuthority,
    audience_id: String,
    now_epoch_seconds: u64,
    uses_so_far: u32,
}

impl HandleUseRequest {
    /// Build a handle-use evaluation request from authenticated audience, trusted
    /// time, and authoritative broker state.
    ///
    /// The eventual broker must supply the audience from its authenticated
    /// workload/service-identity boundary and the state values from caller-
    /// unforgeable storage. Accepting this struct does not make arbitrary caller
    /// strings or counters authoritative.
    #[must_use]
    pub fn new(
        authority: SensitiveDataAuthority,
        audience_id: &str,
        now_epoch_seconds: u64,
        uses_so_far: u32,
    ) -> Self {
        Self {
            authority,
            audience_id: audience_id.to_owned(),
            now_epoch_seconds,
            uses_so_far,
        }
    }
}

/// In-process authoritative use-count state for one opaque sensitive-value handle scope.
///
/// This value removes the caller-supplied prior-use count from the reservation
/// operation. A successful reservation compares the exact authority, authenticated
/// audience, trusted time, expiry, and current count and then increments the count
/// while the caller holds an exclusive mutable borrow of this state. The state also
/// remembers the latest trusted time it has observed and rejects time rollback so an
/// expired handle cannot regain authority from a later stale clock value. Denied
/// reservations never consume a use.
///
/// This is a policy-state primitive, not the trusted broker itself. It contains
/// neither the opaque handle token nor protected data and provides no durable or
/// cross-process transaction, identity authentication, revocation, value resolution,
/// compensation, or persistence. A shared or durable broker must place the state
/// behind its own transactional/locking boundary, obtain `audience_id` from an
/// authenticated workload/service-identity boundary, persist an equivalent trusted-
/// time floor, and recheck lifecycle state before disclosure.
#[derive(Debug, PartialEq, Eq)]
pub struct SensitiveHandleUseState {
    scope: SensitiveValueHandleScope,
    reserved_uses: u32,
    latest_trusted_time_epoch_seconds: Option<u64>,
}

impl SensitiveHandleUseState {
    /// Start authoritative in-process reservation state with no uses consumed.
    #[must_use]
    pub const fn new(scope: SensitiveValueHandleScope) -> Self {
        Self {
            scope,
            reserved_uses: 0,
            latest_trusted_time_epoch_seconds: None,
        }
    }

    /// Return the number of uses already reserved through this state value.
    #[must_use]
    pub const fn reserved_uses(&self) -> u32 {
        self.reserved_uses
    }

    /// Reserve one use from the current authoritative count when policy permits it.
    ///
    /// The supplied audience must be derived from the authenticated broker boundary;
    /// the supplied time must come from trusted time and may not move backward
    /// relative to an earlier reservation attempt on this state. Exact-scope,
    /// audience, expiry, use-limit, and trusted-time rollback denial leave the
    /// authoritative use count unchanged. A non-rollback trusted time is recorded
    /// even when another policy check denies the reservation so later stale time
    /// cannot restore authority.
    #[must_use]
    pub fn reserve_use(
        &mut self,
        authority: SensitiveDataAuthority,
        audience_id: &str,
        now_epoch_seconds: u64,
    ) -> HandleUseDecision {
        if self
            .latest_trusted_time_epoch_seconds
            .is_some_and(|latest| now_epoch_seconds < latest)
        {
            return HandleUseDecision::TrustedTimeRollback;
        }
        self.latest_trusted_time_epoch_seconds = Some(now_epoch_seconds);

        let request = HandleUseRequest::new(
            authority,
            audience_id,
            now_epoch_seconds,
            self.reserved_uses,
        );
        let decision = evaluate_handle_use(&request, &self.scope);
        if decision == HandleUseDecision::Authorized {
            self.reserved_uses += 1;
        }
        decision
    }
}

/// Evaluate whether authoritative broker state is admissible for one handle use.
///
/// This pure function does not consume a use, mutate broker state, authenticate an
/// audience, resolve a handle, or release a protected value. It is therefore not
/// standalone enforcement and cannot detect trusted-time rollback across calls. A
/// trusted broker must authenticate service/workload identity, map it to the
/// request audience, obtain trusted time and caller-unforgeable handle state, reject
/// time rollback through state such as [`SensitiveHandleUseState`], atomically
/// reserve or increment the use count before value resolution, and recheck the
/// reserved authority immediately before disclosure. Missing or malformed authority
/// or audience identifiers fail closed. The authority destination must already have
/// crossed the canonical [`Origin`] boundary.
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
    } else if !authority_identifier_is_valid(&request.audience_id)
        || !authority_identifier_is_valid(&scope.audience_id)
        || request.audience_id != scope.audience_id
    {
        HandleUseDecision::AudienceMismatch
    } else if request.now_epoch_seconds >= scope.expires_at_epoch_seconds {
        HandleUseDecision::Expired
    } else if request.uses_so_far >= scope.max_uses {
        HandleUseDecision::UseLimitReached
    } else {
        HandleUseDecision::Authorized
    }
}
