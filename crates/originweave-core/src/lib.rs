//! Shared security and governance contracts for OriginWeave.
//!
//! The crate deliberately contains no browser-engine integration. It defines
//! small, deterministic value types that can be reused by the browser shell,
//! headless runtime, MCP adapter, and enterprise policy service.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;

#[path = "base.rs"]
mod base;
pub use base::*;

impl fmt::Display for NativeMessagingHostNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHostName => formatter.write_str(
                "native-messaging host name violates the reviewed Chrome identity syntax",
            ),
        }
    }
}

impl std::error::Error for NativeMessagingHostNameError {}

/// Browser control surface represented by assurance evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserAttachmentKind {
    /// OriginWeave is attached to an existing person-controlled browser tab.
    AttachedHumanTab,
    /// OriginWeave operates in a task-isolated browser profile.
    IsolatedProfile,
}

/// Trusted adapter evidence about extension influence on page state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionInfluenceEvidence {
    /// The trusted adapter established that an extension can influence page state.
    CanInfluencePageState,
    /// This bounded rule has no trusted evidence of extension influence.
    /// Absence of known influence is not proof that extensions are absent or unable to interfere.
    NoKnownExtensionInfluence,
}

/// A specific reason that one browser context has reduced assurance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReducedAssuranceReason {
    /// An attached human tab can be influenced by an existing browser extension.
    AttachedTabExtensionInfluence,
}

/// Classify the narrow attached-tab extension-influence assurance reduction.
///
/// `None` means only that this rule did not identify this specific reduction. It
/// is not evidence of full trust, extension absence, or high assurance. A future
/// trusted Chromium adapter must supply the attachment and influence evidence and
/// evaluate any other applicable assurance rules separately.
#[must_use]
pub const fn classify_reduced_assurance(
    attachment: BrowserAttachmentKind,
    extension_influence: ExtensionInfluenceEvidence,
) -> Option<ReducedAssuranceReason> {
    if matches!(
        (attachment, extension_influence),
        (
            BrowserAttachmentKind::AttachedHumanTab,
            ExtensionInfluenceEvidence::CanInfluencePageState
        )
    ) {
        Some(ReducedAssuranceReason::AttachedTabExtensionInfluence)
    } else {
        None
    }
}
