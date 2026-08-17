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
        "scripting",
        "sidePanel",
        "declarativeNetRequest",
        "declarativeNetRequestWithHostAccess",
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
