//! Credential-free audit metadata for one actual sensitive-data break-glass disclosure.
//!
//! This module records bounded authority, approval, timing, monitoring, and review correlation. It
//! does not carry protected values or opaque handles, prove that policy authorization occurred,
//! authenticate identities, verify signatures, execute monitoring, complete a review, or persist
//! evidence.

use std::collections::BTreeSet;
use std::fmt;

use originweave_core::Origin;

use crate::{
    MAX_SENSITIVE_FIELD_COUNT, MAX_SENSITIVE_IDENTIFIER_BYTES, SensitiveAccessClass,
    SensitiveAccessOutcome,
};

/// Approval cardinality recorded for one break-glass event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveBreakGlassApprovalMode {
    /// One human approval reference is required.
    Human,
    /// Two distinct human approval references are required.
    DualControl,
}

/// Validation failure for one sensitive break-glass evidence receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveBreakGlassEvidenceError {
    /// A required identifier was empty, malformed, or oversized.
    InvalidIdentifier,
    /// The governed field set was empty, malformed, duplicated, or oversized.
    InvalidFieldSet,
    /// The actor who disclosed differed from the actor covered by the approval.
    ActorMismatch,
    /// Approval-reference count or uniqueness did not match the approval mode.
    InvalidApprovalSet,
    /// The outcome did not represent an actual disclosure.
    DisclosureNotRecorded,
    /// The validity interval or local maximum was empty or reversed.
    InvalidValidityWindow,
    /// The validity interval exceeded the explicit local maximum.
    WindowExceedsMaximum,
    /// Decision, disclosure, or review timestamps had impossible ordering.
    InvalidLifecycle,
}

impl fmt::Display for SensitiveBreakGlassEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidIdentifier => "invalid sensitive break-glass identifier",
            Self::InvalidFieldSet => "invalid sensitive break-glass field set",
            Self::ActorMismatch => "sensitive break-glass actor mismatch",
            Self::InvalidApprovalSet => "invalid sensitive break-glass approval set",
            Self::DisclosureNotRecorded => {
                "sensitive break-glass evidence did not record a disclosure"
            }
            Self::InvalidValidityWindow => "invalid sensitive break-glass validity window",
            Self::WindowExceedsMaximum => {
                "sensitive break-glass validity window exceeds local maximum"
            }
            Self::InvalidLifecycle => "invalid sensitive break-glass evidence lifecycle",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SensitiveBreakGlassEvidenceError {}

/// Untrusted construction input for one break-glass audit receipt.
///
/// Every field is validated by [`SensitiveBreakGlassEvidence::try_from`]. The structure intentionally
/// has no protected-value, opaque-handle, credential, prompt, model-output, or provider-token field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveBreakGlassEvidenceInput {
    /// Correlation identifier for the originating access request.
    pub request_id: String,
    /// Correlation identifier for the ordinary and break-glass policy decision.
    pub decision_id: String,
    /// Tenant authority that owned the governed data.
    pub tenant_id: String,
    /// Authenticated actor that performed the disclosure.
    pub actor_id: String,
    /// Exact actor identity covered by the break-glass approval.
    pub approved_actor_id: String,
    /// Task authority that owned the exceptional access.
    pub task_id: String,
    /// Exact governed field identifiers.
    pub field_ids: Vec<String>,
    /// Approved purpose identifier.
    pub purpose_id: String,
    /// Canonical destination that received the disclosure.
    pub destination: Origin,
    /// Data classification of the governed field set.
    pub classification: SensitiveAccessClass,
    /// Actual disclosure outcome observed by the trusted runtime.
    pub outcome: SensitiveAccessOutcome,
    /// Policy-version identifier used for the decision.
    pub policy_version: String,
    /// Durable incident, legal, support, or emergency reason reference.
    pub reason_id: String,
    /// Human-versus-dual-control approval requirement that was satisfied.
    pub approval_mode: SensitiveBreakGlassApprovalMode,
    /// Exact bounded approval references used by the event.
    pub approval_references: Vec<String>,
    /// Inclusive start of the exceptional-access validity interval.
    pub valid_from_epoch_seconds: u64,
    /// Exclusive end of the exceptional-access validity interval.
    pub valid_until_epoch_seconds: u64,
    /// Local maximum accepted interval length in the same time units.
    pub maximum_window_seconds: u64,
    /// Trusted policy-decision timestamp.
    pub decision_epoch_seconds: u64,
    /// Trusted timestamp when protected data actually left the broker boundary.
    pub disclosure_epoch_seconds: u64,
    /// Correlation identifier for heightened monitoring of the event.
    pub monitoring_reference: String,
    /// Correlation identifier for mandatory post-event review.
    pub post_event_review_reference: String,
    /// Deadline after disclosure by which post-event review must occur.
    pub post_event_review_due_epoch_seconds: u64,
}

