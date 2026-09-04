use originweave_evidence::{CaptureLifecycleError, CaptureLifecycleState};

#[test]
fn capture_lifecycle_state_has_a_canonical_storage_neutral_round_trip() {
    let cases = [
        (CaptureLifecycleState::CaptureStarted, "capture_started"),
        (CaptureLifecycleState::CaptureCompleted, "capture_completed"),
        (CaptureLifecycleState::Verified, "verified"),
        (CaptureLifecycleState::Retained, "retained"),
        (CaptureLifecycleState::LegalHold, "legal_hold"),
        (
            CaptureLifecycleState::DeletionRequested,
            "deletion_requested",
        ),
        (CaptureLifecycleState::Deleted, "deleted"),
    ];

    for (state, encoded) in cases {
        assert_eq!(state.as_str(), encoded);
        assert_eq!(CaptureLifecycleState::parse(encoded), Ok(state));
    }
}

#[test]
fn capture_lifecycle_state_parser_rejects_aliases_and_ambiguous_values() {
    for invalid in [
        "",
        "CaptureStarted",
        "capture-started",
        "capture_started ",
        "legalhold",
        "deletion_pending",
        "DELETED",
    ] {
        assert_eq!(
            CaptureLifecycleState::parse(invalid),
            Err(CaptureLifecycleError::InvalidRestoredState)
        );
    }

    let error = CaptureLifecycleState::parse("future_state")
        .err()
        .unwrap_or(CaptureLifecycleError::InvalidTransition);
    assert_eq!(
        error.to_string(),
        "persisted capture lifecycle state is internally inconsistent"
    );
    assert!(std::error::Error::source(&error).is_none());
}
