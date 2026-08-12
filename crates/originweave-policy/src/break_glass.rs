//! Fail-closed composition for exceptional sensitive-data break-glass access.
//!
//! Break-glass is not a reusable role or an alternative disclosure authority. This module permits
//! only an existing human-approval or dual-control sensitive-data decision to proceed after exact
//! authority, actor, approver, and reason binding, a locally bounded half-open validity window,
//! sufficient approval evidence, heightened monitoring, and mandatory post-event review are all
//! explicit. The types carry policy metadata only; they never carry protected values, authenticate
//! identities, read a clock, execute monitoring, persist evidence, or perform a review.

use crate::sensitive_data::{
    DisclosureDecision, DisclosureScope, SensitiveDataAuthority, SensitiveDataRequest,
    evaluate_disclosure,
};

const MAX_BREAK_GLASS_IDENTIFIER_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
enum BreakGlassApprovalKind {
    None,
    Human(String),
    DualControl {
        first_approval_id: String,
        second_approval_id: String,
    },
}

/// Credential-free approval references presented for one break-glass decision.
///
/// The identifiers are bounded policy metadata, not signatures or self-authenticating identities.
/// A trusted approval service must derive them from current authorized approvers and preserve the
/// underlying approval evidence outside this pure policy value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakGlassApprovalEvidence {
    kind: BreakGlassApprovalKind,
}

