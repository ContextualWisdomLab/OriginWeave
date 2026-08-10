use originweave_evidence::{
    SensitiveEvidenceError, SensitiveHandleLifecycleEvidence, SensitiveHandleLifecycleEvidenceInput,
};

fn valid_input() -> SensitiveHandleLifecycleEvidenceInput {
    SensitiveHandleLifecycleEvidenceInput {
        request_id: "request-42".to_owned(),
        decision_id: "decision-42".to_owned(),
        issued_epoch_seconds: 1_720_000_000,
        expires_epoch_seconds: 1_720_000_300,
        maximum_uses: 2,
        resolution_count: 1,
        revoked_epoch_seconds: None,
    }
}

#[test]
fn records_bounded_handle_lifecycle_without_handle_or_secret_material() {
    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(valid_input()).expect("valid evidence");

    assert_eq!(evidence.request_id(), "request-42");
    assert_eq!(evidence.decision_id(), "decision-42");
    assert_eq!(evidence.issued_epoch_seconds(), 1_720_000_000);
    assert_eq!(evidence.expires_epoch_seconds(), 1_720_000_300);
    assert_eq!(evidence.maximum_uses(), 2);
    assert_eq!(evidence.resolution_count(), 1);
    assert_eq!(evidence.revoked_epoch_seconds(), None);
    assert!(!evidence.is_revoked());

    let debug = format!("{evidence:?}");
    assert!(!debug.contains("opaque-handle-token-should-never-be-evidence"));
    assert!(!debug.contains("raw-secret-should-never-be-evidence"));
}

#[test]
fn records_revocation_time_without_storing_revocation_payloads() {
    let mut input = valid_input();
    input.revoked_epoch_seconds = Some(1_720_000_120);
    input.resolution_count = 2;

    let evidence =
        SensitiveHandleLifecycleEvidence::try_from(input).expect("valid revoked evidence");

    assert_eq!(evidence.revoked_epoch_seconds(), Some(1_720_000_120));
    assert!(evidence.is_revoked());
    assert_eq!(evidence.resolution_count(), evidence.maximum_uses());
}

#[test]
fn rejects_invalid_request_or_decision_identifiers() {
    for mutate in [0_u8, 1_u8] {
        let mut input = valid_input();
        if mutate == 0 {
            input.request_id = "bad/request".to_owned();
        } else {
            input.decision_id = String::new();
        }
        assert_eq!(
            SensitiveHandleLifecycleEvidence::try_from(input),
            Err(SensitiveEvidenceError::InvalidIdentifier)
        );
    }
}

#[test]
fn rejects_zero_or_non_increasing_handle_lifetime() {
    for (issued, expires) in [
        (0, 1_720_000_300),
        (1_720_000_300, 1_720_000_300),
        (1_720_000_301, 1_720_000_300),
    ] {
        let mut input = valid_input();
        input.issued_epoch_seconds = issued;
        input.expires_epoch_seconds = expires;
        assert_eq!(
            SensitiveHandleLifecycleEvidence::try_from(input),
            Err(SensitiveEvidenceError::InvalidLifecycle)
        );
    }
}

#[test]
fn rejects_zero_use_limit_or_resolution_count_above_limit() {
    let mut zero_limit = valid_input();
    zero_limit.maximum_uses = 0;
    assert_eq!(
        SensitiveHandleLifecycleEvidence::try_from(zero_limit),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    );

    let mut overused = valid_input();
    overused.resolution_count = overused.maximum_uses + 1;
    assert_eq!(
        SensitiveHandleLifecycleEvidence::try_from(overused),
        Err(SensitiveEvidenceError::InvalidLifecycle)
    );
}

#[test]
fn rejects_revocation_outside_handle_lifetime() {
    for revoked in [1_719_999_999, 1_720_000_301] {
        let mut input = valid_input();
        input.revoked_epoch_seconds = Some(revoked);
        assert_eq!(
            SensitiveHandleLifecycleEvidence::try_from(input),
            Err(SensitiveEvidenceError::InvalidLifecycle)
        );
    }
}
