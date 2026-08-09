use originweave_core::Origin;
use originweave_evidence::{
    MAX_SENSITIVE_FIELD_COUNT, SensitiveAccessClass, SensitiveAccessEvidence,
    SensitiveAccessEvidenceInput, SensitiveAccessOutcome, SensitiveEvidenceError,
};

type TestResult = Result<(), String>;

fn valid_input() -> Result<SensitiveAccessEvidenceInput, String> {
    let destination =
        Origin::parse("HTTPS://shipping.example:443").map_err(|error| format!("{error:?}"))?;
    Ok(SensitiveAccessEvidenceInput {
        request_id: "request:shipment:42".to_owned(),
        decision_id: "decision:shipment:42".to_owned(),
        tenant_id: "tenant:acme".to_owned(),
        actor_id: "workload:fulfillment".to_owned(),
        task_id: "task:ship-order-42".to_owned(),
        field_ids: vec![
            "customer.full_name".to_owned(),
            "shipping.address".to_owned(),
        ],
        purpose_id: "purpose:order-fulfillment".to_owned(),
        destination,
        classification: SensitiveAccessClass::PersonalData,
        outcome: SensitiveAccessOutcome::PartialFieldDisclosure,
        policy_version: "policy:2026-08".to_owned(),
        approval_reference: Some("approval:dual-control:7".to_owned()),
        decision_epoch_seconds: 1_786_176_000,
        disclosure_epoch_seconds: Some(1_786_176_001),
        retention_deadline_epoch_seconds: Some(1_786_262_400),
    })
}

fn validated(input: SensitiveAccessEvidenceInput) -> Result<SensitiveAccessEvidence, String> {
    SensitiveAccessEvidence::try_from(input).map_err(|error| format!("{error:?}"))
}

#[test]
fn exact_metadata_receipt_preserves_authority_without_protected_values() -> TestResult {
    let evidence = validated(valid_input()?)?;

    assert_eq!(evidence.request_id(), "request:shipment:42");
    assert_eq!(evidence.decision_id(), "decision:shipment:42");
    assert_eq!(evidence.tenant_id(), "tenant:acme");
    assert_eq!(evidence.actor_id(), "workload:fulfillment");
    assert_eq!(evidence.task_id(), "task:ship-order-42");
    assert_eq!(
        evidence.field_ids(),
        ["customer.full_name", "shipping.address"]
    );
    assert_eq!(evidence.purpose_id(), "purpose:order-fulfillment");
    assert_eq!(evidence.destination().as_str(), "https://shipping.example");
    assert_eq!(
        evidence.classification(),
        SensitiveAccessClass::PersonalData
    );
    assert_eq!(
        evidence.outcome(),
        SensitiveAccessOutcome::PartialFieldDisclosure
    );
    assert_eq!(evidence.policy_version(), "policy:2026-08");
    assert_eq!(
        evidence.approval_reference(),
        Some("approval:dual-control:7")
    );
    assert_eq!(evidence.decision_epoch_seconds(), 1_786_176_000);
    assert_eq!(evidence.disclosure_epoch_seconds(), Some(1_786_176_001));
    assert_eq!(
        evidence.retention_deadline_epoch_seconds(),
        Some(1_786_262_400)
    );
    Ok(())
}

#[test]
fn authority_identifiers_and_field_sets_are_bounded_and_unambiguous() -> TestResult {
    for invalid in [
        "",
        "tenant with spaces",
        "tenant/42",
        "테넌트:42",
        "tenant\n42",
    ] {
        let mut input = valid_input()?;
        input.tenant_id = invalid.to_owned();
        assert!(matches!(
            SensitiveAccessEvidence::try_from(input),
            Err(SensitiveEvidenceError::InvalidIdentifier)
        ));
    }

    let mut oversized = valid_input()?;
    oversized.actor_id = "a".repeat(129);
    assert!(matches!(
        SensitiveAccessEvidence::try_from(oversized),
        Err(SensitiveEvidenceError::InvalidIdentifier)
    ));

    let mut invalid_field = valid_input()?;
    invalid_field.field_ids = vec!["customer/email".to_owned()];
    assert!(matches!(
        SensitiveAccessEvidence::try_from(invalid_field),
        Err(SensitiveEvidenceError::InvalidFieldSet)
    ));

    let mut empty_fields = valid_input()?;
    empty_fields.field_ids.clear();
    assert!(matches!(
        SensitiveAccessEvidence::try_from(empty_fields),
        Err(SensitiveEvidenceError::InvalidFieldSet)
    ));

    let mut duplicate_fields = valid_input()?;
    duplicate_fields.field_ids = vec!["customer.email".to_owned(), "customer.email".to_owned()];
    assert!(matches!(
        SensitiveAccessEvidence::try_from(duplicate_fields),
        Err(SensitiveEvidenceError::InvalidFieldSet)
    ));

    let mut too_many_fields = valid_input()?;
    too_many_fields.field_ids = (0..=MAX_SENSITIVE_FIELD_COUNT)
        .map(|index| format!("field:{index}"))
        .collect();
    assert!(matches!(
        SensitiveAccessEvidence::try_from(too_many_fields),
        Err(SensitiveEvidenceError::InvalidFieldSet)
    ));

    let mut invalid_approval = valid_input()?;
    invalid_approval.approval_reference = Some("approval/7".to_owned());
    assert!(matches!(
        SensitiveAccessEvidence::try_from(invalid_approval),
        Err(SensitiveEvidenceError::InvalidIdentifier)
    ));

    let mut no_approval = valid_input()?;
    no_approval.approval_reference = None;
    assert!(SensitiveAccessEvidence::try_from(no_approval).is_ok());

    let mut no_retention_deadline = valid_input()?;
    no_retention_deadline.retention_deadline_epoch_seconds = None;
    let evidence = validated(no_retention_deadline)?;
    assert_eq!(evidence.retention_deadline_epoch_seconds(), None);
    Ok(())
}