impl BreakGlassApprovalEvidence {
    /// Build evidence that contains no approval reference.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            kind: BreakGlassApprovalKind::None,
        }
    }

    /// Build one human-approval reference.
    ///
    /// Invalid identifiers remain fail-closed when the enclosing scope is evaluated.
    #[must_use]
    pub fn human(approval_id: &str) -> Self {
        Self {
            kind: BreakGlassApprovalKind::Human(approval_id.to_owned()),
        }
    }

    /// Build two independent approval references for dual control.
    ///
    /// Both identifiers must be valid and distinct. This constructor does not authenticate either
    /// approver or prove that the identities are organizationally independent.
    #[must_use]
    pub fn dual_control(first_approval_id: &str, second_approval_id: &str) -> Self {
        Self {
            kind: BreakGlassApprovalKind::DualControl {
                first_approval_id: first_approval_id.to_owned(),
                second_approval_id: second_approval_id.to_owned(),
            },
        }
    }

    fn is_valid(&self) -> bool {
        match &self.kind {
            BreakGlassApprovalKind::None => true,
            BreakGlassApprovalKind::Human(approval_id) => {
                break_glass_identifier_is_valid(approval_id)
            }
            BreakGlassApprovalKind::DualControl {
                first_approval_id,
                second_approval_id,
            } => {
                break_glass_identifier_is_valid(first_approval_id)
                    && break_glass_identifier_is_valid(second_approval_id)
                    && first_approval_id != second_approval_id
            }
        }
    }

    fn satisfies(&self, requirement: BreakGlassApprovalRequirement) -> bool {
        match requirement {
            BreakGlassApprovalRequirement::Human => matches!(
                &self.kind,
                BreakGlassApprovalKind::Human(_) | BreakGlassApprovalKind::DualControl { .. }
            ),
            BreakGlassApprovalRequirement::DualControl => {
                matches!(&self.kind, BreakGlassApprovalKind::DualControl { .. })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BreakGlassApprovalRequirement {
    Human,
    DualControl,
}

/// Caller-supplied actor identity binding for one break-glass evaluation.
///
/// `current_actor_id` identifies the currently authenticated actor according to the trusted caller;
/// `approved_actor_id` identifies the actor covered by the exceptional approval. Both values use a
/// bounded credential-free identifier grammar and must match exactly. Constructing this value does
/// not authenticate either identity or prove that the caller derived them from a trusted source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakGlassActorBinding {
    current_actor_id: String,
    approved_actor_id: String,
}

impl BreakGlassActorBinding {
    /// Build one exact current-actor to approved-actor binding.
    #[must_use]
    pub fn new(current_actor_id: &str, approved_actor_id: &str) -> Self {
        Self {
            current_actor_id: current_actor_id.to_owned(),
            approved_actor_id: approved_actor_id.to_owned(),
        }
    }

    fn is_valid(&self) -> bool {
        break_glass_identifier_is_valid(&self.current_actor_id)
            && break_glass_identifier_is_valid(&self.approved_actor_id)
    }

    fn matches(&self) -> bool {
        self.current_actor_id == self.approved_actor_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BreakGlassApproverKind {
    Human(String),
    DualControl {
        first_approver_id: String,
        second_approver_id: String,
    },
}

/// Caller-supplied approver identities bound to break-glass approval references.
///
/// These identifiers are credential-free policy metadata. They must be derived by a trusted approval
/// service from the same authoritative approval records represented by [`BreakGlassApprovalEvidence`].
/// Exact identity inequality prevents the approved beneficiary from approving their own break-glass
/// access and prevents one identity from satisfying both sides of dual control. This value does not
/// authenticate an approver, verify a signature, or prove organizational-role separation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakGlassApproverBinding {
    kind: BreakGlassApproverKind,
}

impl BreakGlassApproverBinding {
    /// Bind one human approver identity to human approval evidence.
    #[must_use]
    pub fn human(approver_id: &str) -> Self {
        Self {
            kind: BreakGlassApproverKind::Human(approver_id.to_owned()),
        }
    }

    /// Bind two distinct approver identities to dual-control approval evidence.
    #[must_use]
    pub fn dual_control(first_approver_id: &str, second_approver_id: &str) -> Self {
        Self {
            kind: BreakGlassApproverKind::DualControl {
                first_approver_id: first_approver_id.to_owned(),
                second_approver_id: second_approver_id.to_owned(),
            },
        }
    }

    fn is_valid(&self) -> bool {
        match &self.kind {
            BreakGlassApproverKind::Human(approver_id) => {
                break_glass_identifier_is_valid(approver_id)
            }
            BreakGlassApproverKind::DualControl {
                first_approver_id,
                second_approver_id,
            } => {
                break_glass_identifier_is_valid(first_approver_id)
                    && break_glass_identifier_is_valid(second_approver_id)
                    && first_approver_id != second_approver_id
            }
        }
    }

    fn matches_approval(&self, approval: &BreakGlassApprovalEvidence) -> bool {
        matches!(
            (&self.kind, &approval.kind),
            (
                BreakGlassApproverKind::Human(_),
                BreakGlassApprovalKind::Human(_)
            ) | (
                BreakGlassApproverKind::DualControl { .. },
                BreakGlassApprovalKind::DualControl { .. }
            )
        )
    }

    fn is_independent_from(&self, actor_binding: &BreakGlassActorBinding) -> bool {
        match &self.kind {
            BreakGlassApproverKind::Human(approver_id) => {
                approver_id != &actor_binding.current_actor_id
            }
            BreakGlassApproverKind::DualControl {
                first_approver_id,
                second_approver_id,
            } => {
                first_approver_id != &actor_binding.current_actor_id
                    && second_approver_id != &actor_binding.current_actor_id
            }
        }
    }
}

/// Local policy ceiling for one break-glass validity interval.
///
/// The maximum uses the same trusted time units as the scope and evaluation time. A zero maximum is
/// invalid. This value is explicit policy input; it does not select a time unit, read a clock, attest
/// clock provenance, or prove that an arbitrary caller supplied the reviewed local ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakGlassValidityPolicy {
    maximum_window: u64,
}

impl BreakGlassValidityPolicy {
    /// Build a local maximum accepted break-glass validity window.
    #[must_use]
    pub const fn new(maximum_window: u64) -> Self {
        Self { maximum_window }
    }

    fn is_valid(self) -> bool {
        self.maximum_window > 0
    }
}

/// One requested exceptional disclosure without protected field bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveBreakGlassRequest {
    authority: SensitiveDataAuthority,
    reason_id: String,
}

impl SensitiveBreakGlassRequest {
    /// Build one break-glass request bound to exact sensitive-data authority and reason metadata.
    ///
    /// The reason identifier uses a bounded ASCII policy-token grammar. It should refer to a durable
    /// incident, support, legal, or emergency record owned by a trusted system; this value does not
    /// prove that the referenced reason exists or is truthful.
    #[must_use]
    pub fn new(authority: SensitiveDataAuthority, reason_id: &str) -> Self {
        Self {
            authority,
            reason_id: reason_id.to_owned(),
        }
    }

    fn is_valid(&self) -> bool {
        break_glass_identifier_is_valid(&self.reason_id)
    }

    fn matches_disclosure_request(&self, request: &SensitiveDataRequest) -> bool {
        SensitiveDataRequest::new(self.authority.clone()).eq(request)
    }
}

/// Trusted policy scope for one bounded break-glass decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveBreakGlassScope {
    authority: SensitiveDataAuthority,
    reason_id: String,
    approval: BreakGlassApprovalEvidence,
    valid_from: u64,
    valid_until: u64,
    heightened_monitoring: bool,
    post_event_review: bool,
}

