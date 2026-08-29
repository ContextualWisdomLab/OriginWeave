//! Validated, internally consistent browser presentation identities for
//! OriginWeave agent sessions.
//!
//! Web pages can observe a high-entropy fingerprint derived from the host:
//! exact screen metrics, processor topology, locale chains, and timezone.
//! Longitudinal measurement research shows such surfaces are sufficient to
//! reidentify a browser without cookies (Laperdrix, Bielova, Baudry, & Avoine,
//! 2020; Cao, Li, & Wijmans, 2017). This kernel validates an explicit
//! *presentation identity* whose values belong to bounded, internally
//! consistent Chromium-compatible classes (W3C Fingerprinting Guidance,
//! 2025). It deliberately does not select a default profile without an
//! evidence-backed anonymity cohort.
//!
//! The kernel is a pure control-plane contract. It never touches the network,
//! never reads the real machine, and never claims to defeat an access-control
//! decision: defeating bot-management or consent gates remains prohibited by
//! the product policy (`docs/PRD.md`, PRD-CRAWL-003). What it provides is the
//! validated identity surface that adapters may present to pages, plus a
//! lowercase SHA-256 digest for evidence binding.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod coherence;
mod stealth;
mod ua_hints;
mod web_audio_guard;

pub use coherence::{CoherenceError, require_hints_coherence};
pub use stealth::{
    CanvasNoise, StealthError, StealthSurface, WebAudioRate, WebGlRendererToken, WebRtcInterface,
    require_stealth_surfaces,
};
pub use ua_hints::{
    ClientHintsError, HintsArchitecture, HintsBitness, HintsPlatform, UaBrand, UaClientHints,
};
pub use web_audio_guard::{WebAudioDecision, WebAudioFingerprintPolicy, WebAudioPolicyError};

use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

/// A validation failure for a presentation identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationError {
    /// A digest was not `sha256:` followed by 64 lowercase hexadecimal digits.
    InvalidDigest,
    /// A syntactically valid stored digest did not match the replayed fields.
    DigestMismatch,
    /// A profile field violated its bounded plausibility contract.
    InvalidField,
    /// Cross-field consistency failed (for example viewport exceeds screen).
    InconsistentIdentity,
    /// An adapter cannot override one required observable surface.
    MissingSurface(PresentationSurface),
}

impl fmt::Display for PresentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDigest => {
                formatter.write_str("digest must be sha256: plus 64 lowercase hex digits")
            }
            Self::DigestMismatch => {
                formatter.write_str("stored presentation digest does not match profile fields")
            }
            Self::InvalidField => {
                formatter.write_str("presentation field violates its bounded contract")
            }
            Self::InconsistentIdentity => {
                formatter.write_str("presentation fields contradict each other")
            }
            Self::MissingSurface(surface) => {
                write!(
                    formatter,
                    "adapter cannot override required {surface:?} surface"
                )
            }
        }
    }
}

impl Error for PresentationError {}

/// A page-observable field that an adapter must override before admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationSurface {
    /// Screen dimensions and color depth.
    Screen,
    /// Viewport dimensions.
    Viewport,
    /// Device pixel ratio.
    DevicePixelRatio,
    /// Logical processor count.
    HardwareConcurrency,
    /// Named time-zone identity and offset behavior.
    TimeZone,
    /// Browser platform family.
    Platform,
    /// Ordered language preferences.
    Languages,
    /// Reduced-motion preference.
    ReducedMotion,
}

const REQUIRED_PRESENTATION_SURFACES: [PresentationSurface; 8] = [
    PresentationSurface::Screen,
    PresentationSurface::Viewport,
    PresentationSurface::DevicePixelRatio,
    PresentationSurface::HardwareConcurrency,
    PresentationSurface::TimeZone,
    PresentationSurface::Platform,
    PresentationSurface::Languages,
    PresentationSurface::ReducedMotion,
];

/// Require an adapter to override every surface in the current presentation schema.
///
/// The first missing surface is returned in stable contract order. Additional
/// or duplicate supported entries do not change admission.
pub fn require_presentation_surfaces(
    supported: &[PresentationSurface],
) -> Result<(), PresentationError> {
    for required in REQUIRED_PRESENTATION_SURFACES {
        if !supported.contains(&required) {
            return Err(PresentationError::MissingSurface(required));
        }
    }
    Ok(())
}

/// Screen geometry with color depth as pages observe it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenMetrics {
    width_px: u32,
    height_px: u32,
    color_depth_bits: u8,
}

