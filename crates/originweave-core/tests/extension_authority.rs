#![allow(clippy::expect_used)]

use originweave_core::{
    ActionKind, BrowserSessionId, BrowsingContextId, ChromePermissionAuthorityError,
    ExtensionAccessDecision, ExtensionAccessRequest, ExtensionAgentCapability, ExtensionAgentGrant,
    ExtensionId, chrome_permission_authorizes_agent_action, evaluate_extension_access,
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
    let grant = ExtensionAgentGrant::new(
        allowed_extension.clone(),
        session(7),
        context(11),
        [ExtensionAgentCapability::ObserveCurrentContext],
    );

    let exact = ExtensionAccessRequest::new(
        allowed_extension.clone(),
        session(7),
        context(11),
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
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_session, Some(&grant)),
        ExtensionAccessDecision::DenyBrowserSessionMismatch
    );

    let wrong_context = ExtensionAccessRequest::new(
        allowed_extension,
        session(7),
        context(12),
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_context, Some(&grant)),
        ExtensionAccessDecision::DenyBrowsingContextMismatch
    );
}

#[test]
fn chrome_permissions_never_imply_originweave_agent_capabilities() {
    let id = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let grant = ExtensionAgentGrant::new(
        id.clone(),
        session(3),
        context(5),
        [ExtensionAgentCapability::ObserveCurrentContext],
    );

    let propose_action = ExtensionAccessRequest::new(
        id,
        session(3),
        context(5),
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
    let grant = ExtensionAgentGrant::new(
        id.clone(),
        session(13),
        context(17),
        [
            ExtensionAgentCapability::ObserveCurrentContext,
            ExtensionAgentCapability::ProposeTypedAction,
        ],
    );

    for capability in [
        ExtensionAgentCapability::ObserveCurrentContext,
        ExtensionAgentCapability::ProposeTypedAction,
    ] {
        let request = ExtensionAccessRequest::new(id.clone(), session(13), context(17), capability);
        assert_eq!(
            evaluate_extension_access(&request, Some(&grant)),
            ExtensionAccessDecision::Allow
        );
    }
}

#[test]
fn chrome_downloads_permission_cannot_authorize_agent_download() {
    assert_eq!(
        chrome_permission_authorizes_agent_action("downloads", ActionKind::Download),
        Err(ChromePermissionAuthorityError::CompatibilitySurfaceOnly)
    );
    assert_eq!(
        chrome_permission_authorizes_agent_action("bookmarks", ActionKind::Download),
        Err(ChromePermissionAuthorityError::CompatibilitySurfaceOnly)
    );
    assert_eq!(
        chrome_permission_authorizes_agent_action("history", ActionKind::Download),
        Err(ChromePermissionAuthorityError::CompatibilitySurfaceOnly)
    );
    assert_eq!(
        chrome_permission_authorizes_agent_action("DOWNLOADS", ActionKind::Download),
        Err(ChromePermissionAuthorityError::UnrecognizedPermission)
    );
    assert_eq!(
        chrome_permission_authorizes_agent_action("", ActionKind::Download),
        Err(ChromePermissionAuthorityError::UnrecognizedPermission)
    );
    assert_eq!(
        chrome_permission_authorizes_agent_action(
            "downloads\nhttps://example.invalid",
            ActionKind::Navigate
        ),
        Err(ChromePermissionAuthorityError::UnrecognizedPermission)
    );
    assert_eq!(
        chrome_permission_authorizes_agent_action("cookies", ActionKind::Download),
        Err(ChromePermissionAuthorityError::UnrecognizedPermission)
    );
}
