//! Credential-free metadata for completed sensitive break-glass post-event review.
//!
//! Review evidence binds to one exact break-glass disclosure receipt. It records bounded reviewer,
//! outcome, finding-count, remediation, completion, and timeliness metadata without carrying
//! protected values, free-form findings, approval payloads, credentials, or model data.

use std::fmt;

use crate::{MAX_SENSITIVE_IDENTIFIER_BYTES, SensitiveBreakGlassEvidence};

/// Bounded outcome of one completed break-glass post-event review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveBreakGlassReviewOutcome {
    /// The reviewer found the exceptional disclosure compliant with reviewed policy.
    ConfirmedCompliant,
    /// The reviewer found one or more policy violations requiring remediation.
    PolicyViolation,
    /// The reviewer escalated one or more findings into an incident process.
    IncidentEscalated,
}

/// Whether a completed review met the deadline recorded by the disclosure receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveBreakGlassReviewTimeliness {
    /// Review completed no later than the mandatory due time.
    OnTime,
    /// Review completed after the due time but before evidence-retention expiry.
    Late,
}

/// Validation failure for one completed break-glass review receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveBreakGlassReviewEvidenceError {
    /// Reviewer or remediation metadata was empty, malformed, or oversized.
    InvalidIdentifier,
    /// The reviewer identity matched the actor that performed or was approved for disclosure.
    ReviewerConflict,
    /// Completion did not follow disclosure or exceeded the evidence-retention deadline.
    InvalidCompletionTime,
    /// Outcome, finding count, and remediation metadata were inconsistent.
    InvalidOutcomeEvidence,
}

impl fmt::Display for SensitiveBreakGlassReviewEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentifier => "invalid sensitive break-glass review identifier",
            Self::ReviewerConflict => {
                "sensitive break-glass reviewer conflicts with disclosed actor"
            }
            Self::InvalidCompletionTime => "invalid sensitive break-glass review completion time",
            Self::InvalidOutcomeEvidence => "invalid sensitive break-glass review outcome evidence",
        })
    }
}

impl std::error::Error for SensitiveBreakGlassReviewEvidenceError {}

/// Untrusted construction input for one completed break-glass review receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveBreakGlassReviewEvidenceInput {
    /// Authenticated reviewer identity derived by a trusted review service.
    pub reviewer_id: String,
    /// Trusted time when review completed.
    pub completed_epoch_seconds: u64,
    /// Bounded review outcome.
    pub outcome: SensitiveBreakGlassReviewOutcome,
    /// Number of findings recorded by the authoritative review process.
    pub finding_count: u32,
    /// Bounded remediation or incident reference when findings exist.
    pub remediation_reference: Option<String>,
}

/// Immutable credential-free receipt for one completed break-glass post-event review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveBreakGlassReviewEvidence {
    request_id: String,
    decision_id: String,
    review_reference: String,
    reviewer_id: String,
    completed_epoch_seconds: u64,
    timeliness: SensitiveBreakGlassReviewTimeliness,
    outcome: SensitiveBreakGlassReviewOutcome,
    finding_count: u32,
    remediation_reference: Option<String>,
}

impl SensitiveBreakGlassReviewEvidence {
    /// Validate and bind one review completion to an exact break-glass disclosure receipt.
    ///
    /// Late review remains recordable while the receipt is still inside its retention deadline. This
    /// value does not authenticate the reviewer, inspect review content, prove remediation, persist
    /// evidence, enforce retention, or close an incident.
    pub fn try_from_receipt(
        receipt: &SensitiveBreakGlassEvidence,
        input: SensitiveBreakGlassReviewEvidenceInput,
    ) -> Result<Self, SensitiveBreakGlassReviewEvidenceError> {
        validate_identifiers(&input)?;
        if input.reviewer_id == receipt.actor_id() {
            return Err(SensitiveBreakGlassReviewEvidenceError::ReviewerConflict);
        }
        if input.completed_epoch_seconds <= receipt.disclosure_epoch_seconds()
            || input.completed_epoch_seconds > receipt.retention_deadline_epoch_seconds()
        {
            return Err(SensitiveBreakGlassReviewEvidenceError::InvalidCompletionTime);
        }
        validate_outcome(&input)?;

        let timeliness =
            if input.completed_epoch_seconds <= receipt.post_event_review_due_epoch_seconds() {
                SensitiveBreakGlassReviewTimeliness::OnTime
            } else {
                SensitiveBreakGlassReviewTimeliness::Late
            };

        Ok(Self {
            request_id: receipt.request_id().to_owned(),
            decision_id: receipt.decision_id().to_owned(),
            review_reference: receipt.post_event_review_reference().to_owned(),
            reviewer_id: input.reviewer_id,
            completed_epoch_seconds: input.completed_epoch_seconds,
            timeliness,
            outcome: input.outcome,
            finding_count: input.finding_count,
            remediation_reference: input.remediation_reference,
        })
    }

    /// Return the originating break-glass request identifier.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return the originating break-glass decision identifier.
    #[must_use]
    pub fn decision_id(&self) -> &str {
        &self.decision_id
    }

    /// Return the mandatory review correlation identifier from the disclosure receipt.
    #[must_use]
    pub fn review_reference(&self) -> &str {
        &self.review_reference
    }

    /// Return the bounded reviewer identifier.
    #[must_use]
    pub fn reviewer_id(&self) -> &str {
        &self.reviewer_id
    }

    /// Return the trusted completion time.
    #[must_use]
    pub const fn completed_epoch_seconds(&self) -> u64 {
        self.completed_epoch_seconds
    }

    /// Return whether review met the original due time.
    #[must_use]
    pub const fn timeliness(&self) -> SensitiveBreakGlassReviewTimeliness {
        self.timeliness
    }

    /// Return the bounded review outcome.
    #[must_use]
    pub const fn outcome(&self) -> SensitiveBreakGlassReviewOutcome {
        self.outcome
    }

    /// Return the authoritative bounded finding count.
    #[must_use]
    pub const fn finding_count(&self) -> u32 {
        self.finding_count
    }

    /// Return the bounded remediation or incident reference when findings exist.
    #[must_use]
    pub fn remediation_reference(&self) -> Option<&str> {
        self.remediation_reference.as_deref()
    }
}

fn validate_identifiers(
    input: &SensitiveBreakGlassReviewEvidenceInput,
) -> Result<(), SensitiveBreakGlassReviewEvidenceError> {
    if !valid_identifier(&input.reviewer_id)
        || input
            .remediation_reference
            .as_deref()
            .is_some_and(|reference| !valid_identifier(reference))
    {
        return Err(SensitiveBreakGlassReviewEvidenceError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_outcome(
    input: &SensitiveBreakGlassReviewEvidenceInput,
) -> Result<(), SensitiveBreakGlassReviewEvidenceError> {
    let valid = match input.outcome {
        SensitiveBreakGlassReviewOutcome::ConfirmedCompliant => {
            input.finding_count == 0 && input.remediation_reference.is_none()
        }
        SensitiveBreakGlassReviewOutcome::PolicyViolation
        | SensitiveBreakGlassReviewOutcome::IncidentEscalated => {
            input.finding_count > 0 && input.remediation_reference.is_some()
        }
    };
    if !valid {
        return Err(SensitiveBreakGlassReviewEvidenceError::InvalidOutcomeEvidence);
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
