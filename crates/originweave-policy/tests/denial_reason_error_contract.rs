use std::error::Error;

use originweave_core::Capability;
use originweave_policy::DenialReason;

fn assert_source_free(error: &(dyn Error + 'static)) {
    assert!(error.source().is_none());
}

#[test]
fn denial_reasons_expose_stable_standard_error_contracts() {
    let cases = [
        (
            DenialReason::HumanMode,
            "human mode does not permit agent execution",
        ),
        (
            DenialReason::ModePurposeMismatch,
            "session mode and execution purpose are incompatible",
        ),
        (
            DenialReason::CrawlerMutationForbidden,
            "crawler mode forbids state-mutating actions",
        ),
        (
            DenialReason::RobotsDenied,
            "robots policy denies this crawl action",
        ),
        (
            DenialReason::RobotsUnknown,
            "robots policy is unknown for this crawl action",
        ),
        (
            DenialReason::UntrustedInstructionSource,
            "untrusted web content cannot authorize this action",
        ),
        (
            DenialReason::MissingCapability(Capability::Navigate),
            "required action capability is missing",
        ),
        (
            DenialReason::OriginNotReadable,
            "target origin is not readable under the current policy",
        ),
        (
            DenialReason::OriginNotWritable,
            "target origin is not writable under the current policy",
        ),
        (
            DenialReason::CrossOriginMutation,
            "cross-origin mutation requires separately authorized source and target origins",
        ),
        (
            DenialReason::SecretBrokerRequired,
            "secret-bearing actions require opaque broker delivery",
        ),
        (
            DenialReason::UnexpectedSecretMaterial,
            "non-secret action cannot carry secret material",
        ),
        (
            DenialReason::SecretNotAllowedForPurpose,
            "crawler and public-crawl purposes cannot use secret material",
        ),
        (
            DenialReason::LegalConsentForbidden,
            "legal consent cannot be delegated to an agent",
        ),
    ];

    for (reason, expected) in cases {
        assert_eq!(reason.to_string(), expected);
        assert_source_free(&reason);
    }
}
