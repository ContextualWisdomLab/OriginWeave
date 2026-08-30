use originweave_bap::{
    BapExternalSideEffectOutcome, BapExternalSideEffectOutcomeParseError, BapRecoveryAction,
    BapRecoveryActionParseError,
};

#[test]
fn recovery_outcome_has_a_canonical_storage_neutral_round_trip() {
    let cases = [
        (
            BapExternalSideEffectOutcome::ConfirmedNoSideEffect,
            "confirmed_no_side_effect",
        ),
        (
            BapExternalSideEffectOutcome::ConfirmedSideEffect,
            "confirmed_side_effect",
        ),
        (
            BapExternalSideEffectOutcome::UnknownOutcome,
            "unknown_outcome",
        ),
        (
            BapExternalSideEffectOutcome::ReconciliationRequired,
            "reconciliation_required",
        ),
    ];

    for (outcome, encoded) in cases {
        assert_eq!(outcome.as_str(), encoded);
        assert_eq!(BapExternalSideEffectOutcome::parse(encoded), Ok(outcome));
    }
}

#[test]
fn recovery_outcome_parser_rejects_unknown_or_noncanonical_values() {
    for invalid in [
        "",
        "confirmed-no-side-effect",
        "ConfirmedNoSideEffect",
        "unknown",
        "reconciliation_required ",
    ] {
        assert_eq!(
            BapExternalSideEffectOutcome::parse(invalid),
            Err(BapExternalSideEffectOutcomeParseError::UnsupportedValue)
        );
    }

    let error = BapExternalSideEffectOutcomeParseError::UnsupportedValue;
    assert_eq!(
        error.to_string(),
        "BAP external side-effect outcome has an unsupported canonical value"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn recovery_action_has_a_canonical_storage_neutral_round_trip() {
    let cases = [
        (
            BapRecoveryAction::RevalidateBeforeRedispatch,
            "revalidate_before_redispatch",
        ),
        (
            BapRecoveryAction::VerifyConfirmedSideEffect,
            "verify_confirmed_side_effect",
        ),
        (
            BapRecoveryAction::ReconcileBeforeFurtherAction,
            "reconcile_before_further_action",
        ),
    ];

    for (action, encoded) in cases {
        assert_eq!(action.as_str(), encoded);
        assert_eq!(BapRecoveryAction::parse(encoded), Ok(action));
    }
}

#[test]
fn recovery_action_parser_rejects_unknown_or_noncanonical_values() {
    for invalid in [
        "",
        "revalidate-before-redispatch",
        "RevalidateBeforeRedispatch",
        "retry",
        "reconcile_before_further_action ",
    ] {
        assert_eq!(
            BapRecoveryAction::parse(invalid),
            Err(BapRecoveryActionParseError::UnsupportedValue)
        );
    }

    let error = BapRecoveryActionParseError::UnsupportedValue;
    assert_eq!(
        error.to_string(),
        "BAP recovery action has an unsupported canonical value"
    );
    assert!(std::error::Error::source(&error).is_none());
}
