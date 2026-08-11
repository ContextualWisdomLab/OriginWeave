use originweave_policy::{
    ModelOutputDecision, ModelOutputRequest, ModelOutputScope, ModelOutputValidation,
    evaluate_model_output,
};

const OUTPUT_SCHEMA_ID: &str = "customer-summary.v1";
const OUTPUT_RETENTION_POLICY_ID: &str = "task-local-retention";

fn request(validation: ModelOutputValidation) -> ModelOutputRequest {
    ModelOutputRequest::new(OUTPUT_SCHEMA_ID, OUTPUT_RETENTION_POLICY_ID, validation)
}

fn scope() -> ModelOutputScope {
    ModelOutputScope::new(OUTPUT_SCHEMA_ID, OUTPUT_RETENTION_POLICY_ID)
}

#[test]
fn validated_output_requires_an_exact_separate_output_policy() {
    assert_eq!(
        evaluate_model_output(&request(ModelOutputValidation::Validated), &scope()),
        ModelOutputDecision::Authorized
    );
}

#[test]
fn rejected_output_validation_cannot_be_authorized() {
    assert_eq!(
        evaluate_model_output(&request(ModelOutputValidation::Rejected), &scope()),
        ModelOutputDecision::ValidationRejected
    );
}

#[test]
fn output_schema_and_retention_policy_must_match_exactly() {
    let wrong_schema = ModelOutputRequest::new(
        "different-schema.v1",
        OUTPUT_RETENTION_POLICY_ID,
        ModelOutputValidation::Validated,
    );
    assert_eq!(
        evaluate_model_output(&wrong_schema, &scope()),
        ModelOutputDecision::OutputPolicyMismatch
    );

    let wrong_retention = ModelOutputRequest::new(
        OUTPUT_SCHEMA_ID,
        "different-retention",
        ModelOutputValidation::Validated,
    );
    assert_eq!(
        evaluate_model_output(&wrong_retention, &scope()),
        ModelOutputDecision::OutputPolicyMismatch
    );
}

#[test]
fn malformed_output_policy_identifiers_fail_closed_on_request_or_scope() {
    let malformed_requests = [
        ModelOutputRequest::new(
            "",
            OUTPUT_RETENTION_POLICY_ID,
            ModelOutputValidation::Validated,
        ),
        ModelOutputRequest::new(
            OUTPUT_SCHEMA_ID,
            "retention/policy",
            ModelOutputValidation::Validated,
        ),
    ];
    for malformed in malformed_requests {
        assert_eq!(
            evaluate_model_output(&malformed, &scope()),
            ModelOutputDecision::OutputPolicyMismatch
        );
    }

    let oversized_schema = "a".repeat(129);
    let malformed_scopes = [
        ModelOutputScope::new(&oversized_schema, OUTPUT_RETENTION_POLICY_ID),
        ModelOutputScope::new(OUTPUT_SCHEMA_ID, "retention\npolicy"),
    ];
    for malformed in malformed_scopes {
        assert_eq!(
            evaluate_model_output(&request(ModelOutputValidation::Validated), &malformed,),
            ModelOutputDecision::OutputPolicyMismatch
        );
    }
}

#[test]
fn policy_mismatch_precedes_validation_result() {
    let mismatched = ModelOutputRequest::new(
        "different-schema.v1",
        OUTPUT_RETENTION_POLICY_ID,
        ModelOutputValidation::Rejected,
    );
    assert_eq!(
        evaluate_model_output(&mismatched, &scope()),
        ModelOutputDecision::OutputPolicyMismatch
    );
}
