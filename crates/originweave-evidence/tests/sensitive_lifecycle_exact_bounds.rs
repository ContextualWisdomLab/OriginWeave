use originweave_core::Origin;
use originweave_evidence::{
    SensitiveAccessClass, SensitiveAccessEvidence, SensitiveAccessEvidenceInput,
    SensitiveAccessOutcome, SensitiveHandleLifecycleEvidence,
    SensitiveHandleLifecycleEvidenceInput,
};

type TestResult = Result<(), String>;

fn valid_access_evidence() -> Result<SensitiveAccessEvidence, String> {
    valid_access_evidence_with_retention(Some(1_720_003_600))
}

fn valid_access_evidence_with_retention(
    retention_deadline_epoch_seconds: Option<u64>,
) -> Result<SensitiveAccessEvidence, String> {
    let destination =
        Origin::parse("https://shipping.example").map_err(|error| format!("{error:?}"))?;
    SensitiveAccessEvidence::try_from(SensitiveAccessEvidenceInput {
        request_id: "request-42".to_owned(),
        decision_id: "decision-42".to_owned(),
        tenant_id: "tenant-7".to_owned(),
        actor_id: "workload-fulfillment".to_owned(),
        task_id: "task-42".to_owned(),
        field_ids: vec!["shipping.address".to_owned()],
        purpose_id: "fulfill-shipment".to_owned(),
        destination,
        classification: SensitiveAccessClass::PersonalData,
        outcome: SensitiveAccessOutcome::OpaqueHandleOnly,
        policy_version: "sensitive-policy-v3".to_owned(),
        approval_reference: None,
        decision_epoch_seconds: 1_720_000_000,
        disclosure_epoch_seconds: None,
        retention_deadline_epoch_seconds,
    })
    .map_err(|error| format!("{error:?}"))
}

fn valid_input() -> Result<SensitiveHandleLifecycleEvidenceInput, String> {
    Ok(SensitiveHandleLifecycleEvidenceInput {
        access_evidence: valid_access_evidence()?,
        issued_epoch_seconds: 1_720_000_001,
        expires_epoch_seconds: 1_720_000_301,
        maximum_uses: 2,
        resolution_count: 1,
        revoked_epoch_seconds: None,
    })
}

#[test]
fn one_second_handle_lifetime_is_preserved() -> TestResult {
    let mut input = valid_input()?;
    input.issued_epoch_seconds = 1_720_000_300;
    input.expires_epoch_seconds = 1_720_000_301;
    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(input).map_err(|error| format!("{error:?}"))?;
    assert_eq!(
        evidence.expires_epoch_seconds() - evidence.issued_epoch_seconds(),
        1
    );
    Ok(())
}

#[test]
fn maximum_epoch_lifecycle_is_preserved_without_overflow() -> TestResult {
    let mut input = valid_input()?;
    input.access_evidence = valid_access_evidence_with_retention(Some(u64::MAX))?;
    input.issued_epoch_seconds = u64::MAX - 1;
    input.expires_epoch_seconds = u64::MAX;
    input.revoked_epoch_seconds = Some(u64::MAX);
    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(input).map_err(|error| format!("{error:?}"))?;
    assert_eq!(evidence.issued_epoch_seconds(), u64::MAX - 1);
    assert_eq!(evidence.expires_epoch_seconds(), u64::MAX);
    assert_eq!(evidence.revoked_epoch_seconds(), Some(u64::MAX));
    assert!(evidence.is_revoked());
    Ok(())
}

#[test]
fn unused_handle_accepts_zero_resolutions() -> TestResult {
    let mut input = valid_input()?;
    input.resolution_count = 0;
    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(input).map_err(|error| format!("{error:?}"))?;
    assert_eq!(evidence.resolution_count(), 0);
    assert_eq!(evidence.maximum_uses(), 2);
    Ok(())
}

#[test]
fn maximum_use_count_and_exact_exhaustion_are_preserved() -> TestResult {
    let mut input = valid_input()?;
    input.maximum_uses = u32::MAX;
    input.resolution_count = u32::MAX;
    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(input).map_err(|error| format!("{error:?}"))?;
    assert_eq!(evidence.maximum_uses(), u32::MAX);
    assert_eq!(evidence.resolution_count(), u32::MAX);
    Ok(())
}

#[test]
fn revocation_at_exact_issue_time_is_retained() -> TestResult {
    let mut input = valid_input()?;
    input.revoked_epoch_seconds = Some(input.issued_epoch_seconds);
    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(input).map_err(|error| format!("{error:?}"))?;
    assert_eq!(
        evidence.revoked_epoch_seconds(),
        Some(evidence.issued_epoch_seconds())
    );
    assert!(evidence.is_revoked());
    Ok(())
}
