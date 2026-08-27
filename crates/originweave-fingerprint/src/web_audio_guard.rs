//! Default-deny Web Audio fingerprinting policy and deterministic guard asset.
//!
//! Web Audio exposes implementation-specific timing and digital-signal-
//! processing behavior that a page can combine into a device fingerprint. This
//! module binds exact-origin exceptions to a reviewed pre-document guard rather
//! than copying host audio characteristics or injecting random noise.

use originweave_core::Origin;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

const MAX_ALLOWED_ORIGINS: usize = 128;
const ALLOWLIST_MARKER: &str = "/* ORIGINWEAVE_ALLOWED_WEB_AUDIO_ORIGINS */";
const GUARD_SCRIPT_TEMPLATE: &str =
    include_str!("../../../extensions/originweave-privacy-guard/web_audio_guard.js");

/// The result of evaluating one page origin against the Web Audio policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebAudioDecision {
    /// Web Audio construction must be blocked because no exact grant exists.
    BlockFingerprinting,
    /// A trusted profile explicitly granted the exact canonical origin.
    AllowExplicitOrigin,
}

impl WebAudioDecision {
    /// Return whether the privacy guard must block Web Audio constructors.
    #[must_use]
    pub const fn blocks_fingerprinting(self) -> bool {
        matches!(self, Self::BlockFingerprinting)
    }

    /// Return the stable credential-free denial reason for audit evidence.
    #[must_use]
    pub const fn reason_code(self) -> Option<&'static str> {
        match self {
            Self::BlockFingerprinting => Some("web_audio_fingerprinting_no_explicit_origin_grant"),
            Self::AllowExplicitOrigin => None,
        }
    }
}

/// A bounded Web Audio privacy-policy configuration failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebAudioPolicyError {
    /// The canonical allowlist exceeded its reviewed unique-origin ceiling.
    TooManyAllowedOrigins {
        /// Maximum number of unique canonical origins permitted by the policy.
        maximum: usize,
        /// Actual number of unique canonical origins supplied by the caller.
        actual: usize,
    },
}

impl fmt::Display for WebAudioPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyAllowedOrigins { maximum, actual } => write!(
                formatter,
                "web audio allowlist contains {actual} unique origins; maximum is {maximum}"
            ),
        }
    }
}

impl Error for WebAudioPolicyError {}

/// An immutable exact-origin policy for the reviewed Web Audio guard.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WebAudioFingerprintPolicy {
    allowed_origins: BTreeSet<Origin>,
}

impl WebAudioFingerprintPolicy {
    /// Build a policy from canonical origins, deduplicating before bounding it.
    pub fn new(allowed_origins: Vec<Origin>) -> Result<Self, WebAudioPolicyError> {
        let allowed_origins = allowed_origins.into_iter().collect::<BTreeSet<_>>();
        if allowed_origins.len() > MAX_ALLOWED_ORIGINS {
            return Err(WebAudioPolicyError::TooManyAllowedOrigins {
                maximum: MAX_ALLOWED_ORIGINS,
                actual: allowed_origins.len(),
            });
        }
        Ok(Self { allowed_origins })
    }

    /// Evaluate one exact canonical origin without subdomain or port widening.
    #[must_use]
    pub fn decision(&self, origin: &Origin) -> WebAudioDecision {
        if self.allowed_origins.contains(origin) {
            WebAudioDecision::AllowExplicitOrigin
        } else {
            WebAudioDecision::BlockFingerprinting
        }
    }

    /// Return the number of unique exact-origin grants in this policy.
    #[must_use]
    pub fn allowed_origin_count(&self) -> usize {
        self.allowed_origins.len()
    }

    /// Render the reviewed MAIN-world `document_start` guard deterministically.
    ///
    /// [`Origin`] admits only canonical scheme/authority strings, so each value
    /// is safe to place inside the generated JSON string literal without path,
    /// quote, backslash, control-character, or user-information ambiguity.
    #[must_use]
    pub fn render_guard_script(&self) -> String {
        let rendered_origins = self
            .allowed_origins
            .iter()
            .map(|origin| format!("    \"{}\"", origin.as_str()))
            .collect::<Vec<_>>()
            .join(",\n");
        GUARD_SCRIPT_TEMPLATE.replacen(ALLOWLIST_MARKER, &rendered_origins, 1)
    }
}
