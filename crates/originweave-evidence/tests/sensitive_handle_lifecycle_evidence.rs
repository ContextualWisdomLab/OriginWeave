use originweave_core::Origin;
use originweave_evidence::{
    SensitiveAccessClass, SensitiveAccessEvidence, SensitiveAccessEvidenceInput,
    SensitiveAccessOutcome, SensitiveEvidenceError, SensitiveHandleLifecycleEvidence,
    SensitiveHandleLifecycleEvidenceInput,
};

type TestResult = Result<(), String>;

fn valid_access_evidence() -> Result<SensitiveAccessEvidence, String> {
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
        retention_deadline_epoch_seconds: Some(1_720_003_600),
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
fn records_bounded_handle_lifecycle_without_handle_or_secret_material() -> TestResult {
    let evidence = SensitiveHandleLifecycleEvidence::try_from(valid_input()?)
        .map_err(|error| format!("{error:?}"))?;

    assert_eq!(evidence.request_id(), "request-42");
    assert_eq!(evidence.decision_id(), "decision-42");
    assert_eq!(evidence.issued_epoch_seconds(), 1_720_000_001);
    assert_eq!(evidence.expires_epoch_seconds(), 1_720_000_301);
    assert_eq!(evidence.maximum_uses(), 2);
    assert_eq!(evidence.resolution_count(), 1);
    assert_eq!(evidence.revoked_epoch_seconds(), None);
    assert!(!evidence.is_revoked());

    let debug = format!("{evidence:?}");
    assert!(!debug.contains("opaque-handle-token-should-never-be-evidence"));
    assert!(!debug.contains("raw-secret-should-never-be-evidence"));
    Ok(())
}

#[test]
fn records_revocation_time_without_storing_revocation_payloads() -> TestResult {
    let mut input = valid_input()?;
    input.revoked_epoch_seconds = Some(1_720_000_120);
    input.resolution_count = 2;

    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(input).map_err(|error| format!("{error:?}"))?;

    assert_eq!(evidence.revoked_epoch_seconds(), Some(1_720_000_120));
    assert!(evidence.is_revoked());
    assert_eq!(evidence.resolution_count(), evidence.maximum_uses());
    Ok(())
}

#[test]
fn records_revocation_at_exact_expiry_boundary() -> TestResult {
    let mut input = valid_input()?;
    input.revoked_epoch_seconds = Some(input.expires_epoch_seconds);

    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(input).map_err(|error| format!("{error:?}"))?;

    assert_eq!(
        evidence.revoked_epoch_seconds(),
        Some(evidence.expires_epoch_seconds())
    );
    assert!(evidence.is_revoked());
    Ok(())
}

#[test]
fn rejects_zero_or_non_increasing_handle_lifetime() -> TestResult {
    for (issued, expires) in [
        (0, 1_720_000_301),
        (1_720_000_301, 1_720_000_301),
        (1_720_000_302, 1_720_000_301),
    ] {
        let mut input = valid_input()?;
        input.issued_epoch_seconds = issued;
        input.expires_epoch_seconds = expires;
        assert_eq!(
            SensitiveHandleLifecycleEvidence::try_from(input),
            Err(SensitiveEvidenceError::InvalidLifecycle)
        );
    }
    Ok(())
}

#[test]
fn rejects_zero_use_limit_or_resolution_count_above_limit() -> TestResult {
    let mut zero_limit = valid_input()?;
    zero_limit.maximum_uses = 0;
    assert_eq!(
        SensitiveHandleLifecycleEvidence::try_from(zero_limit),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    );

    let mut overused = valid_input()?;
    overused.resolution_count = overused.maximum_uses + 1;
    assert_eq!(
        SensitiveHandleLifecycleEvidence::try_from(overused),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    );
    Ok(())
}

#[test]
fn rejects_revocation_before_issue_or_after_expiry() -> TestResult {
    for revoked in [1_720_000_000, 1_720_000_302] {
        let mut input = valid_input()?;
        input.revoked_epoch_seconds = Some(revoked);
        assert_eq!(
            SensitiveHandleLifecycleEvidence::try_from(input),
            Err(SensitiveEvidenceError::InvalidLifecycle)
        );
    }
    Ok(())
}
