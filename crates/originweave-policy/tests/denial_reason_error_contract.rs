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
            DenialReason::HumanModeNotAgentControlled,
            "human mode does not grant autonomous agent control",
        ),
        (
            DenialReason::ModePurposeMismatch,
            "session mode and execution purpose are incompatible",
        ),
        (
            DenialReason::UntrustedInstructionSource,
            "untrusted web content cannot authorize this action",
        ),
        (
            DenialReason::McpActionMismatch,
            "validated MCP route does not match the requested policy action",
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
            DenialReason::CrawlerMutation,
            "crawler mode forbids state-mutating actions",
        ),
        (
            DenialReason::CrossOriginMutation,
            "cross-origin mutation requires separately authorized source and target origins",
        ),
        (
            DenialReason::OriginNotWritable,
            "target origin is not writable under the current policy",
        ),
        (
            DenialReason::RobotsDisallowed,
            "robots policy denies this public crawl",
        ),
        (
            DenialReason::RobotsUnknown,
            "robots policy is unknown for this public crawl",
        ),
        (
            DenialReason::RobotsNotApplicable,
            "public crawl requires an applicable robots policy decision",
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
            DenialReason::ForbiddenRisk,
            "action risk class is not delegable",
        ),
        (
            DenialReason::ApprovalScopeMismatch,
            "approval evidence does not authorize this action scope",
        ),
    ];

    for (reason, expected) in cases {
        assert_eq!(reason.to_string(), expected);
        assert_source_free(&reason);
    }
}
