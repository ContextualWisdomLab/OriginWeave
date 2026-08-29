#![allow(clippy::expect_used)]

use originweave_core::{
    BrowserSessionId, BrowsingContextId, ExtensionAccessDecision, ExtensionAccessRequest,
    ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId, Origin, evaluate_extension_access,
};

fn extension_id(value: &str) -> ExtensionId {
    ExtensionId::parse(value).expect("valid extension id")
}

fn session(value: u64) -> BrowserSessionId {
    BrowserSessionId::new(value).expect("nonzero browser session")
}

fn context(value: u64) -> BrowsingContextId {
    BrowsingContextId::new(value).expect("nonzero browsing context")
}

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("canonical origin")
}

const UNEXPIRED_NOW_EPOCH_SECONDS: u64 = 1_700_000_000;
const UNEXPIRED_EXPIRES_AT_EPOCH_SECONDS: u64 = 1_700_000_600;

#[test]
fn extension_id_accepts_only_canonical_chromium_extension_ids() {
    let canonical = "abcdefghijklmnopabcdefghijklmnop";
    assert_eq!(extension_id(canonical).as_str(), canonical);

    for invalid in [
        "",
        "abcdefghijklmnopabcdefghijklmno",
        "abcdefghijklmnopabcdefghijklmnopq",
        "ABCDEFGHIJKLMNOPABCDEFGHIJKLMNOP",
        "abcdefghijklmnopabcdefghijklmno0",
        "abcdefghijklmnopabcdefghijklmno-",
        "abcdefghijklmnopabcdefghijklmno\n",
        "abcdefghijklmnopabcdefghijklmnoπ",
    ] {
        assert!(
            ExtensionId::parse(invalid).is_err(),
            "unexpected id: {invalid:?}"
        );
    }
}

#[test]
fn extension_agent_access_requires_an_explicit_exact_grant() {
    let allowed_extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let other_extension = extension_id("bcdefghijklmnopabcdefghijklmnopa");
    let granted_origin = origin("https://app.example");
    let grant = ExtensionAgentGrant::new(
        allowed_extension.clone(),
        session(7),
        context(11),
        granted_origin.clone(),
        UNEXPIRED_EXPIRES_AT_EPOCH_SECONDS,
        [ExtensionAgentCapability::ObserveCurrentContext],
    );

    let exact = ExtensionAccessRequest::new(
        allowed_extension.clone(),
        session(7),
        context(11),
        granted_origin.clone(),
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&exact, Some(&grant)),
        ExtensionAccessDecision::Allow
    );

    let no_grant = evaluate_extension_access(&exact, None);
    assert_eq!(no_grant, ExtensionAccessDecision::DenyMissingGrant);

    let wrong_extension = ExtensionAccessRequest::new(
        other_extension,
        session(7),
        context(11),
        granted_origin.clone(),
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_extension, Some(&grant)),
        ExtensionAccessDecision::DenyExtensionMismatch
    );

    let wrong_session = ExtensionAccessRequest::new(
        allowed_extension.clone(),
        session(8),
        context(11),
        granted_origin.clone(),
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_session, Some(&grant)),
        ExtensionAccessDecision::DenyBrowserSessionMismatch
    );

    let wrong_context = ExtensionAccessRequest::new(
        allowed_extension.clone(),
        session(7),
        context(12),
        granted_origin.clone(),
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_context, Some(&grant)),
        ExtensionAccessDecision::DenyBrowsingContextMismatch
    );

    let wrong_origin = ExtensionAccessRequest::new(
        allowed_extension.clone(),
        session(7),
        context(11),
        origin("https://other.example"),
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_origin, Some(&grant)),
        ExtensionAccessDecision::DenyOriginMismatch
    );

    let wrong_port = ExtensionAccessRequest::new(
        allowed_extension,
        session(7),
        context(11),
        origin("https://app.example:8443"),
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_port, Some(&grant)),
        ExtensionAccessDecision::DenyOriginMismatch
    );
}

#[test]
fn chrome_permissions_never_imply_originweave_agent_capabilities() {
    let id = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let granted_origin = origin("https://mail.example");
    let grant = ExtensionAgentGrant::new(
        id.clone(),
        session(3),
        context(5),
        granted_origin.clone(),
        UNEXPIRED_EXPIRES_AT_EPOCH_SECONDS,
        [ExtensionAgentCapability::ObserveCurrentContext],
    );

    let propose_action = ExtensionAccessRequest::new(
        id,
        session(3),
        context(5),
        granted_origin,
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&propose_action, Some(&grant)),
        ExtensionAccessDecision::DenyCapabilityNotGranted
    );
}

#[test]
fn explicit_grant_can_authorize_multiple_bounded_agent_capabilities() {
    let id = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let granted_origin = origin("http://127.0.0.1:8080");
    let grant = ExtensionAgentGrant::new(
        id.clone(),
        session(13),
        context(17),
        granted_origin.clone(),
        UNEXPIRED_EXPIRES_AT_EPOCH_SECONDS,
        [
            ExtensionAgentCapability::ObserveCurrentContext,
            ExtensionAgentCapability::ProposeTypedAction,
        ],
    );

    for capability in [
        ExtensionAgentCapability::ObserveCurrentContext,
        ExtensionAgentCapability::ProposeTypedAction,
    ] {
        let request = ExtensionAccessRequest::new(
            id.clone(),
            session(13),
            context(17),
            granted_origin.clone(),
            UNEXPIRED_NOW_EPOCH_SECONDS,
            capability,
        );
        assert_eq!(
            evaluate_extension_access(&request, Some(&grant)),
            ExtensionAccessDecision::Allow
        );
    }
}

#[test]
fn expired_origin_bound_grant_cannot_be_reused_after_exclusive_deadline() {
    let id = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let granted_origin = origin("https://billing.example");
    let expires_at_epoch_seconds = 1_700_000_100;
    let grant = ExtensionAgentGrant::new(
        id.clone(),
        session(19),
        context(23),
        granted_origin.clone(),
        expires_at_epoch_seconds,
        [ExtensionAgentCapability::ObserveCurrentContext],
    );

    let before_deadline = ExtensionAccessRequest::new(
        id.clone(),
        session(19),
        context(23),
        granted_origin.clone(),
        expires_at_epoch_seconds - 1,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&before_deadline, Some(&grant)),
        ExtensionAccessDecision::Allow
    );

    let at_deadline = ExtensionAccessRequest::new(
        id.clone(),
        session(19),
        context(23),
        granted_origin.clone(),
        expires_at_epoch_seconds,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&at_deadline, Some(&grant)),
        ExtensionAccessDecision::DenyExpired
    );

    let after_deadline = ExtensionAccessRequest::new(
        id,
        session(19),
        context(23),
        granted_origin,
        expires_at_epoch_seconds + 1,
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&after_deadline, Some(&grant)),
        ExtensionAccessDecision::DenyExpired
    );
}
