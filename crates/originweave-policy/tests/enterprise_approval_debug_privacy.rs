#![allow(clippy::expect_used)]

use originweave_core::{ActionIntentDigest, ActionKind, ApprovalScope, Origin};
use originweave_policy::{ApprovalPrincipalRef, EnterpriseApprovalRequest};

const PRIVATE_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const PRIVATE_ISSUER: &str = "https://issuer-private.example";
const PRIVATE_ORIGIN: &str = "https://approval-private.example";

fn approval_scope() -> ApprovalScope {
    ApprovalScope::new(
        ActionKind::Purchase,
        Origin::parse(PRIVATE_ORIGIN).expect("test origin must be valid"),
        ActionIntentDigest::parse(PRIVATE_INTENT).expect("test intent digest must be valid"),
    )
}

fn principal(subject: &str) -> ApprovalPrincipalRef {
    ApprovalPrincipalRef::new(PRIVATE_ISSUER, subject).expect("test principal must be valid")
}

fn assert_debug_omits(debug_output: &str, private_values: &[&str]) {
    for private_value in private_values {
        assert!(
            !debug_output.contains(private_value),
            "Debug output must not disclose approval identity or scope values: {debug_output}"
        );
    }
}

#[test]
fn enterprise_approval_debug_omits_principal_and_scope_identity() {
    let maker_subject = "maker-private-subject";
    let checker_subject = "checker-private-subject";
    let maker = principal(maker_subject);

    assert_debug_omits(&format!("{maker:?}"), &[PRIVATE_ISSUER, maker_subject]);

    let scope = approval_scope();
    let mut request = EnterpriseApprovalRequest::new(scope.clone(), maker, 100, 200, 1)
        .expect("approval request must be valid");

    assert_debug_omits(
        &format!("{request:?}"),
        &[
            PRIVATE_ISSUER,
            maker_subject,
            PRIVATE_ORIGIN,
            PRIVATE_INTENT,
        ],
    );

    request
        .approve(principal(checker_subject), 110)
        .expect("approval must succeed");
    assert_debug_omits(
        &format!("{request:?}"),
        &[
            PRIVATE_ISSUER,
            maker_subject,
            checker_subject,
            PRIVATE_ORIGIN,
            PRIVATE_INTENT,
        ],
    );

    let approval_use = request
        .consume(&scope, 120)
        .expect("approval use must be consumable");
    assert_debug_omits(
        &format!("{approval_use:?}"),
        &[PRIVATE_ORIGIN, PRIVATE_INTENT],
    );
}