/// Immutable credential-free receipt for one actual break-glass disclosure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveBreakGlassEvidence {
    request_id: String,
    decision_id: String,
    tenant_id: String,
    actor_id: String,
    approved_actor_id: String,
    task_id: String,
    field_ids: Vec<String>,
    purpose_id: String,
    destination: Origin,
    classification: SensitiveAccessClass,
    outcome: SensitiveAccessOutcome,
    policy_version: String,
    reason_id: String,
    approval_mode: SensitiveBreakGlassApprovalMode,
    approval_references: Vec<String>,
    valid_from_epoch_seconds: u64,
    valid_until_epoch_seconds: u64,
    maximum_window_seconds: u64,
    decision_epoch_seconds: u64,
    disclosure_epoch_seconds: u64,
    monitoring_reference: String,
    post_event_review_reference: String,
    post_event_review_due_epoch_seconds: u64,
}

impl TryFrom<SensitiveBreakGlassEvidenceInput> for SensitiveBreakGlassEvidence {
    type Error = SensitiveBreakGlassEvidenceError;

    fn try_from(input: SensitiveBreakGlassEvidenceInput) -> Result<Self, Self::Error> {
        validate_identifiers(&input)?;
        validate_field_ids(&input.field_ids)?;
        if input.actor_id != input.approved_actor_id {
            return Err(SensitiveBreakGlassEvidenceError::ActorMismatch);
        }
        validate_approval_set(input.approval_mode, &input.approval_references)?;
        if !records_disclosure(input.outcome) {
            return Err(SensitiveBreakGlassEvidenceError::DisclosureNotRecorded);
        }
        validate_validity_window(&input)?;
        validate_lifecycle(&input)?;

        Ok(Self {
            request_id: input.request_id,
            decision_id: input.decision_id,
            tenant_id: input.tenant_id,
            actor_id: input.actor_id,
            approved_actor_id: input.approved_actor_id,
            task_id: input.task_id,
            field_ids: input.field_ids,
            purpose_id: input.purpose_id,
            destination: input.destination,
            classification: input.classification,
            outcome: input.outcome,
            policy_version: input.policy_version,
            reason_id: input.reason_id,
            approval_mode: input.approval_mode,
            approval_references: input.approval_references,
            valid_from_epoch_seconds: input.valid_from_epoch_seconds,
            valid_until_epoch_seconds: input.valid_until_epoch_seconds,
            maximum_window_seconds: input.maximum_window_seconds,
            decision_epoch_seconds: input.decision_epoch_seconds,
            disclosure_epoch_seconds: input.disclosure_epoch_seconds,
            monitoring_reference: input.monitoring_reference,
            post_event_review_reference: input.post_event_review_reference,
            post_event_review_due_epoch_seconds: input.post_event_review_due_epoch_seconds,
        })
    }
}

impl SensitiveBreakGlassEvidence {
    /// Return the originating request identifier.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return the policy-decision identifier.
    #[must_use]
    pub fn decision_id(&self) -> &str {
        &self.decision_id
    }

    /// Return the governing tenant identifier.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Return the authenticated actor that performed the disclosure.
    #[must_use]
    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }

    /// Return the exact actor identity covered by approval.
    #[must_use]
    pub fn approved_actor_id(&self) -> &str {
        &self.approved_actor_id
    }

    /// Return the governing task identifier.
    #[must_use]
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// Return the exact governed field identifiers.
    #[must_use]
    pub fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    /// Return the approved purpose identifier.
    #[must_use]
    pub fn purpose_id(&self) -> &str {
        &self.purpose_id
    }

    /// Return the canonical destination that received disclosure.
    #[must_use]
    pub const fn destination(&self) -> &Origin {
        &self.destination
    }

    /// Return the governed data classification.
    #[must_use]
    pub const fn classification(&self) -> SensitiveAccessClass {
        self.classification
    }

    /// Return the actual disclosure outcome.
    #[must_use]
    pub const fn outcome(&self) -> SensitiveAccessOutcome {
        self.outcome
    }

    /// Return the policy-version identifier.
    #[must_use]
    pub fn policy_version(&self) -> &str {
        &self.policy_version
    }

    /// Return the durable break-glass reason reference.
    #[must_use]
    pub fn reason_id(&self) -> &str {
        &self.reason_id
    }

    /// Return the approval cardinality used by the event.
    #[must_use]
    pub const fn approval_mode(&self) -> SensitiveBreakGlassApprovalMode {
        self.approval_mode
    }

    /// Return the exact bounded approval references.
    #[must_use]
    pub fn approval_references(&self) -> &[String] {
        &self.approval_references
    }

    /// Return the inclusive validity start.
    #[must_use]
    pub const fn valid_from_epoch_seconds(&self) -> u64 {
        self.valid_from_epoch_seconds
    }

    /// Return the exclusive validity deadline.
    #[must_use]
    pub const fn valid_until_epoch_seconds(&self) -> u64 {
        self.valid_until_epoch_seconds
    }

    /// Return the local maximum accepted validity window.
    #[must_use]
    pub const fn maximum_window_seconds(&self) -> u64 {
        self.maximum_window_seconds
    }

    /// Return the trusted decision timestamp.
    #[must_use]
    pub const fn decision_epoch_seconds(&self) -> u64 {
        self.decision_epoch_seconds
    }

    /// Return the trusted actual-disclosure timestamp.
    #[must_use]
    pub const fn disclosure_epoch_seconds(&self) -> u64 {
        self.disclosure_epoch_seconds
    }

    /// Return the heightened-monitoring correlation identifier.
    #[must_use]
    pub fn monitoring_reference(&self) -> &str {
        &self.monitoring_reference
    }

    /// Return the mandatory post-event-review correlation identifier.
    #[must_use]
    pub fn post_event_review_reference(&self) -> &str {
        &self.post_event_review_reference
    }

    /// Return the mandatory post-event-review deadline.
    #[must_use]
    pub const fn post_event_review_due_epoch_seconds(&self) -> u64 {
        self.post_event_review_due_epoch_seconds
    }
}

