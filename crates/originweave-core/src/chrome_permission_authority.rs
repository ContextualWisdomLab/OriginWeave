//! Separation between Chrome extension compatibility permissions and Agent authority.

use crate::ActionKind;
use std::fmt;

/// Why a Chrome extension permission cannot authorize an OriginWeave Agent action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromePermissionAuthorityError {
    /// The permission names a reviewed Chrome compatibility surface, not Agent authority.
    CompatibilitySurfaceOnly,
    /// The permission is not a reviewed Chrome surface and still grants no Agent capability.
    UnrecognizedPermission,
}

impl fmt::Display for ChromePermissionAuthorityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CompatibilitySurfaceOnly => {
                "Chrome compatibility permission cannot authorize an OriginWeave Agent action"
            }
            Self::UnrecognizedPermission => {
                "Chrome permission is not a reviewed compatibility surface and cannot authorize an OriginWeave Agent action"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ChromePermissionAuthorityError {}

const REVIEWED_CHROME_COMPATIBILITY_PERMISSIONS: &[&str] = &[
    "bookmarks",
    "commands",
    "declarativeNetRequest",
    "declarativeNetRequestWithHostAccess",
    "downloads",
    "history",
    "nativeMessaging",
    "scripting",
    "sidePanel",
    "storage",
    "tabs",
    "windows",
];

/// Refuse to treat a Chrome extension permission as OriginWeave Agent authority.
///
/// A successful Chrome compatibility proof never becomes an OriginWeave Agent
/// capability. Adapters must keep browser compatibility evidence and explicit
/// OriginWeave grants separate and call this boundary before exposing a typed
/// action to policy. The action is accepted only to make that separation
/// explicit at the adapter boundary; no action kind can make this function
/// return success.
pub fn chrome_permission_authorizes_agent_action(
    permission: &str,
    _action: ActionKind,
) -> Result<(), ChromePermissionAuthorityError> {
    if !is_exact_chrome_permission_token(permission) {
        return Err(ChromePermissionAuthorityError::UnrecognizedPermission);
    }
    if REVIEWED_CHROME_COMPATIBILITY_PERMISSIONS.contains(&permission) {
        return Err(ChromePermissionAuthorityError::CompatibilitySurfaceOnly);
    }
    Err(ChromePermissionAuthorityError::UnrecognizedPermission)
}

fn is_exact_chrome_permission_token(permission: &str) -> bool {
    let mut characters = permission.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    first.is_ascii_lowercase() && characters.all(|character| character.is_ascii_alphabetic())
}
