//! Credential-free audit metadata for policy-approved model disclosure.
//!
//! This value object records only the reviewed route and policy identifiers
//! associated with a sensitive-data decision. It intentionally carries no
//! protected value, model prompt or output, provider credential, or execution
//! authority.

use crate::sensitive_access::{SensitiveEvidenceError, valid_identifier};

/// Unvalidated metadata describing one sensitive-data model disclosure route.
///
/// The input is restricted to correlation and policy identifiers. It cannot
/// carry protected field values, prompts, model outputs, or provider secrets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveModelDisclosureEvidenceInput {
    /// Correlation identifier for the sensitive-data access request.
    pub request_id: String,
    /// Identifier for the policy decision governing the disclosure.
    pub decision_id: String,
    /// Reviewed provider identifier selected for the model route.
    pub provider_id: String,
    /// Reviewed model identifier selected for the model route.
    pub model_id: String,
    /// Reviewed processing-region identifier for the model route.
    pub region_id: String,
    /// Reviewed retention-policy identifier for the model route.
    pub retention_policy_id: String,
    /// Reviewed provider training-policy identifier for the model route.
    pub training_policy_id: String,
    /// Reviewed subprocessor-policy identifier for the model route.
    pub subprocessor_policy_id: String,
    /// Reviewed export-policy identifier for the model route.
    pub export_policy_id: String,
}

/// Immutable credential-free evidence for one approved model disclosure route.
///
/// Construction validates every identifier with the same bounded evidence
/// rules as other sensitive-access records. The object records policy metadata
/// only and does not authorize or execute a model disclosure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveModelDisclosureEvidence {
    request_id: String,
    decision_id: String,
    provider_id: String,
    model_id: String,
    region_id: String,
    retention_policy_id: String,
    training_policy_id: String,
    subprocessor_policy_id: String,
    export_policy_id: String,
}

impl TryFrom<SensitiveModelDisclosureEvidenceInput> for SensitiveModelDisclosureEvidence {
    type Error = SensitiveEvidenceError;

    fn try_from(input: SensitiveModelDisclosureEvidenceInput) -> Result<Self, Self::Error> {
        if !valid_identifier(&input.request_id)
            || !valid_identifier(&input.decision_id)
            || !valid_identifier(&input.provider_id)
            || !valid_identifier(&input.model_id)
            || !valid_identifier(&input.region_id)
            || !valid_identifier(&input.retention_policy_id)
            || !valid_identifier(&input.training_policy_id)
            || !valid_identifier(&input.subprocessor_policy_id)
            || !valid_identifier(&input.export_policy_id)
        {
            return Err(SensitiveEvidenceError::InvalidIdentifier);
        }

        Ok(Self {
            request_id: input.request_id,
            decision_id: input.decision_id,
            provider_id: input.provider_id,
            model_id: input.model_id,
            region_id: input.region_id,
            retention_policy_id: input.retention_policy_id,
            training_policy_id: input.training_policy_id,
            subprocessor_policy_id: input.subprocessor_policy_id,
            export_policy_id: input.export_policy_id,
        })
    }
}

impl SensitiveModelDisclosureEvidence {
    /// Return the originating sensitive-data access request identifier.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return the sensitive-data policy decision identifier.
    #[must_use]
    pub fn decision_id(&self) -> &str {
        &self.decision_id
    }

    /// Return the reviewed provider identifier.
    #[must_use]
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Return the reviewed model identifier.
    #[must_use]
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Return the reviewed processing-region identifier.
    #[must_use]
    pub fn region_id(&self) -> &str {
        &self.region_id
    }

    /// Return the reviewed retention-policy identifier.
    #[must_use]
    pub fn retention_policy_id(&self) -> &str {
        &self.retention_policy_id
    }

    /// Return the reviewed provider training-policy identifier.
    #[must_use]
    pub fn training_policy_id(&self) -> &str {
        &self.training_policy_id
    }

    /// Return the reviewed subprocessor-policy identifier.
    #[must_use]
    pub fn subprocessor_policy_id(&self) -> &str {
        &self.subprocessor_policy_id
    }

    /// Return the reviewed export-policy identifier.
    #[must_use]
    pub fn export_policy_id(&self) -> &str {
        &self.export_policy_id
    }
}