impl SensitiveBreakGlassScope {
    /// Build exact exceptional-access policy metadata.
    ///
    /// `valid_from` and `valid_until` belong to one caller-supplied trusted time domain and form a
    /// half-open interval. The two boolean controls are trusted policy facts supplied by the runtime;
    /// setting them does not execute monitoring or schedule a post-event review.
    #[must_use]
    pub fn new(
        authority: SensitiveDataAuthority,
        reason_id: &str,
        approval: BreakGlassApprovalEvidence,
        valid_from: u64,
        valid_until: u64,
        heightened_monitoring: bool,
        post_event_review: bool,
    ) -> Self {
        Self {
            authority,
            reason_id: reason_id.to_owned(),
            approval,
            valid_from,
            valid_until,
            heightened_monitoring,
            post_event_review,
        }
    }

    fn is_valid(&self) -> bool {
        break_glass_identifier_is_valid(&self.reason_id) && self.approval.is_valid()
    }

    fn matches_disclosure_request(&self, request: &SensitiveDataRequest) -> bool {
        SensitiveDataRequest::new(self.authority.clone()).eq(request)
    }
}

/// Result of evaluating one exceptional sensitive-data disclosure attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveBreakGlassDecision {
    /// Every ordinary and exceptional policy prerequisite is satisfied.
    Authorized,
    /// The ordinary disclosure decision was not an approval-gated outcome that break-glass may use.
    DisclosureNotApprovalGated(DisclosureDecision),
    /// Request or scope authority differed from the ordinary disclosure authority.
    AuthorityMismatch,
    /// Current or approved actor identity metadata was malformed or outside the bounded grammar.
    InvalidActorBinding,
    /// The currently authenticated actor differed from the actor covered by the approval.
    ActorMismatch,
    /// Approver identity metadata was malformed, duplicated, or outside the bounded grammar.
    InvalidApproverBinding,
    /// Approver binding cardinality differed from the supplied approval evidence shape.
    ApproverBindingMismatch,
    /// An approver identity matched the beneficiary actor receiving exceptional access.
    ApproverIndependenceRequired,
    /// Requested and approved break-glass reason identifiers differed.
    ReasonMismatch,
    /// Request reason metadata was malformed or outside the bounded grammar.
    InvalidRequest,
    /// Scope reason or approval metadata was malformed or internally inconsistent.
    InvalidScope,
    /// The local maximum validity window was zero and therefore unusable.
    InvalidValidityPolicy,
    /// The exceptional-access validity interval was empty or reversed.
    InvalidValidityWindow,
    /// The exceptional-access validity interval exceeded the reviewed local maximum.
    ValidityWindowTooLong,
    /// Trusted evaluation time was before the approved validity interval.
    NotYetValid,
    /// Trusted evaluation time was at or after the exclusive validity deadline.
    Expired,
    /// Approval evidence did not satisfy the ordinary human or dual-control requirement.
    ApprovalInsufficient,
    /// Heightened monitoring was not explicitly required for the exceptional access.
    HeightenedMonitoringRequired,
    /// Mandatory post-event review was not explicitly required for the exceptional access.
    PostEventReviewRequired,
}

