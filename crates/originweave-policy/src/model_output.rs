//! Separate admission for validated model output and reviewed retention policy.
//!
//! This module evaluates output-policy metadata only. Authorization never implies that OriginWeave
//! inspected model-output bytes, executed schema validation, persisted output, enforced retention,
//! or disclosed a protected value. A trusted validator and retention owner must supply the facts
//! consumed by this deterministic policy boundary.

const MAX_OUTPUT_POLICY_IDENTIFIER_BYTES: usize = 128;

/// Trusted validation result for one model output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelOutputValidation {
    /// A trusted validator accepted the output under the reviewed schema contract.
    Validated,
    /// A trusted validator rejected the output.
    Rejected,
}

/// Result of evaluating one model output against a separate reviewed output policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelOutputDecision {
    /// The output policy matches exactly and trusted validation accepted the output.
    Authorized,
    /// Output-schema or retention-policy metadata is malformed or does not match exactly.
    OutputPolicyMismatch,
    /// The reviewed output policy matched, but trusted validation rejected the output.
    ValidationRejected,
}

/// One proposed model output after model invocation has completed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOutputRequest {
    output_schema_id: String,
    output_retention_policy_id: String,
    validation: ModelOutputValidation,
}

impl ModelOutputRequest {
    /// Build output metadata without authorizing validation, persistence, retention, or disclosure.
    #[must_use]
    pub fn new(
        output_schema_id: &str,
        output_retention_policy_id: &str,
        validation: ModelOutputValidation,
    ) -> Self {
        Self {
            output_schema_id: output_schema_id.to_owned(),
            output_retention_policy_id: output_retention_policy_id.to_owned(),
            validation,
        }
    }
}

/// Reviewed output-schema and retention-policy scope for one model-output boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOutputScope {
    output_schema_id: String,
    output_retention_policy_id: String,
}

impl ModelOutputScope {
    /// Build trusted output policy metadata.
    ///
    /// Identifiers are validated during evaluation so malformed trusted policy cannot become
    /// authority merely because request and scope happen to match.
    #[must_use]
    pub fn new(output_schema_id: &str, output_retention_policy_id: &str) -> Self {
        Self {
            output_schema_id: output_schema_id.to_owned(),
            output_retention_policy_id: output_retention_policy_id.to_owned(),
        }
    }
}

fn output_policy_identifier_is_valid(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier.len() <= MAX_OUTPUT_POLICY_IDENTIFIER_BYTES
        && identifier.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

/// Evaluate one trusted validation result under an exact separate output policy.
///
/// Policy metadata is evaluated before the validation result so a rejected output carrying a
/// different schema or retention policy remains a policy mismatch rather than collapsing distinct
/// governance failures. Identifiers are bounded 1–128 byte ASCII policy tokens containing at least
/// one alphanumeric character and otherwise only `.`, `_`, `:`, or `-`.
///
/// Authorization is metadata-only. It does not inspect output bytes, perform schema validation,
/// persist output, enforce retention, authorize model invocation, or disclose protected values.
#[must_use]
pub fn evaluate_model_output(
    request: &ModelOutputRequest,
    scope: &ModelOutputScope,
) -> ModelOutputDecision {
    if !output_policy_identifier_is_valid(&request.output_schema_id)
        || !output_policy_identifier_is_valid(&request.output_retention_policy_id)
        || !output_policy_identifier_is_valid(&scope.output_schema_id)
        || !output_policy_identifier_is_valid(&scope.output_retention_policy_id)
        || request.output_schema_id != scope.output_schema_id
        || request.output_retention_policy_id != scope.output_retention_policy_id
    {
        return ModelOutputDecision::OutputPolicyMismatch;
    }

    match request.validation {
        ModelOutputValidation::Validated => ModelOutputDecision::Authorized,
        ModelOutputValidation::Rejected => ModelOutputDecision::ValidationRejected,
    }
}
