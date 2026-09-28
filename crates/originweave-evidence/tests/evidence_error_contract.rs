use std::error::Error as _;

use originweave_evidence::EvidenceError;

#[test]
fn evidence_errors_have_a_stable_standard_error_contract() {
    let cases = [
        (EvidenceError::InvalidPath, "evidence path is invalid"),
        (EvidenceError::LimitExceeded, "evidence limit exceeded"),
        (EvidenceError::EmptyLocator, "evidence locator is empty"),
        (EvidenceError::InvalidHash, "evidence hash is invalid"),
        (
            EvidenceError::InvalidSourceUrl,
            "evidence source URL is invalid",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }

    fn assert_standard_error_contract<E: std::error::Error + Send + Sync + 'static>() {}
    assert_standard_error_contract::<EvidenceError>();
}