impl ScreenMetrics {
    /// Validate screen geometry; Chromium reports 24-bit color depth.
    pub const fn new(width_px: u32, height_px: u32) -> Result<Self, PresentationError> {
        if width_px == 0
            || height_px == 0
            || width_px > MAX_SCREEN_EDGE
            || height_px > MAX_SCREEN_EDGE
        {
            return Err(PresentationError::InvalidField);
        }
        Ok(Self {
            width_px,
            height_px,
            color_depth_bits: COLOR_DEPTH_BITS,
        })
    }

    /// Return the CSS-pixel screen width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width_px
    }

    /// Return the CSS-pixel screen height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height_px
    }

    /// Return the reported color depth in bits per pixel channel group.
    #[must_use]
    pub const fn color_depth_bits(&self) -> u8 {
        self.color_depth_bits
    }
}

/// The maximum accepted CSS-pixel edge length for a screen.
const MAX_SCREEN_EDGE: u32 = 7680;

/// The color depth Chromium reports for standard desktop panels.
const COLOR_DEPTH_BITS: u8 = 24;

/// Viewport bounds (`window.innerWidth` / `innerHeight` class values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportBounds {
    width_px: u32,
    height_px: u32,
}

impl ViewportBounds {
    /// Validate nonzero viewport dimensions within the accepted ceiling.
    pub const fn new(width_px: u32, height_px: u32) -> Result<Self, PresentationError> {
        if width_px == 0
            || height_px == 0
            || width_px > MAX_SCREEN_EDGE
            || height_px > MAX_SCREEN_EDGE
        {
            return Err(PresentationError::InvalidField);
        }
        Ok(Self {
            width_px,
            height_px,
        })
    }

    /// Return the viewport width in CSS pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width_px
    }

    /// Return the viewport height in CSS pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height_px
    }
}

/// Quantized device pixel ratios that desktop Chromium commonly reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePixelRatio {
    /// Standard-density displays report exactly 1.0.
    Quantized1,
    /// Common scaled laptop panels report exactly 1.5.
    Quantized15,
    /// High-density retina-class panels report exactly 2.0.
    Quantized2,
}

impl DevicePixelRatio {
    /// Map an observed ratio onto its quantized class, rejecting others.
    #[must_use]
    pub fn from_ratio(value: f64) -> Option<Self> {
        if (value - 1.0).abs() < f64::EPSILON {
            Some(Self::Quantized1)
        } else if (value - 1.5).abs() < f64::EPSILON {
            Some(Self::Quantized15)
        } else if (value - 2.0).abs() < f64::EPSILON {
            Some(Self::Quantized2)
        } else {
            None
        }
    }

    /// Return the exact numeric value this class represents.
    #[must_use]
    pub const fn value(self) -> f64 {
        match self {
            Self::Quantized1 => 1.0,
            Self::Quantized15 => 1.5,
            Self::Quantized2 => 2.0,
        }
    }
}

/// The operating-system platform token a page observes through `navigator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationPlatform {
    /// Windows desktop Chromium.
    Windows,
    /// macOS desktop Chromium.
    MacOS,
    /// Linux desktop Chromium.
    Linux,
}
/// A named time-zone identity that Chromium can expose consistently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationTimeZone {
    /// Coordinated Universal Time, which has no daylight-saving transition.
    Utc,
}

impl PresentationTimeZone {
    /// Return the IANA identifier supplied to the browser adapter.
    #[must_use]
    pub const fn iana_name(self) -> &'static str {
        match self {
            Self::Utc => "UTC",
        }
    }

    /// Return the fixed offset for the supported standardized identity.
    #[must_use]
    pub const fn offset_minutes(self) -> i32 {
        match self {
            Self::Utc => 0,
        }
    }
}

impl PresentationPlatform {
    /// Return the JavaScript-visible platform string for this family.
    #[must_use]
    pub const fn user_agent_token(self) -> &'static str {
        match self {
            Self::Windows => "Win32",
            Self::MacOS => "MacIntel",
            Self::Linux => "Linux x86_64",
        }
    }

    /// Return the canonical UA Client Hints platform token for this family.
    ///
    /// A page reconciles the presentation platform with
    /// `navigator.userAgentData.platform`, so the two must agree; the mapping
    /// is the single source of truth an adapter consumes (see ADR 0112).
    #[must_use]
    pub const fn hints_platform(self) -> HintsPlatform {
        match self {
            Self::Windows => HintsPlatform::Windows,
            Self::MacOS => HintsPlatform::MacOs,
            Self::Linux => HintsPlatform::Linux,
        }
    }
}

/// A lowercase SHA-256 digest identifier bound to one canonical profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PresentationDigest(String);

