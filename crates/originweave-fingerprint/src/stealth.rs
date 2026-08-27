//! Bounded stealth-normalization surfaces for browser presentation.
//!
//! A page can observe rendered and media surfaces that carry more entropy
//! than static profile fields: canvas readback noise, WebGL renderer tokens,
//! Web Audio sample-rate reporting, and WebRTC interface exposure. The W3C
//! Fingerprinting Guidance prefers standardized, bounded values over
//! independent per-session randomization, and longitudinal fingerprint
//! research shows that renderer and audio surfaces are strong
//! re-identification vectors (Laperdrix, Bielova, Baudry, & Avoine, 2020).
//! This module exposes the deterministic, evidence-bound contract those
//! surfaces must satisfy before an adapter may claim a complete stealth
//! presentation. It deliberately performs no evasion: it never defeats an
//! access-control, CAPTCHA, or bot-management gate, and never reads the host.

use std::error::Error;
use std::fmt;

/// A page-observable render or media surface that a stealth adapter must
/// prove before admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StealthSurface {
    /// Canvas pixel and text rendering observations.
    Canvas,
    /// WebGL vendor, renderer, and UNMASKED extension observations.
    WebGL,
    /// WebAudio sample-rate and analyser observations.
    WebAudio,
    /// WebRTC interface candidate observations.
    WebRtc,
}

const REQUIRED_STEALTH_SURFACES: [StealthSurface; 4] = [
    StealthSurface::Canvas,
    StealthSurface::WebGL,
    StealthSurface::WebAudio,
    StealthSurface::WebRtc,
];

/// Validate that an adapter overrides every required stealth surface.
///
/// The first missing surface is reported in stable contract order. Extra,
/// duplicate, or reordered supported entries do not change admission, so
/// feature negotiation stays order independent.
pub fn require_stealth_surfaces(supported: &[StealthSurface]) -> Result<(), StealthError> {
    for required in REQUIRED_STEALTH_SURFACES {
        if !supported.contains(&required) {
            return Err(StealthError::MissingSurface(required));
        }
    }
    Ok(())
}

/// A validation failure when assembling a stealth presentation surface set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StealthError {
    /// A canvas noise class was outside the enumerated supported set.
    InvalidCanvasNoise,
    /// A WebAudio sample rate was not a supported standard rate.
    InvalidSampleRate,
    /// An adapter claims a stealth surface it cannot override.
    MissingSurface(StealthSurface),
}

impl fmt::Display for StealthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCanvasNoise => formatter
                .write_str("canvas noise class must be one of the enumerated supported values"),
            Self::InvalidSampleRate => {
                formatter.write_str("web audio sample rate must be a supported standard rate")
            }
            Self::MissingSurface(surface) => {
                write!(
                    formatter,
                    "adapter cannot override required {surface:?} stealth surface"
                )
            }
        }
    }
}

impl Error for StealthError {}

/// A bounded, deterministic canvas pixel-noise class.
///
/// Classes map to small closed ranges of least-significant pixel bits so an
/// adapter can widen or narrow noise without presenting a freshly randomized
/// per-session value, which W3C guidance warns can create new distinguishers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasNoise {
    /// No injected pixel noise; the smallest observed-distortion class.
    Crisp,
    /// A single least-significant-bit noise class.
    Smooth,
    /// A two-bit noise class.
    Diffuse,
}

impl CanvasNoise {
    /// Map an enumerated class index onto a noise class, rejecting others.
    pub const fn quantize(class: u8) -> Result<Self, StealthError> {
        match class {
            0 => Ok(Self::Crisp),
            1 => Ok(Self::Smooth),
            2 => Ok(Self::Diffuse),
            _ => Err(StealthError::InvalidCanvasNoise),
        }
    }

    /// Return the bounded least-significant bit shift for this class.
    #[must_use]
    pub const fn bit_shift(self) -> u8 {
        match self {
            Self::Crisp => 0,
            Self::Smooth => 1,
            Self::Diffuse => 2,
        }
    }
}

/// A standardized WebGL renderer token that does not name the host GPU.
///
/// Adapters expose one of these tokens instead of surfacing vendor-specific
/// GPU model strings, which fingerprinting research identifies as a strong
/// re-identification signal (Laperdrix et al., 2020).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebGlRendererToken {
    /// ANGLE over a hardware driver family.
    Angle,
    /// Software rendering with no identifying driver string.
    Standard,
}

impl WebGlRendererToken {
    /// Canonicalize a known renderer spelling onto a bounded token.
    ///
    /// Unrecognized spellings fail closed to `None` rather than being echoed
    /// to a new class, so an adapter cannot widen the token set by fiat.
    #[must_use]
    pub fn canonical(spelling: &str) -> Option<Self> {
        let upper = spelling.to_ascii_uppercase();
        if upper.starts_with("ANGLE") {
            Some(Self::Angle)
        } else if upper.contains("SOFTWARE") {
            Some(Self::Standard)
        } else {
            None
        }
    }
}

/// A supported WebAudio sample rate in hertz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebAudioRate {
    /// The standard 44.1 kHz rate.
    Rate44100,
    /// The standard 48 kHz rate.
    Rate48000,
}

impl WebAudioRate {
    /// Normalize an observed sample rate onto a supported standard rate.
    pub fn normalize(rate_hz: u32) -> Result<Self, StealthError> {
        match rate_hz {
            44_100 => Ok(Self::Rate44100),
            48_000 => Ok(Self::Rate48000),
            _ => Err(StealthError::InvalidSampleRate),
        }
    }

    /// Return the exact hertz value for this rate.
    #[must_use]
    pub const fn rate_hz(self) -> u32 {
        match self {
            Self::Rate44100 => 44_100,
            Self::Rate48000 => 48_000,
        }
    }
}

/// A bounded WebRTC interface-candidate policy.
///
/// This is policy only; the kernel never creates a peer connection or exposes
/// an address. Variant names describe the page-visible candidate behavior
/// directly so adapter code cannot mistake candidate disclosure for a safe
/// privacy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebRtcInterface {
    /// The adapter deliberately exposes direct interface candidates.
    DirectCandidates,
    /// The adapter publishes only mDNS-candidate interfaces.
    MDnsOnly,
}

impl WebRtcInterface {
    /// Whether this policy exposes local interface candidates directly.
    #[must_use]
    pub fn exposes_candidates(self) -> bool {
        matches!(self, Self::DirectCandidates)
    }
}
