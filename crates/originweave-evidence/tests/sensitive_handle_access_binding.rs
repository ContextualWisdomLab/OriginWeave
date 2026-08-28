use originweave_core::Origin;
use originweave_evidence::{
    SensitiveAccessClass, SensitiveAccessEvidence, SensitiveAccessEvidenceInput,
    SensitiveAccessOutcome, SensitiveEvidenceError, SensitiveHandleLifecycleEvidence,
    SensitiveHandleLifecycleEvidenceInput,
};

type TestResult = Result<(), String>;

fn access_evidence(
    outcome: SensitiveAccessOutcome,
    decision_epoch_seconds: u64,
) -> Result<SensitiveAccessEvidence, String> {
    let destination =
        Origin::parse("https://checkout.example.com").map_err(|error| format!("{error:?}"))?;
    SensitiveAccessEvidence::try_from(SensitiveAccessEvidenceInput {
        request_id: "request-42".to_owned(),
        decision_id: "decision-42".to_owned(),
        tenant_id: "tenant-7".to_owned(),
        actor_id: "workload-browser-adapter".to_owned(),
        task_id: "task-99".to_owned(),
        field_ids: vec!["shipping_name".to_owned(), "shipping_address".to_owned()],
        purpose_id: "fulfill-shipment".to_owned(),
        destination,
        classification: SensitiveAccessClass::PersonalData,
        outcome,
        policy_version: "sensitive-policy-v3".to_owned(),
        approval_reference: None,
        decision_epoch_seconds,
        disclosure_epoch_seconds: None,
        retention_deadline_epoch_seconds: Some(decision_epoch_seconds + 3_600),
    })
    .map_err(|error| format!("{error:?}"))
}

fn lifecycle_input(
    access_evidence: SensitiveAccessEvidence,
    issued_epoch_seconds: u64,
) -> SensitiveHandleLifecycleEvidenceInput {
    SensitiveHandleLifecycleEvidenceInput {
        access_evidence,
        issued_epoch_seconds,
        expires_epoch_seconds: issued_epoch_seconds + 300,
        maximum_uses: 2,
        resolution_count: 0,
        revoked_epoch_seconds: None,
    }
}

#[test]
fn lifecycle_identity_retains_complete_opaque_handle_access_receipt() -> TestResult {
    let access = access_evidence(SensitiveAccessOutcome::OpaqueHandleOnly, 1_720_000_000)?;
    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(lifecycle_input(access.clone(), 1_720_000_001))
            .map_err(|error| format!("{error:?}"))?;

    assert_eq!(evidence.access_evidence(), &access);
    assert_eq!(evidence.request_id(), access.request_id());
    assert_eq!(evidence.decision_id(), access.decision_id());
    assert_eq!(evidence.access_evidence().tenant_id(), "tenant-7");
    assert_eq!(evidence.access_evidence().task_id(), "task-99");
    assert_eq!(
        evidence.access_evidence().field_ids(),
        ["shipping_name", "shipping_address"]
    );
    assert_eq!(
        evidence.access_evidence().destination().as_str(),
        "https://checkout.example.com"
    );
    Ok(())
}

#[test]
fn lifecycle_rejects_non_opaque_handle_access_decision() -> TestResult {
    let denied = access_evidence(SensitiveAccessOutcome::DenyAccess, 1_720_000_000)?;

    assert_eq!(
        SensitiveHandleLifecycleEvidence::try_from(lifecycle_input(denied, 1_720_000_001)),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    );
    Ok(())
}

#[test]
fn lifecycle_rejects_issuance_before_policy_decision() -> TestResult {
    let access = access_evidence(SensitiveAccessOutcome::OpaqueHandleOnly, 1_720_000_100)?;

    assert_eq!(
        SensitiveHandleLifecycleEvidence::try_from(lifecycle_input(access, 1_720_000_099)),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    );
    Ok(())
}

#[test]
fn lifecycle_expiry_respects_access_retention_deadline() -> TestResult {
    let access = access_evidence(SensitiveAccessOutcome::OpaqueHandleOnly, 1_720_000_000)?;
    let retention_deadline = access
        .retention_deadline_epoch_seconds()
        .ok_or_else(|| "fixture must carry a retention deadline".to_owned())?;

    let mut exact_deadline = lifecycle_input(access.clone(), 1_720_000_001);
    exact_deadline.expires_epoch_seconds = retention_deadline;
    SensitiveHandleLifecycleEvidence::try_from(exact_deadline)
        .map_err(|error| format!("{error:?}"))?;

    let mut after_deadline = lifecycle_input(access, 1_720_000_001);
    after_deadline.expires_epoch_seconds = retention_deadline + 1;
    assert_eq!(
        SensitiveHandleLifecycleEvidence::try_from(after_deadline),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    );
    Ok(())
}
