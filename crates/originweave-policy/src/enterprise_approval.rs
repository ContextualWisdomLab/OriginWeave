//! Deterministic enterprise maker-checker approval lifecycle.
//!
//! This module deliberately stores only opaque identity references and exact
//! [`ApprovalScope`] values. Authentication, wall-clock acquisition, durable
//! persistence, signatures, and external identity resolution belong to trusted
//! control-plane boundaries outside this crate.

use std::fmt;

use originweave_core::{
    ActionKind, ActionRequest, ApprovalEvidence, ApprovalScope, PolicyContext,
};

const MAX_PRINCIPAL_REFERENCE_BYTES: usize = 256;

/// An opaque, already-authenticated enterprise principal reference.
///
/// Identity is the exact `(issuer, subject)` tuple. In particular, callers must
/// not merge principals by email address or another mutable display attribute.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApprovalPrincipalRef {
    issuer: String,
    subject: String,
}

impl ApprovalPrincipalRef {
    /// Construct an opaque principal reference from trusted identity metadata.
    ///
    /// This validates only a bounded canonical representation. It does not
    /// authenticate the issuer or subject.
    pub fn new(issuer: &str, subject: &str) -> Result<Self, ApprovalPrincipalRefError> {
        if !principal_component_is_valid(issuer) {
            return Err(ApprovalPrincipalRefError::InvalidIssuer);
        }
        if !principal_component_is_valid(subject) {
            return Err(ApprovalPrincipalRefError::InvalidSubject);
        }
        Ok(Self {
            issuer: issuer.to_owned(),
            subject: subject.to_owned(),
        })
    }

    /// Return the exact trusted issuer reference.
    #[must_use]
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// Return the exact issuer-scoped subject reference.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

fn principal_component_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PRINCIPAL_REFERENCE_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

/// A validation error for an enterprise principal reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalPrincipalRefError {
    /// The issuer reference was empty, non-canonical, contained controls, or was oversized.
    InvalidIssuer,
    /// The subject reference was empty, non-canonical, contained controls, or was oversized.
    InvalidSubject,
}

impl fmt::Display for ApprovalPrincipalRefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIssuer => formatter.write_str("approval principal issuer is invalid"),
            Self::InvalidSubject => formatter.write_str("approval principal subject is invalid"),
        }
    }
}

impl std::error::Error for ApprovalPrincipalRefError {}

/// The fail-closed state of one bounded enterprise approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalLifecycleState {
    /// A maker requested approval and no checker decision exists yet.
    ApprovalRequested,
    /// A distinct checker approved the exact immutable scope.
    Approved,
    /// A distinct checker denied the request.
    Denied,
    /// The trusted validity deadline was reached before a permitted transition.
    Expired,
    /// The requesting maker withdrew the pending request.
    Withdrawn,
    /// Every configured bounded use of the approval has been consumed.
    Consumed,
    /// The approving checker revoked an approved, not-yet-exhausted request.
    Revoked,
}

/// One consumed, non-replayable enterprise approval use.
///
/// This value is intentionally not [`Clone`]. It is created only by
/// [`EnterpriseApprovalRequest::consume`] after exact-scope, trusted-time, and
/// use-count checks succeed. [`Self::evaluate`] consumes the value, injects the
/// approved scope into a private copy of the supplied policy context, and then
/// delegates to the normal fail-closed policy evaluator. The use is burned even
/// when policy evaluation denies the action or requires a different approval.
///
/// ```compile_fail
/// # use originweave_core::{ActionRequest, PolicyContext};
/// # use originweave_policy::EnterpriseApprovalUse;
/// # fn replay_is_rejected(
/// #     approval_use: EnterpriseApprovalUse,
/// #     request: &ActionRequest,
/// #     context: &PolicyContext,
/// # ) {
/// let _ = approval_use.evaluate(request, context);
/// let _ = approval_use.evaluate(request, context);
/// # }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct EnterpriseApprovalUse {
    scope: ApprovalScope,
}

