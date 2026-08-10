//! Purpose-bound sensitive-data access evidence without protected values.
//!
//! These value objects record authorization metadata only. They intentionally
//! exclude protected field values, opaque-handle payloads, credentials, model
//! prompts, and other content that could turn an audit receipt into a new data
//! disclosure channel.

use std::collections::BTreeSet;
use std::fmt;

use originweave_core::Origin;

/// Maximum byte length of one sensitive-access evidence identifier.
pub const MAX_SENSITIVE_IDENTIFIER_BYTES: usize = 128;
/// Maximum number of protected field identifiers carried by one evidence receipt.
pub const MAX_SENSITIVE_FIELD_COUNT: usize = 64;

/// Classification recorded for the protected fields involved in one access decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SensitiveAccessClass {
    /// Public information that does not require sensitive-data handling.
    PublicData,
    /// Internal information that is not intended for unrestricted disclosure.
    InternalData,
    /// Personal information associated with an identifiable person.
    PersonalData,
    /// Sensitive personal information requiring stronger disclosure controls.
    SensitivePersonalData,
    /// Authentication, authorization, or credential material.
    CredentialData,
    /// Payment or financial-account material.
    PaymentData,
}

/// Outcome recorded for one purpose-bound sensitive-data access decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SensitiveAccessOutcome {
    /// No disclosure was authorized.
    DenyAccess,
    /// Only an opaque broker handle was authorized; no protected value was disclosed.
    OpaqueHandleOnly,
    /// Only an authorized derived value was disclosed.
    DerivedValueOnly,
    /// A bounded subset of the protected field was disclosed.
    PartialFieldDisclosure,
    /// The complete protected field was disclosed to the exact bound destination.
    FullFieldDisclosure,
    /// The request stopped pending a separate human approval decision.
    HumanApprovalRequired,
    /// The request stopped pending two independent controls.
    DualControlRequired,
}

impl SensitiveAccessOutcome {
    const fn records_disclosure(self) -> bool {
        matches!(
            self,
            Self::DerivedValueOnly | Self::PartialFieldDisclosure | Self::FullFieldDisclosure
        )
    }
}

/// Validation failure while constructing sensitive-access evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveEvidenceError {
    /// An authority, policy, or approval identifier was empty, oversized, or ambiguous.
    InvalidIdentifier,
    /// The protected-field identifier set was empty, oversized, malformed, or duplicated.
    InvalidFieldSet,
    /// Decision, disclosure, outcome, or retention timestamps described an impossible lifecycle.
    InvalidLifecycle,
}

impl fmt::Display for SensitiveEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentifier => "invalid sensitive-access identifier",
            Self::InvalidFieldSet => "invalid sensitive-access field set",
            Self::InvalidLifecycle => "invalid sensitive-access lifecycle",
        })
    }
}

impl std::error::Error for SensitiveEvidenceError {}

/// Unvalidated metadata for one purpose-bound sensitive-data access receipt.
///
/// This input type contains authority and lifecycle metadata only. It has no
/// protected-value field by design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveAccessEvidenceInput {
    /// Correlation identifier for the originating access request.
    pub request_id: String,
    /// Identifier for the policy decision that produced the recorded outcome.
    pub decision_id: String,
    /// Tenant whose protected data was governed by the decision.
    pub tenant_id: String,
    /// Human, workload, or service actor accountable for the requested access.
    pub actor_id: String,
    /// Bounded task whose business purpose authorized the access decision.
    pub task_id: String,
    /// Protected field identifiers governed by this exact decision, never field values.
    pub field_ids: Vec<String>,
    /// Declared business-purpose identifier for the requested access.
    pub purpose_id: String,
    /// Canonical destination origin bound to the access decision.
    pub destination: Origin,
    /// Classification applied to the governed fields.
    pub classification: SensitiveAccessClass,
    /// Decision outcome recorded by the evidence receipt.
    pub outcome: SensitiveAccessOutcome,
    /// Version identifier for the policy evaluated by the decision.
    pub policy_version: String,
    /// Optional reference to separate approval evidence, never the approval payload itself.
    pub approval_reference: Option<String>,
    /// Trusted Unix epoch second at which policy reached the recorded decision.
    pub decision_epoch_seconds: u64,
    /// Trusted Unix epoch second at which an authorized value or derived value was disclosed.
    pub disclosure_epoch_seconds: Option<u64>,
    /// Optional Unix epoch second after which this receipt's governed retention period ends.
    pub retention_deadline_epoch_seconds: Option<u64>,
}

/// Immutable credential-free receipt for one sensitive-data access decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveAccessEvidence {
    request_id: String,
    decision_id: String,
    tenant_id: String,
    actor_id: String,
    task_id: String,
    field_ids: Vec<String>,
    purpose_id: String,
    destination: Origin,
    classification: SensitiveAccessClass,
    outcome: SensitiveAccessOutcome,
    policy_version: String,
    approval_reference: Option<String>,
    decision_epoch_seconds: u64,
    disclosure_epoch_seconds: Option<u64>,
    retention_deadline_epoch_seconds: Option<u64>,
}

impl TryFrom<SensitiveAccessEvidenceInput> for SensitiveAccessEvidence {
    type Error = SensitiveEvidenceError;

