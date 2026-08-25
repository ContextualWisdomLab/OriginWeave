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
    /// The supplied exact authority, audience, expiry, and prior-use count permit broker admission.
    Authorized,
    /// The authoritative in-process handle state was revoked before this use.
    Revoked,
    /// Tenant, task, field, purpose, destination, or classification did not match the handle scope.
    ScopeMismatch,
    /// The caller audience was invalid or did not match the handle's non-transferable audience.
    AudienceMismatch,
    /// The handle is no longer valid at the supplied trusted time.
    Expired,
    /// The supplied trusted time predates time already observed by the reservation state.
    TrustedTimeRollback,
    /// The bounded use count has already been consumed.
    UseLimitReached,
}

/// Reason that authoritative in-process handle state was revoked.
///
/// The reason is credential-free policy metadata. The first successful
/// revocation is retained so a later duplicate transition cannot rewrite the
/// original lifecycle cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleRevocationReason {
    /// The delegated task completed and no further disclosure is permitted.
    TaskCompleted,
    /// A relevant authorization or disclosure policy changed.
    PolicyChanged,
    /// Key rotation invalidated the handle lifecycle controlled by the broker.
    KeyRotated,
    /// The task or browser session terminated.
    SessionTerminated,
    /// Security monitoring identified suspicious handle use.
    SuspiciousUse,
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
    /// The audience identifier uses the same bounded ASCII policy-token grammar as
    /// other authority identifiers. Invalid audience identifiers remain fail-closed
    /// when the scope is evaluated. A later field reclassification or audience
    /// change therefore requires a newly authorized handle.
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
    /// Build a handle-use evaluation request from exact authority, caller audience,
    /// trusted time, and authoritative broker use state.
    ///
    /// The eventual broker must derive `audience_id` from authenticated service or
    /// workload identity and supply the state values from caller-unforgeable
    /// storage. Accepting this value object does not make arbitrary caller input
    /// authoritative.
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

/// In-process authoritative use-count, trusted-time, and revocation state for one opaque sensitive-value handle scope.
///
/// This value removes the caller-supplied prior-use count from the reservation
/// operation. A successful reservation compares the exact authority, exact
/// non-transferable audience, trusted time, expiry, revocation state, and current
/// count and then increments the count while the caller holds an exclusive mutable
/// borrow of this state. The state remembers the latest trusted time observed for
/// its exact authority-and-audience binding and rejects time rollback so an expired
/// handle cannot regain authority from a stale clock value. Binding-mismatched
/// requests cannot read or mutate that floor. Denied reservations never consume a
/// use.
///
/// This is a policy-state primitive, not the trusted broker itself. It contains
/// neither the opaque handle token nor protected data and provides no authenticated
/// workload identity, durable or cross-process transaction, value resolution,
/// compensation, or persistence. A shared or durable broker must derive the
/// audience from authenticated caller identity, place the state behind its own
/// transactional or locking boundary, persist lifecycle state and an equivalent
/// trusted-time floor, and recheck both before disclosure.
#[derive(Debug, PartialEq, Eq)]
pub struct SensitiveHandleUseState {
    scope: SensitiveValueHandleScope,
    reserved_uses: u32,
    revocation_reason: Option<HandleRevocationReason>,
    latest_trusted_time_epoch_seconds: Option<u64>,
}

impl SensitiveHandleUseState {
    /// Start authoritative in-process reservation state with no uses consumed or revocation recorded.
    #[must_use]
    pub const fn new(scope: SensitiveValueHandleScope) -> Self {
        Self {
            scope,
            reserved_uses: 0,
            revocation_reason: None,
            latest_trusted_time_epoch_seconds: None,
        }
    }

    /// Return the number of uses already reserved through this state value.
    #[must_use]
    pub const fn reserved_uses(&self) -> u32 {
        self.reserved_uses
    }

    /// Return the first authoritative revocation reason, if this state was revoked.
    #[must_use]
    pub const fn revocation_reason(&self) -> Option<HandleRevocationReason> {
        self.revocation_reason
    }

    /// Revoke future reservations and retain the first lifecycle reason.
    ///
    /// Returns `true` only for the state transition from active to revoked. A
    /// later duplicate call is a no-op and cannot rewrite the original reason.
    pub fn revoke(&mut self, reason: HandleRevocationReason) -> bool {
        if self.revocation_reason.is_some() {
            false
        } else {
            self.revocation_reason = Some(reason);
            true
        }
    }

    /// Reserve one use from the current authoritative count when policy permits it.
    ///
    /// The audience must be derived by the trusted broker from authenticated caller
    /// identity, and the supplied time must come from the broker's trusted clock and
    /// may not move backward relative to an earlier exact-binding reservation
    /// attempt on this state. Exact authority and audience binding are evaluated
    /// before lifecycle state; a caller outside either binding receives only
    /// `ScopeMismatch` or `AudienceMismatch` and cannot read or advance revocation
    /// or trusted-time state. For a correctly bound caller, revocation remains
    /// terminal and precedes rollback, expiry, and use-limit results. A non-rollback
    /// trusted time is recorded for downstream nonterminal policy denials, while
    /// every denial leaves the authoritative count unchanged.
    #[must_use]
    pub fn reserve_use(
        &mut self,
        authority: SensitiveDataAuthority,
        audience_id: &str,
        now_epoch_seconds: u64,
    ) -> HandleUseDecision {
        let request = HandleUseRequest::new(
            authority,
            audience_id,
            now_epoch_seconds,
            self.reserved_uses,
        );
        let policy_decision = evaluate_handle_use(&request, &self.scope);
        if matches!(
            policy_decision,
            HandleUseDecision::ScopeMismatch | HandleUseDecision::AudienceMismatch
        ) {
            return policy_decision;
        }

        if self.revocation_reason.is_some() {
            return HandleUseDecision::Revoked;
        }
        if self
            .latest_trusted_time_epoch_seconds
            .is_some_and(|latest| now_epoch_seconds < latest)
        {
            return HandleUseDecision::TrustedTimeRollback;
        }
        self.latest_trusted_time_epoch_seconds = Some(now_epoch_seconds);

        if policy_decision == HandleUseDecision::Authorized {
            self.reserved_uses += 1;
        }
        policy_decision
    }
}

/// Evaluate whether authoritative broker state is admissible for one handle use.
///
/// This pure function does not consume a use, mutate broker state, resolve a
/// handle, or release a protected value. It is therefore not standalone
/// enforcement and cannot detect trusted-time rollback across calls. A trusted
/// broker must obtain authenticated caller audience, trusted time, and
/// caller-unforgeable handle state, reject time rollback through state such as
/// [`SensitiveHandleUseState`], atomically reserve or increment the use count
/// before value resolution, and recheck the reserved authority immediately before
/// disclosure. Missing or malformed authority or audience identifiers fail closed.
/// The authority destination must already have crossed the canonical [`Origin`]
/// boundary.
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
