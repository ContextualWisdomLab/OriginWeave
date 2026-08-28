//! Cross-surface platform-coherence contracts for stealth presentations.
//!
//! The presentation platform, the JavaScript UA token, and the UA Client
//! Hints platform are three surfaces a page can reconcile into one identity.
//! If an adapter presents one platform in the static profile and a different
//! one in `navigator.userAgentData`, the contradiction is itself a
//! reidentification signal. This module binds the hints platform to the
//! presentation platform with a deterministic, fail-closed check. It
//! performs no evasion and never reads the host.

use crate::{PresentationPlatform, UaClientHints};
use std::error::Error;
use std::fmt;

/// A cross-surface coherence failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoherenceError {
    /// The UA Client Hints platform contradicted the presentation platform.
    HintsPlatformMismatch,
}

impl fmt::Display for CoherenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HintsPlatformMismatch => formatter
                .write_str("UA Client Hints platform contradicts the presentation platform"),
        }
    }
}

impl Error for CoherenceError {}

/// Require the presented UA Client Hints to agree with the presentation
/// platform.
///
/// The canonical hints token for `presentation` comes from
/// [`PresentationPlatform::hints_platform`]; any other hints platform fails
/// closed so an adapter cannot surface a contradicting identity.
pub fn require_hints_coherence(
    hints: &UaClientHints,
    presentation: PresentationPlatform,
) -> Result<(), CoherenceError> {
    if hints.platform() != presentation.hints_platform() {
        return Err(CoherenceError::HintsPlatformMismatch);
    }
    Ok(())
}