impl EnterpriseApprovalUse {
    /// Evaluate exactly one action using this already-consumed approval use.
    ///
    /// The caller-provided context is cloned so the reusable caller context is
    /// never upgraded with replayable approval evidence. This value itself is
    /// consumed regardless of the resulting decision.
    #[must_use]
    pub fn evaluate(self, request: &ActionRequest, context: &PolicyContext) -> crate::Decision {
        let mut one_shot_context = context.clone();
        one_shot_context.set_approval(ApprovalEvidence::UserConfirmed(self.scope));
        crate::evaluate(request, &one_shot_context)
    }
}

/// A deterministic enterprise approval request bound to one immutable action intent.
///
/// The caller supplies trusted control-plane epoch seconds to transition methods.
/// Model output, page content, or another untrusted source must never supply that
/// time value. This type performs no I/O and does not persist or authenticate data.
#[derive(Debug, PartialEq, Eq)]
pub struct EnterpriseApprovalRequest {
    scope: ApprovalScope,
    requester: ApprovalPrincipalRef,
    decision_actor: Option<ApprovalPrincipalRef>,
    requested_at_epoch_seconds: u64,
    expires_at_epoch_seconds: u64,
    last_transition_at_epoch_seconds: u64,
    max_uses: u32,
    uses_consumed: u32,
    state: ApprovalLifecycleState,
}

impl EnterpriseApprovalRequest {
    /// Create one pending request for an exact scope and bounded validity/use window.
    ///
    /// `requested_at_epoch_seconds` and `expires_at_epoch_seconds` must come from
    /// the same trusted control-plane clock. Legal consent is intentionally
    /// non-delegable and cannot enter this approval lifecycle.
    pub fn new(
        scope: ApprovalScope,
        requester: ApprovalPrincipalRef,
        requested_at_epoch_seconds: u64,
        expires_at_epoch_seconds: u64,
        max_uses: u32,
    ) -> Result<Self, ApprovalLifecycleError> {
        if expires_at_epoch_seconds <= requested_at_epoch_seconds {
            return Err(ApprovalLifecycleError::InvalidValidityWindow);
        }
        if max_uses == 0 {
            return Err(ApprovalLifecycleError::InvalidUseLimit);
        }
        if scope.action() == ActionKind::LegalConsent {
            return Err(ApprovalLifecycleError::NonDelegableAction);
        }
        Ok(Self {
            scope,
            requester,
            decision_actor: None,
            requested_at_epoch_seconds,
            expires_at_epoch_seconds,
            last_transition_at_epoch_seconds: requested_at_epoch_seconds,
            max_uses,
            uses_consumed: 0,
            state: ApprovalLifecycleState::ApprovalRequested,
        })
    }

    /// Return the exact action/origin/intent scope covered by the request.
    #[must_use]
    pub const fn scope(&self) -> &ApprovalScope {
        &self.scope
    }

    /// Return the maker that created the request.
    #[must_use]
    pub const fn requester(&self) -> &ApprovalPrincipalRef {
        &self.requester
    }

    /// Return the checker that approved or denied the request, when present.
    #[must_use]
    pub const fn decision_actor(&self) -> Option<&ApprovalPrincipalRef> {
        self.decision_actor.as_ref()
    }

    /// Return the trusted request creation time in Unix epoch seconds.
    #[must_use]
    pub const fn requested_at_epoch_seconds(&self) -> u64 {
        self.requested_at_epoch_seconds
    }

    /// Return the exclusive trusted expiry deadline in Unix epoch seconds.
    #[must_use]
    pub const fn expires_at_epoch_seconds(&self) -> u64 {
        self.expires_at_epoch_seconds
    }

    /// Return the maximum number of exact-scope consumptions permitted.
    #[must_use]
    pub const fn max_uses(&self) -> u32 {
        self.max_uses
    }

    /// Return how many exact-scope consumptions have already occurred.
    #[must_use]
    pub const fn uses_consumed(&self) -> u32 {
        self.uses_consumed
    }

