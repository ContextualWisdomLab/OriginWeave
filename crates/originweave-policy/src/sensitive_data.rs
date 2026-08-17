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
    /// The supplied exact scope, classification, expiry, and prior-use count permit broker admission.
    Authorized,
    /// Tenant, task, field, purpose, destination, or classification did not match the handle scope.
    ScopeMismatch,
    /// The exact in-process handle state was revoked before this reservation.
    Revoked,
    /// The handle is no longer valid at the supplied trusted time.
    Expired,
    /// The bounded use count has already been consumed.
    UseLimitReached,
}

/// Authority metadata attached to an opaque sensitive-value handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveValueHandleScope {
    authority: SensitiveDataAuthority,
    expires_at_epoch_seconds: u64,
    max_uses: u32,
}

impl SensitiveValueHandleScope {
    /// Build an opaque-handle scope with exact authority, exclusive expiry, and bounded use count.
    ///
    /// A later field reclassification creates a different [`SensitiveDataAuthority`]
    /// and therefore requires a newly authorized handle.
    #[must_use]
    pub const fn new(
        authority: SensitiveDataAuthority,
        expires_at_epoch_seconds: u64,
        max_uses: u32,
    ) -> Self {
        Self {
            authority,
            expires_at_epoch_seconds,
            max_uses,
        }
    }
}

/// One proposed use of an opaque sensitive-value handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleUseRequest {
    authority: SensitiveDataAuthority,
    now_epoch_seconds: u64,
    uses_so_far: u32,
}

impl HandleUseRequest {
    /// Build a handle-use evaluation request from trusted time and authoritative broker state.
    ///
    /// The eventual broker must supply these state values from its own trusted,
    /// caller-unforgeable storage; accepting this struct does not make arbitrary
    /// caller input authoritative.
    #[must_use]
    pub const fn new(
        authority: SensitiveDataAuthority,
        now_epoch_seconds: u64,
        uses_so_far: u32,
    ) -> Self {
        Self {
            authority,
            now_epoch_seconds,
            uses_so_far,
        }
    }
}

/// In-process authoritative use-count and revocation state for one opaque handle scope.
///
/// This value removes the caller-supplied prior-use count from the reservation
/// operation. A successful reservation compares the exact authority, trusted
/// time, revocation state, expiry, and current count and then increments the count
/// while the caller holds an exclusive mutable borrow of this state. Denied
/// reservations never consume a use. Revocation is idempotent and affects every
/// later exact-scope reservation through this state value.
///
/// This is a policy-state primitive, not the trusted broker itself. It contains
/// neither the opaque handle token nor protected data and provides no durable or
/// cross-process transaction, revocation reason/evidence, value resolution,
/// compensation, or persistence. A shared or durable broker must place the state
/// behind its own transactional/locking boundary and recheck lifecycle state
/// immediately before disclosure.
#[derive(Debug, PartialEq, Eq)]
pub struct SensitiveHandleUseState {
    scope: SensitiveValueHandleScope,
    reserved_uses: u32,
    revoked: bool,
}

impl SensitiveHandleUseState {
    /// Start authoritative in-process reservation state with no uses consumed.
    #[must_use]
    pub const fn new(scope: SensitiveValueHandleScope) -> Self {
        Self {
            scope,
            reserved_uses: 0,
            revoked: false,
        }
    }

    /// Return the number of uses already reserved through this state value.
    #[must_use]
    pub const fn reserved_uses(&self) -> u32 {
        self.reserved_uses
    }

    /// Return whether this in-process handle state has been revoked.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        self.revoked
    }

    /// Revoke this in-process handle state for all later reservations.
    ///
    /// Repeated calls are idempotent. This method intentionally records no reason,
    /// actor, or timestamp; a durable broker must own those lifecycle and evidence
    /// fields and persist them atomically with its authoritative handle state.
    pub const fn revoke(&mut self) {
        self.revoked = true;
    }

    /// Reserve one use from the current authoritative count when policy permits it.
    ///
    /// The supplied time must come from the trusted broker boundary. Scope mismatch
    /// is evaluated before revocation so a foreign authority cannot probe whether
    /// an otherwise matching handle state has been revoked. For an exact scope,
    /// revocation takes precedence over expiry/use-limit state and never consumes
    /// another use.
    pub fn reserve_use(
        &mut self,
        authority: SensitiveDataAuthority,
        now_epoch_seconds: u64,
    ) -> HandleUseDecision {
        let request = HandleUseRequest::new(authority, now_epoch_seconds, self.reserved_uses);
        let decision = evaluate_handle_use(&request, &self.scope);
        if decision == HandleUseDecision::ScopeMismatch {
            return decision;
        }
        if self.revoked {
            return HandleUseDecision::Revoked;
        }
        if decision == HandleUseDecision::Authorized {
            self.reserved_uses += 1;
        }
        decision
    }
}

/// Evaluate whether authoritative broker state is admissible for one handle use.
///
/// This pure function does not consume a use, inspect mutable revocation state,
/// mutate broker state, resolve a handle, or release a protected value. It is
/// therefore not standalone enforcement. A trusted broker must obtain trusted
/// time and caller-unforgeable handle state, atomically reserve or increment the
/// use count before value resolution, and recheck the reserved authority and
/// lifecycle immediately before disclosure. Missing or malformed authority
/// identifiers fail closed as a scope mismatch. The authority destination must
/// already have crossed the canonical [`Origin`] boundary.
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