#[test]
fn disclosure_and_retention_times_fail_closed_when_semantics_are_impossible() -> TestResult {
    let mut zero_decision = valid_input()?;
    zero_decision.decision_epoch_seconds = 0;
    assert!(matches!(
        SensitiveAccessEvidence::try_from(zero_decision),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    ));

    let mut disclosure_before_decision = valid_input()?;
    disclosure_before_decision.disclosure_epoch_seconds = Some(1_786_175_999);
    assert!(matches!(
        SensitiveAccessEvidence::try_from(disclosure_before_decision),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    ));

    let mut retention_before_disclosure = valid_input()?;
    retention_before_disclosure.retention_deadline_epoch_seconds = Some(1_786_176_000);
    assert!(matches!(
        SensitiveAccessEvidence::try_from(retention_before_disclosure),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    ));

    let mut denied_but_disclosed = valid_input()?;
    denied_but_disclosed.outcome = SensitiveAccessOutcome::DenyAccess;
    assert!(matches!(
        SensitiveAccessEvidence::try_from(denied_but_disclosed),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    ));

    let mut disclosure_without_time = valid_input()?;
    disclosure_without_time.outcome = SensitiveAccessOutcome::FullFieldDisclosure;
    disclosure_without_time.disclosure_epoch_seconds = None;
    assert!(matches!(
        SensitiveAccessEvidence::try_from(disclosure_without_time),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    ));
    Ok(())
}

#[test]
fn every_disclosure_and_control_outcome_has_consistent_lifecycle_semantics() -> TestResult {
    let outcomes = [
        SensitiveAccessOutcome::DenyAccess,
        SensitiveAccessOutcome::OpaqueHandleOnly,
        SensitiveAccessOutcome::DerivedValueOnly,
        SensitiveAccessOutcome::PartialFieldDisclosure,
        SensitiveAccessOutcome::FullFieldDisclosure,
        SensitiveAccessOutcome::HumanApprovalRequired,
        SensitiveAccessOutcome::DualControlRequired,
    ];

    for outcome in outcomes {
        let mut input = valid_input()?;
        input.outcome = outcome;
        input.disclosure_epoch_seconds = match outcome {
            SensitiveAccessOutcome::DerivedValueOnly
            | SensitiveAccessOutcome::PartialFieldDisclosure
            | SensitiveAccessOutcome::FullFieldDisclosure => Some(1_786_176_001),
            SensitiveAccessOutcome::DenyAccess
            | SensitiveAccessOutcome::OpaqueHandleOnly
            | SensitiveAccessOutcome::HumanApprovalRequired
            | SensitiveAccessOutcome::DualControlRequired => None,
        };
        let evidence = validated(input)?;
        assert_eq!(evidence.outcome(), outcome);
    }
    Ok(())
}

#[test]
fn every_documented_classification_is_representable() -> TestResult {
    let classifications = [
        SensitiveAccessClass::PublicData,
        SensitiveAccessClass::InternalData,
        SensitiveAccessClass::PersonalData,
        SensitiveAccessClass::SensitivePersonalData,
        SensitiveAccessClass::CredentialData,
        SensitiveAccessClass::PaymentData,
    ];

    for classification in classifications {
        let mut input = valid_input()?;
        input.classification = classification;
        let evidence = validated(input)?;
        assert_eq!(evidence.classification(), classification);
    }
    Ok(())
}

#[test]
fn sensitive_evidence_errors_integrate_with_standard_error_chains() {
    let cases = [
        (
            SensitiveEvidenceError::InvalidIdentifier,
            "invalid sensitive-access identifier",
        ),
        (
            SensitiveEvidenceError::InvalidFieldSet,
            "invalid sensitive-access field set",
        ),
        (
            SensitiveEvidenceError::InvalidLifecycle,
            "invalid sensitive-access lifecycle",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}