impl PresentationDigest {
    /// Validate the canonical `sha256:<64 lowercase hex>` form.
    pub fn new(value: &str) -> Result<Self, PresentationError> {
        let Some(hexadecimal) = value.strip_prefix("sha256:") else {
            return Err(PresentationError::InvalidDigest);
        };
        let bytes = hexadecimal.as_bytes();
        if bytes.len() != 64
            || bytes
                .iter()
                .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        {
            return Err(PresentationError::InvalidDigest);
        }
        Ok(Self(value.to_owned()))
    }

    /// Return the digest text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PresentationDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// An immutable, internally consistent browser presentation identity.
///
/// Values are quantized onto enumerated plausible classes instead of copying
/// host-specific observations, which reduces the entropy available to a page
/// while keeping every field mutually consistent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationProfile {
    screen: ScreenMetrics,
    viewport: ViewportBounds,
    device_pixel_ratio: DevicePixelRatio,
    hardware_concurrency: u16,
    timezone: PresentationTimeZone,
    platform: PresentationPlatform,
    languages: Vec<String>,
    reduced_motion: bool,
    digest: PresentationDigest,
}

/// Enumerated plausible desktop screen sizes in CSS pixels.
const SCREEN_SET: [(u32, u32); 8] = [
    (1280, 720),
    (1366, 768),
    (1440, 900),
    (1536, 864),
    (1600, 900),
    (1920, 1080),
    (2560, 1440),
    (3840, 2160),
];

/// Enumerated plausible window widths, filtered against the chosen screen.
const VIEWPORT_WIDTH_SET: [u32; 7] = [1024, 1200, 1280, 1366, 1440, 1600, 1920];

/// Enumerated plausible window heights, filtered against the chosen screen.
const VIEWPORT_HEIGHT_SET: [u32; 6] = [600, 720, 800, 900, 937, 1080];

/// Enumerated plausible logical processor counts.
const HARDWARE_CONCURRENCY_SET: [u16; 6] = [2, 4, 6, 8, 12, 16];

/// Enumerated common first languages in BCP 47 form.
const FIRST_LANGUAGE_SET: [&str; 8] = [
    "en-US", "en-GB", "de-DE", "fr-FR", "es-ES", "ja-JP", "ko-KR", "zh-CN",
];

/// The optional second language appended when the stream selects it.
const SECOND_LANGUAGE: &str = "en";

