use std::error::Error as _;

use originweave_evidence::EvidenceError;

#[test]
fn evidence_error_has_stable_credential_safe_standard_error_contract() {
    let cases = [
        (
            EvidenceError::InvalidPath,
            "network evidence path is invalid",
        ),
        (
            EvidenceError::LimitExceeded,
            "network evidence exceeds a configured limit",
        ),
        (
            EvidenceError::EmptyLocator,
            "evidence source locator must not be empty",
        ),
        (
            EvidenceError::InvalidHash,
            "evidence source hash must be canonical lowercase SHA-256",
        ),
        (
            EvidenceError::InvalidSourceUrl,
            "evidence source URL is invalid",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