    /// Return the current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> ApprovalLifecycleState {
        self.state
    }

    fn ensure_monotonic_transition_time(
        &self,
        now_epoch_seconds: u64,
    ) -> Result<(), ApprovalLifecycleError> {
        if now_epoch_seconds < self.last_transition_at_epoch_seconds {
            return Err(ApprovalLifecycleError::NonMonotonicTime);
        }
        Ok(())
    }

    /// Approve a pending request as a distinct checker.
    ///
    /// `now_epoch_seconds` must be trusted control-plane time. Expiry is
    /// exclusive: a transition at the deadline fails closed.
    pub fn approve(
        &mut self,
        approver: ApprovalPrincipalRef,
        now_epoch_seconds: u64,
    ) -> Result<(), ApprovalLifecycleError> {
        if self.state != ApprovalLifecycleState::ApprovalRequested {
            return Err(ApprovalLifecycleError::InvalidState(self.state));
        }
        self.ensure_monotonic_transition_time(now_epoch_seconds)?;
        if now_epoch_seconds >= self.expires_at_epoch_seconds {
            self.last_transition_at_epoch_seconds = now_epoch_seconds;
            self.state = ApprovalLifecycleState::Expired;
            return Err(ApprovalLifecycleError::Expired);
        }
        if approver == self.requester {
            return Err(ApprovalLifecycleError::SelfApproval);
        }
        self.decision_actor = Some(approver);
        self.last_transition_at_epoch_seconds = now_epoch_seconds;
        self.state = ApprovalLifecycleState::Approved;
        Ok(())
    }

    /// Deny a pending request as a distinct checker.
    ///
    /// `now_epoch_seconds` must be trusted control-plane time.
    pub fn deny(
        &mut self,
        actor: ApprovalPrincipalRef,
        now_epoch_seconds: u64,
    ) -> Result<(), ApprovalLifecycleError> {
        if self.state != ApprovalLifecycleState::ApprovalRequested {
            return Err(ApprovalLifecycleError::InvalidState(self.state));
        }
        self.ensure_monotonic_transition_time(now_epoch_seconds)?;
        if now_epoch_seconds >= self.expires_at_epoch_seconds {
            self.last_transition_at_epoch_seconds = now_epoch_seconds;
            self.state = ApprovalLifecycleState::Expired;
            return Err(ApprovalLifecycleError::Expired);
        }
        if actor == self.requester {
            return Err(ApprovalLifecycleError::SelfApproval);
        }
        self.decision_actor = Some(actor);
        self.last_transition_at_epoch_seconds = now_epoch_seconds;
        self.state = ApprovalLifecycleState::Denied;
        Ok(())
    }

    /// Withdraw a pending request as the exact requesting maker.
    ///
    /// `now_epoch_seconds` must be trusted control-plane time.
    pub fn withdraw(
        &mut self,
        actor: &ApprovalPrincipalRef,
        now_epoch_seconds: u64,
    ) -> Result<(), ApprovalLifecycleError> {
        if self.state != ApprovalLifecycleState::ApprovalRequested {
            return Err(ApprovalLifecycleError::InvalidState(self.state));
        }
        self.ensure_monotonic_transition_time(now_epoch_seconds)?;
        if now_epoch_seconds >= self.expires_at_epoch_seconds {
            self.last_transition_at_epoch_seconds = now_epoch_seconds;
            self.state = ApprovalLifecycleState::Expired;
            return Err(ApprovalLifecycleError::Expired);
        }
        if actor != &self.requester {
            return Err(ApprovalLifecycleError::RequesterMismatch);
        }
        self.last_transition_at_epoch_seconds = now_epoch_seconds;
        self.state = ApprovalLifecycleState::Withdrawn;
        Ok(())
    }

    /// Consume one use of an approved request for the exact immutable scope.
    ///
    /// `now_epoch_seconds` must be trusted control-plane time. Scope mismatch
    /// does not consume a use. Successful consumption returns a non-cloneable
    /// [`EnterpriseApprovalUse`] rather than replayable approval evidence.
    pub fn consume(
        &mut self,
        required_scope: &ApprovalScope,
        now_epoch_seconds: u64,
    ) -> Result<EnterpriseApprovalUse, ApprovalLifecycleError> {
        if self.state != ApprovalLifecycleState::Approved {
            return Err(ApprovalLifecycleError::InvalidState(self.state));
        }
        self.ensure_monotonic_transition_time(now_epoch_seconds)?;
        if now_epoch_seconds >= self.expires_at_epoch_seconds {
            self.last_transition_at_epoch_seconds = now_epoch_seconds;
            self.state = ApprovalLifecycleState::Expired;
            return Err(ApprovalLifecycleError::Expired);
        }
        if required_scope != &self.scope {
            return Err(ApprovalLifecycleError::ScopeMismatch);
        }
        self.uses_consumed += 1;
        self.last_transition_at_epoch_seconds = now_epoch_seconds;
        if self.uses_consumed == self.max_uses {
            self.state = ApprovalLifecycleState::Consumed;
        }
        Ok(EnterpriseApprovalUse {
            scope: self.scope.clone(),
        })
    }

    /// Revoke an approved request as the exact checker that approved it.
    ///
    /// `now_epoch_seconds` must be trusted control-plane time.
    pub fn revoke(
        &mut self,
        actor: &ApprovalPrincipalRef,
        now_epoch_seconds: u64,
    ) -> Result<(), ApprovalLifecycleError> {
        if self.state != ApprovalLifecycleState::Approved {
            return Err(ApprovalLifecycleError::InvalidState(self.state));
        }
        self.ensure_monotonic_transition_time(now_epoch_seconds)?;
        if now_epoch_seconds >= self.expires_at_epoch_seconds {
            self.last_transition_at_epoch_seconds = now_epoch_seconds;
            self.state = ApprovalLifecycleState::Expired;
            return Err(ApprovalLifecycleError::Expired);
        }
        if self.decision_actor.as_ref() != Some(actor) {
            return Err(ApprovalLifecycleError::DecisionActorMismatch);
        }
        self.last_transition_at_epoch_seconds = now_epoch_seconds;
        self.state = ApprovalLifecycleState::Revoked;
        Ok(())
    }
}

