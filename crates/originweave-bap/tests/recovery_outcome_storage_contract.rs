use originweave_bap::{
    BapExternalSideEffectOutcome, BapExternalSideEffectOutcomeParseError,
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
        (BapExternalSideEffectOutcome::UnknownOutcome, "unknown_outcome"),
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
