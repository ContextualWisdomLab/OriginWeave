use originweave_evidence::{
    MAX_SENSITIVE_IDENTIFIER_BYTES, SensitiveEvidenceError, SensitiveModelDisclosureEvidence,
    SensitiveModelDisclosureEvidenceInput,
};

fn valid_input() -> SensitiveModelDisclosureEvidenceInput {
    SensitiveModelDisclosureEvidenceInput {
        request_id: "request-42".to_owned(),
        decision_id: "decision-42".to_owned(),
        provider_id: "provider-private".to_owned(),
        model_id: "model-reviewed-v1".to_owned(),
        region_id: "kr-central".to_owned(),
        retention_policy_id: "ephemeral-retention".to_owned(),
        training_policy_id: "no-training".to_owned(),
        subprocessor_policy_id: "subprocessors-reviewed-v1".to_owned(),
        export_policy_id: "no-export".to_owned(),
    }
}

#[test]
fn records_exact_model_route_policy_without_protected_values() -> Result<(), SensitiveEvidenceError>
{
    let evidence = SensitiveModelDisclosureEvidence::try_from(valid_input())?;

    assert_eq!(evidence.request_id(), "request-42");
    assert_eq!(evidence.decision_id(), "decision-42");
    assert_eq!(evidence.provider_id(), "provider-private");
    assert_eq!(evidence.model_id(), "model-reviewed-v1");
    assert_eq!(evidence.region_id(), "kr-central");
    assert_eq!(evidence.retention_policy_id(), "ephemeral-retention");
    assert_eq!(evidence.training_policy_id(), "no-training");
    assert_eq!(
        evidence.subprocessor_policy_id(),
        "subprocessors-reviewed-v1"
    );
    assert_eq!(evidence.export_policy_id(), "no-export");

    let debug = format!("{evidence:?}");
    assert!(!debug.contains("protected-value-must-never-enter-evidence"));
    assert!(!debug.contains("provider-credential-must-never-enter-evidence"));
    Ok(())
}

#[test]
fn rejects_invalid_model_route_evidence_identifiers() {
    for field in 0..9 {
        let mut input = valid_input();
        let invalid = if field == 8 {
            "a".repeat(MAX_SENSITIVE_IDENTIFIER_BYTES + 1)
        } else {
            "bad/value".to_owned()
        };
        match field {
            0 => input.request_id = invalid,
            1 => input.decision_id = invalid,
            2 => input.provider_id = invalid,
            3 => input.model_id = invalid,
            4 => input.region_id = invalid,
            5 => input.retention_policy_id = invalid,
            6 => input.training_policy_id = invalid,
            7 => input.subprocessor_policy_id = invalid,
            8 => input.export_policy_id = invalid,
            _ => unreachable!(),
        }
        assert_eq!(
            SensitiveModelDisclosureEvidence::try_from(input),
            Err(SensitiveEvidenceError::InvalidIdentifier)
        );
    }
}

#[test]
fn identifiers_require_meaningful_ascii_policy_tokens() {
    for invalid in [String::new(), "---".to_owned(), "with space".to_owned()] {
        let mut input = valid_input();
        input.provider_id = invalid;
        assert_eq!(
            SensitiveModelDisclosureEvidence::try_from(input),
            Err(SensitiveEvidenceError::InvalidIdentifier)
        );
    }
}