fn validate_identifiers(
    input: &SensitiveBreakGlassEvidenceInput,
) -> Result<(), SensitiveBreakGlassEvidenceError> {
    let required = [
        input.request_id.as_str(),
        input.decision_id.as_str(),
        input.tenant_id.as_str(),
        input.actor_id.as_str(),
        input.approved_actor_id.as_str(),
        input.task_id.as_str(),
        input.purpose_id.as_str(),
        input.policy_version.as_str(),
        input.reason_id.as_str(),
        input.monitoring_reference.as_str(),
        input.post_event_review_reference.as_str(),
    ];
    if required.into_iter().any(|value| !valid_identifier(value))
        || input
            .approval_references
            .iter()
            .any(|value| !valid_identifier(value))
    {
        return Err(SensitiveBreakGlassEvidenceError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_field_ids(field_ids: &[String]) -> Result<(), SensitiveBreakGlassEvidenceError> {
    if field_ids.is_empty() || field_ids.len() > MAX_SENSITIVE_FIELD_COUNT {
        return Err(SensitiveBreakGlassEvidenceError::InvalidFieldSet);
    }
    let mut unique = BTreeSet::new();
    if field_ids
        .iter()
        .any(|field_id| !valid_identifier(field_id) || !unique.insert(field_id.as_str()))
    {
        return Err(SensitiveBreakGlassEvidenceError::InvalidFieldSet);
    }
    Ok(())
}

fn validate_approval_set(
    mode: SensitiveBreakGlassApprovalMode,
    approval_references: &[String],
) -> Result<(), SensitiveBreakGlassEvidenceError> {
    let valid = match mode {
        SensitiveBreakGlassApprovalMode::Human => approval_references.len() == 1,
        SensitiveBreakGlassApprovalMode::DualControl => {
            approval_references.len() == 2
                && approval_references[0] != approval_references[1]
        }
    };
    if !valid {
        return Err(SensitiveBreakGlassEvidenceError::InvalidApprovalSet);
    }
    Ok(())
}

const fn records_disclosure(outcome: SensitiveAccessOutcome) -> bool {
    matches!(
        outcome,
        SensitiveAccessOutcome::DerivedValueOnly
            | SensitiveAccessOutcome::PartialFieldDisclosure
            | SensitiveAccessOutcome::FullFieldDisclosure
    )
}

fn validate_validity_window(
    input: &SensitiveBreakGlassEvidenceInput,
) -> Result<(), SensitiveBreakGlassEvidenceError> {
    if input.maximum_window_seconds == 0
        || input.valid_from_epoch_seconds >= input.valid_until_epoch_seconds
    {
        return Err(SensitiveBreakGlassEvidenceError::InvalidValidityWindow);
    }
    if input.valid_until_epoch_seconds - input.valid_from_epoch_seconds
        > input.maximum_window_seconds
    {
        return Err(SensitiveBreakGlassEvidenceError::WindowExceedsMaximum);
    }
    Ok(())
}

fn validate_lifecycle(
    input: &SensitiveBreakGlassEvidenceInput,
) -> Result<(), SensitiveBreakGlassEvidenceError> {
    if input.decision_epoch_seconds == 0
        || input.decision_epoch_seconds < input.valid_from_epoch_seconds
        || input.decision_epoch_seconds >= input.valid_until_epoch_seconds
        || input.disclosure_epoch_seconds < input.decision_epoch_seconds
        || input.disclosure_epoch_seconds >= input.valid_until_epoch_seconds
        || input.post_event_review_due_epoch_seconds <= input.disclosure_epoch_seconds
    {
        return Err(SensitiveBreakGlassEvidenceError::InvalidLifecycle);
    }
    Ok(())
}

fn valid_identifier(identifier: &str) -> bool {
    if identifier.is_empty() || identifier.len() > MAX_SENSITIVE_IDENTIFIER_BYTES {
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
