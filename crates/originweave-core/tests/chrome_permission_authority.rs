use originweave_core::{
    ActionKind, ChromePermissionAuthorityError, chrome_permission_authorizes_agent_action,
};

#[test]
fn chrome_compatibility_permissions_never_mint_agent_authority() {
    for permission in [
        "downloads",
        "bookmarks",
        "history",
        "storage",
        "tabs",
        "windows",
        "scripting",
        "commands",
        "sidePanel",
        "declarativeNetRequest",
        "declarativeNetRequestWithHostAccess",
        "nativeMessaging",
    ] {
        assert_eq!(
            chrome_permission_authorizes_agent_action(permission, ActionKind::Download),
            Err(ChromePermissionAuthorityError::CompatibilitySurfaceOnly)
        );
    }
}

#[test]
fn malformed_or_unreviewed_chrome_permissions_remain_unrecognized() {
    for permission in [
        "",
        "DOWNLOADS",
        "downloads\nhttps://example.invalid",
        "cookies",
        "downloads ",
    ] {
        assert_eq!(
            chrome_permission_authorizes_agent_action(permission, ActionKind::Download),
            Err(ChromePermissionAuthorityError::UnrecognizedPermission)
        );
    }
}

#[test]
fn chrome_permission_authority_errors_are_standard_credential_safe_errors() {
    let cases = [
        (
            ChromePermissionAuthorityError::CompatibilitySurfaceOnly,
            "Chrome compatibility permission cannot authorize an OriginWeave Agent action",
        ),
        (
            ChromePermissionAuthorityError::UnrecognizedPermission,
            "Chrome permission is not a reviewed compatibility surface and cannot authorize an OriginWeave Agent action",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        assert!(std::error::Error::source(&error).is_none());
    }
}