impl PresentationProfile {
    /// Construct and fully validate one profile from explicit fields.
    ///
    /// This binds a fresh digest to the canonical serialization. Callers that
    /// replay persisted evidence must use [`Self::replay`] so a stored digest
    /// is checked instead of silently replaced by a recomputed value.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        screen: ScreenMetrics,
        viewport: ViewportBounds,
        device_pixel_ratio: DevicePixelRatio,
        hardware_concurrency: u16,
        timezone: PresentationTimeZone,
        platform: PresentationPlatform,
        languages: Vec<String>,
        reduced_motion: bool,
    ) -> Result<Self, PresentationError> {
        if viewport.width_px > screen.width_px || viewport.height_px > screen.height_px {
            return Err(PresentationError::InconsistentIdentity);
        }
        if platform == PresentationPlatform::MacOS
            && device_pixel_ratio == DevicePixelRatio::Quantized15
        {
            return Err(PresentationError::InconsistentIdentity);
        }
        if !SCREEN_SET.contains(&(screen.width_px, screen.height_px))
            || !VIEWPORT_WIDTH_SET.contains(&viewport.width_px)
            || !VIEWPORT_HEIGHT_SET.contains(&viewport.height_px)
            || !HARDWARE_CONCURRENCY_SET.contains(&hardware_concurrency)
        {
            return Err(PresentationError::InvalidField);
        }
        let languages_are_enumerated = match languages.as_slice() {
            [first] => FIRST_LANGUAGE_SET.contains(&first.as_str()),
            [first, second] => {
                FIRST_LANGUAGE_SET.contains(&first.as_str()) && second == SECOND_LANGUAGE
            }
            _ => false,
        };
        if !languages_are_enumerated {
            return Err(PresentationError::InvalidField);
        }

        Ok(Self::assemble(
            screen,
            viewport,
            device_pixel_ratio,
            hardware_concurrency,
            timezone,
            platform,
            languages,
            reduced_motion,
        ))
    }

    /// Replay a previously issued profile and verify its persisted digest.
    ///
    /// Field validation is identical to [`Self::new`]. The supplied digest is
    /// then compared with the digest recomputed from the exact canonical field
    /// serialization; a mismatch fails closed and never substitutes the newly
    /// computed value for the persisted evidence identity.
    #[allow(clippy::too_many_arguments)]
    pub fn replay(
        screen: ScreenMetrics,
        viewport: ViewportBounds,
        device_pixel_ratio: DevicePixelRatio,
        hardware_concurrency: u16,
        timezone: PresentationTimeZone,
        platform: PresentationPlatform,
        languages: Vec<String>,
        reduced_motion: bool,
        expected_digest: &PresentationDigest,
    ) -> Result<Self, PresentationError> {
        let profile = Self::new(
            screen,
            viewport,
            device_pixel_ratio,
            hardware_concurrency,
            timezone,
            platform,
            languages,
            reduced_motion,
        )?;
        if profile.digest() != expected_digest {
            return Err(PresentationError::DigestMismatch);
        }
        Ok(profile)
    }

    /// Assemble one profile and bind its canonical digest.
    ///
    /// Callers must have validated the fields already; assembly itself is
    /// total so derivation from enumerated sets stays infallible.
    #[allow(clippy::too_many_arguments)]
    fn assemble(
        screen: ScreenMetrics,
        viewport: ViewportBounds,
        device_pixel_ratio: DevicePixelRatio,
        hardware_concurrency: u16,
        timezone: PresentationTimeZone,
        platform: PresentationPlatform,
        languages: Vec<String>,
        reduced_motion: bool,
    ) -> Self {
        let mut candidate = Self {
            screen,
            viewport,
            device_pixel_ratio,
            hardware_concurrency,
            timezone,
            platform,
            languages,
            reduced_motion,
            digest: PresentationDigest(String::new()),
        };
        candidate.digest = candidate.compute_digest();
        candidate
    }

    /// Compute the lowercase SHA-256 digest of this exact field set.
    fn compute_digest(&self) -> PresentationDigest {
        let serialized = canonical_serialization(self);
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let finalized = hasher.finalize();
        let mut text = String::with_capacity(7 + 64);
        text.push_str("sha256:");
        for byte in finalized {
            text.push(hex_digit(byte >> 4));
            text.push(hex_digit(byte & 0x0f));
        }
        PresentationDigest(text)
    }

    /// Return the validated screen metrics.
    #[must_use]
    pub const fn screen(&self) -> &ScreenMetrics {
        &self.screen
    }

    /// Return the validated viewport bounds.
    #[must_use]
    pub const fn viewport(&self) -> &ViewportBounds {
        &self.viewport
    }

    /// Return the quantized device pixel ratio class.
    #[must_use]
    pub const fn device_pixel_ratio(&self) -> DevicePixelRatio {
        self.device_pixel_ratio
    }

    /// Return the quantized logical processor count.
    #[must_use]
    pub const fn hardware_concurrency(&self) -> u16 {
        self.hardware_concurrency
    }

    /// Return the whole-hour UTC offset in minutes.
    #[must_use]
    pub const fn timezone_offset_minutes(&self) -> i32 {
        self.timezone.offset_minutes()
    }

    /// Return the named time-zone identity presented to pages.
    #[must_use]
    pub const fn timezone(&self) -> PresentationTimeZone {
        self.timezone
    }

    /// Return the platform family.
    #[must_use]
    pub const fn platform(&self) -> PresentationPlatform {
        self.platform
    }

    /// Return the ordered BCP 47 language tags.
    #[must_use]
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// Return whether reduced motion was requested for this identity.
    #[must_use]
    pub const fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    /// Return the lowercase SHA-256 digest bound to this exact profile.
    #[must_use]
    pub fn digest(&self) -> &PresentationDigest {
        &self.digest
    }
}

fn canonical_serialization(profile: &PresentationProfile) -> String {
    format!(
        "originweave-presentation/v1|screen={}x{}x{}|viewport={}x{}|dpr={}|hw={}|tz={}|platform={}|langs={}|reduced_motion={}",
        profile.screen.width_px,
        profile.screen.height_px,
        profile.screen.color_depth_bits,
        profile.viewport.width_px,
        profile.viewport.height_px,
        format_ratio(profile.device_pixel_ratio),
        profile.hardware_concurrency,
        profile.timezone.iana_name(),
        profile.platform.user_agent_token(),
        profile.languages.join(","),
        profile.reduced_motion
    )
}

const fn format_ratio(ratio: DevicePixelRatio) -> &'static str {
    match ratio {
        DevicePixelRatio::Quantized1 => "1",
        DevicePixelRatio::Quantized15 => "1.5",
        DevicePixelRatio::Quantized2 => "2",
    }
}

const fn hex_digit(value: u8) -> char {
    if value < 10 {
        (b'0' + value) as char
    } else {
        (b'a' + value - 10) as char
    }
}