/// A fail-closed error produced by an enterprise approval lifecycle transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalLifecycleError {
    /// The expiry deadline was not strictly later than the request time.
    InvalidValidityWindow,
    /// The configured maximum number of uses was zero.
    InvalidUseLimit,
    /// The requested action is intentionally non-delegable.
    NonDelegableAction,
    /// The requested transition is not valid from the current terminal or pending state.
    InvalidState(ApprovalLifecycleState),
    /// Trusted transition time moved backward relative to the last accepted lifecycle event.
    NonMonotonicTime,
    /// The requester attempted to act as their own checker.
    SelfApproval,
    /// A withdrawal actor did not match the original requester.
    RequesterMismatch,
    /// A revocation actor did not match the checker that approved the request.
    DecisionActorMismatch,
    /// The requested action/origin/intent scope did not exactly match the approval.
    ScopeMismatch,
    /// The trusted exclusive expiry deadline was reached.
    Expired,
}

impl fmt::Display for ApprovalLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValidityWindow => {
                formatter.write_str("approval validity window is invalid")
            }
            Self::InvalidUseLimit => formatter.write_str("approval use limit must be nonzero"),
            Self::NonDelegableAction => formatter.write_str("action is not delegable by approval"),
            Self::InvalidState(state) => write!(
                formatter,
                "approval transition is invalid from state {state:?}"
            ),
            Self::NonMonotonicTime => {
                formatter.write_str("approval transition time moved backward")
            }
            Self::SelfApproval => formatter.write_str("maker and checker must be distinct"),
            Self::RequesterMismatch => formatter.write_str("approval requester does not match"),
            Self::DecisionActorMismatch => {
                formatter.write_str("approval decision actor does not match")
            }
            Self::ScopeMismatch => formatter.write_str("approval scope does not match"),
            Self::Expired => formatter.write_str("approval request has expired"),
        }
    }
}

impl std::error::Error for ApprovalLifecycleError {}