    fn try_from(input: SensitiveAccessEvidenceInput) -> Result<Self, Self::Error> {
        validate_identifiers(&input)?;
        validate_fields(&input.field_ids)?;
        validate_lifecycle(&input)?;
        Ok(Self {
            request_id: input.request_id,
            decision_id: input.decision_id,
            tenant_id: input.tenant_id,
            actor_id: input.actor_id,
            task_id: input.task_id,
            field_ids: input.field_ids,
            purpose_id: input.purpose_id,
            destination: input.destination,
            classification: input.classification,
            outcome: input.outcome,
            policy_version: input.policy_version,
            approval_reference: input.approval_reference,
            decision_epoch_seconds: input.decision_epoch_seconds,
            disclosure_epoch_seconds: input.disclosure_epoch_seconds,
            retention_deadline_epoch_seconds: input.retention_deadline_epoch_seconds,
        })
    }
}

impl SensitiveAccessEvidence {
    /// Return the originating request identifier.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return the policy decision identifier.
    #[must_use]
    pub fn decision_id(&self) -> &str {
        &self.decision_id
    }

    /// Return the tenant identifier bound to the decision.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Return the accountable actor identifier.
    #[must_use]
    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }

    /// Return the task identifier bound to the decision.
    #[must_use]
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// Return the exact protected-field identifiers without any field values.
    #[must_use]
    pub fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    /// Return the declared business-purpose identifier.
    #[must_use]
    pub fn purpose_id(&self) -> &str {
        &self.purpose_id
    }

    /// Return the canonical destination origin bound to the decision.
    #[must_use]
    pub const fn destination(&self) -> &Origin {
        &self.destination
    }

    /// Return the recorded sensitive-data classification.
    #[must_use]
    pub const fn classification(&self) -> SensitiveAccessClass {
        self.classification
    }

    /// Return the recorded access outcome.
    #[must_use]
    pub const fn outcome(&self) -> SensitiveAccessOutcome {
        self.outcome
    }

    /// Return the evaluated policy version identifier.
    #[must_use]
    pub fn policy_version(&self) -> &str {
        &self.policy_version
    }

    /// Return the optional reference to separate approval evidence.
    #[must_use]
    pub fn approval_reference(&self) -> Option<&str> {
        self.approval_reference.as_deref()
    }

    /// Return the trusted policy-decision time as a Unix epoch second.
    #[must_use]
    pub const fn decision_epoch_seconds(&self) -> u64 {
        self.decision_epoch_seconds
    }

    /// Return the trusted disclosure time when the outcome actually disclosed data.
    #[must_use]
    pub const fn disclosure_epoch_seconds(&self) -> Option<u64> {
        self.disclosure_epoch_seconds
    }

    /// Return the optional retention deadline for the receipt.
    #[must_use]
    pub const fn retention_deadline_epoch_seconds(&self) -> Option<u64> {
        self.retention_deadline_epoch_seconds
    }
}

fn validate_identifiers(
    input: &SensitiveAccessEvidenceInput,
) -> Result<(), SensitiveEvidenceError> {
    let required = [
        input.request_id.as_str(),
        input.decision_id.as_str(),
        input.tenant_id.as_str(),
        input.actor_id.as_str(),
        input.task_id.as_str(),
        input.purpose_id.as_str(),
        input.policy_version.as_str(),
    ];
    if required.into_iter().any(|value| !valid_identifier(value))
        || input
            .approval_reference
            .as_deref()
            .is_some_and(|value| !valid_identifier(value))
    {
        return Err(SensitiveEvidenceError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_fields(field_ids: &[String]) -> Result<(), SensitiveEvidenceError> {
    if field_ids.is_empty() || field_ids.len() > MAX_SENSITIVE_FIELD_COUNT {
        return Err(SensitiveEvidenceError::InvalidFieldSet);
    }
    let mut unique_fields = BTreeSet::new();
    if field_ids
        .iter()
        .any(|field_id| !valid_identifier(field_id) || !unique_fields.insert(field_id.as_str()))
    {
        return Err(SensitiveEvidenceError::InvalidFieldSet);
    }
    Ok(())
}

pub(crate) fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SENSITIVE_IDENTIFIER_BYTES
        && value.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn validate_lifecycle(input: &SensitiveAccessEvidenceInput) -> Result<(), SensitiveEvidenceError> {
    if input.decision_epoch_seconds == 0 {
        return Err(SensitiveEvidenceError::InvalidLifecycle);
    }
    if input.outcome.records_disclosure() != input.disclosure_epoch_seconds.is_some() {
        return Err(SensitiveEvidenceError::InvalidLifecycle);
    }
    if input
        .disclosure_epoch_seconds
        .is_some_and(|disclosure| disclosure < input.decision_epoch_seconds)
    {
        return Err(SensitiveEvidenceError::InvalidLifecycle);
    }
    let lifecycle_floor = input
        .disclosure_epoch_seconds
        .unwrap_or(input.decision_epoch_seconds);
    if input
        .retention_deadline_epoch_seconds
        .is_some_and(|deadline| deadline <= lifecycle_floor)
    {
        return Err(SensitiveEvidenceError::InvalidLifecycle);
    }
    Ok(())
}