/// Evaluate one bounded sensitive-data break-glass request.
///
/// Ordinary sensitive-data policy is evaluated first. Break-glass cannot upgrade denial, handle-only,
/// derived-value, partial-field, or ordinary full-field outcomes. Only an exact
/// [`DisclosureDecision::HumanApprovalRequired`] or [`DisclosureDecision::DualControlRequired`]
/// result reaches the exceptional controls.
///
/// `actor_binding`, `approver_binding`, and `trusted_time` must be supplied by a trusted runtime. The
/// approver binding must be derived from the same authoritative approval records represented by the
/// scope approval evidence. Time must use the same domain and units as the scope and
/// [`BreakGlassValidityPolicy`]. Even an [`SensitiveBreakGlassDecision::Authorized`] result is
/// metadata-only: a trusted broker must authenticate the current caller and approvers, verify the
/// approval records, revalidate policy/lifecycle immediately before disclosure, execute monitoring,
/// emit durable audit evidence, and ensure post-event review occurs.
#[must_use]
pub fn evaluate_sensitive_break_glass(
    disclosure_request: &SensitiveDataRequest,
    disclosure_scope: &DisclosureScope,
    break_glass_request: &SensitiveBreakGlassRequest,
    break_glass_scope: &SensitiveBreakGlassScope,
    actor_binding: &BreakGlassActorBinding,
    approver_binding: &BreakGlassApproverBinding,
    validity_policy: &BreakGlassValidityPolicy,
    trusted_time: u64,
) -> SensitiveBreakGlassDecision {
    let disclosure_decision = evaluate_disclosure(disclosure_request, disclosure_scope);
    let approval_requirement = match disclosure_decision {
        DisclosureDecision::HumanApprovalRequired => BreakGlassApprovalRequirement::Human,
        DisclosureDecision::DualControlRequired => BreakGlassApprovalRequirement::DualControl,
        other => return SensitiveBreakGlassDecision::DisclosureNotApprovalGated(other),
    };

    if !break_glass_request.is_valid() {
        return SensitiveBreakGlassDecision::InvalidRequest;
    }
    if !break_glass_scope.is_valid() {
        return SensitiveBreakGlassDecision::InvalidScope;
    }
    if !actor_binding.is_valid() {
        return SensitiveBreakGlassDecision::InvalidActorBinding;
    }
    if !approver_binding.is_valid() {
        return SensitiveBreakGlassDecision::InvalidApproverBinding;
    }
    if !validity_policy.is_valid() {
        return SensitiveBreakGlassDecision::InvalidValidityPolicy;
    }
    if !break_glass_request.matches_disclosure_request(disclosure_request)
        || !break_glass_scope.matches_disclosure_request(disclosure_request)
    {
        return SensitiveBreakGlassDecision::AuthorityMismatch;
    }
    if !actor_binding.matches() {
        return SensitiveBreakGlassDecision::ActorMismatch;
    }
    if break_glass_request.reason_id != break_glass_scope.reason_id {
        return SensitiveBreakGlassDecision::ReasonMismatch;
    }
    if break_glass_scope.valid_from >= break_glass_scope.valid_until {
        return SensitiveBreakGlassDecision::InvalidValidityWindow;
    }
    if break_glass_scope.valid_until - break_glass_scope.valid_from > validity_policy.maximum_window
    {
        return SensitiveBreakGlassDecision::ValidityWindowTooLong;
    }
    if trusted_time < break_glass_scope.valid_from {
        return SensitiveBreakGlassDecision::NotYetValid;
    }
    if trusted_time >= break_glass_scope.valid_until {
        return SensitiveBreakGlassDecision::Expired;
    }
    if !break_glass_scope.approval.satisfies(approval_requirement) {
        return SensitiveBreakGlassDecision::ApprovalInsufficient;
    }
    if !approver_binding.matches_approval(&break_glass_scope.approval) {
        return SensitiveBreakGlassDecision::ApproverBindingMismatch;
    }
    if !approver_binding.is_independent_from(actor_binding) {
        return SensitiveBreakGlassDecision::ApproverIndependenceRequired;
    }
    if !break_glass_scope.heightened_monitoring {
        return SensitiveBreakGlassDecision::HeightenedMonitoringRequired;
    }
    if !break_glass_scope.post_event_review {
        return SensitiveBreakGlassDecision::PostEventReviewRequired;
    }

    SensitiveBreakGlassDecision::Authorized
}

fn break_glass_identifier_is_valid(identifier: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }
    if identifier.len() > MAX_BREAK_GLASS_IDENTIFIER_BYTES {
        return false;
    }

    let mut has_alphanumeric = false;
    for byte in identifier.bytes() {
        if byte.is_ascii_alphanumeric() {
            has_alphanumeric = true;
        } else if !matches!(byte, b'.' | b'_' | b':' | b'-') {
            return false;
        }
    }
    has_alphanumeric
}
