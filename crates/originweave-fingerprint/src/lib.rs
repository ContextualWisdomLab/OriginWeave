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

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn presentation_error_display_covers_every_variant() {
        assert_eq!(
            PresentationError::InvalidDigest.to_string(),
            "digest must be sha256: plus 64 lowercase hex digits"
        );
        assert_eq!(
            PresentationError::DigestMismatch.to_string(),
            "stored presentation digest does not match profile fields"
        );
        assert_eq!(
            PresentationError::InvalidField.to_string(),
            "presentation field violates its bounded contract"
        );
        assert_eq!(
            PresentationError::InconsistentIdentity.to_string(),
            "presentation fields contradict each other"
        );
        assert_eq!(
            PresentationError::MissingSurface(PresentationSurface::HardwareConcurrency).to_string(),
            "adapter cannot override required HardwareConcurrency surface"
        );
    }

    #[test]
    fn screen_metrics_reject_zero_and_oversized_edges() {
        assert_eq!(
            ScreenMetrics::new(0, 1080),
            Err(PresentationError::InvalidField)
        );
        assert_eq!(
            ScreenMetrics::new(1920, 0),
            Err(PresentationError::InvalidField)
        );
        assert_eq!(
            ScreenMetrics::new(MAX_SCREEN_EDGE + 1, 1080),
            Err(PresentationError::InvalidField)
        );
        assert_eq!(
            ScreenMetrics::new(1920, MAX_SCREEN_EDGE + 1),
            Err(PresentationError::InvalidField)
        );
        let screen = ScreenMetrics::new(1920, 1080).expect("valid screen");
        assert_eq!(screen.color_depth_bits(), COLOR_DEPTH_BITS);
    }

    #[test]
    fn viewport_bounds_reject_invalid_dimensions() {
        assert_eq!(
            ViewportBounds::new(0, 100),
            Err(PresentationError::InvalidField)
        );
        assert_eq!(
            ViewportBounds::new(100, 0),
            Err(PresentationError::InvalidField)
        );
        assert_eq!(
            ViewportBounds::new(MAX_SCREEN_EDGE + 1, 100),
            Err(PresentationError::InvalidField)
        );
        assert_eq!(
            ViewportBounds::new(100, MAX_SCREEN_EDGE + 1),
            Err(PresentationError::InvalidField)
        );
        let viewport = ViewportBounds::new(1280, 720).expect("valid viewport");
        assert_eq!((viewport.width(), viewport.height()), (1280, 720));
    }

    #[test]
    fn device_pixel_ratio_maps_exact_quantized_values() {
        assert_eq!(
            DevicePixelRatio::from_ratio(1.0),
            Some(DevicePixelRatio::Quantized1)
        );
        assert_eq!(
            DevicePixelRatio::from_ratio(1.5),
            Some(DevicePixelRatio::Quantized15)
        );
        assert_eq!(
            DevicePixelRatio::from_ratio(2.0),
            Some(DevicePixelRatio::Quantized2)
        );
        assert_eq!(DevicePixelRatio::from_ratio(1.25), None);
        for ratio in [
            DevicePixelRatio::Quantized1,
            DevicePixelRatio::Quantized15,
            DevicePixelRatio::Quantized2,
        ] {
            assert_eq!(
                ratio.value(),
                DevicePixelRatio::from_ratio(ratio.value())
                    .expect("round trip")
                    .value()
            );
        }
    }

    #[test]
    fn platform_tokens_are_stable() {
        assert_eq!(PresentationPlatform::Windows.user_agent_token(), "Win32");
        assert_eq!(PresentationPlatform::MacOS.user_agent_token(), "MacIntel");
        assert_eq!(
            PresentationPlatform::Linux.user_agent_token(),
            "Linux x86_64"
        );
    }

    #[test]
    fn digest_validation_rejects_each_malformation() {
        assert_eq!(
            PresentationDigest::new(""),
            Err(PresentationError::InvalidDigest)
        );
        assert_eq!(
            PresentationDigest::new(
                "sha257:0000000000000000000000000000000000000000000000000000000000000000"
            ),
            Err(PresentationError::InvalidDigest)
        );
        assert_eq!(
            PresentationDigest::new(
                "sha256:00000000000000000000000000000000000000000000000000000000000000"
            ),
            Err(PresentationError::InvalidDigest)
        );
        assert_eq!(
            PresentationDigest::new(
                "sha256:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
            ),
            Err(PresentationError::InvalidDigest)
        );
        assert_eq!(
            PresentationDigest::new(
                "sha256:A000000000000000000000000000000000000000000000000000000000000000"
            ),
            Err(PresentationError::InvalidDigest)
        );
        let valid = PresentationDigest::new(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("valid digest");
        assert_eq!(valid.to_string(), valid.as_str());
    }

    #[test]
    fn standardized_timezone_has_one_consistent_identity() {
        assert_eq!(PresentationTimeZone::Utc.iana_name(), "UTC");
        assert_eq!(PresentationTimeZone::Utc.offset_minutes(), 0);
    }

    #[test]
    fn profile_new_validates_each_field_independently() {
        let screen = ScreenMetrics::new(1920, 1080).expect("screen");
        let viewport = ViewportBounds::new(1920, 900).expect("viewport");

        // Viewport taller than the screen is impossible.
        let tall = ViewportBounds::new(1920, 1200).expect("viewport");
        assert_eq!(
            PresentationProfile::new(
                screen,
                tall,
                DevicePixelRatio::Quantized1,
                8,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                vec!["en".to_owned()],
                false
            ),
            Err(PresentationError::InconsistentIdentity)
        );
        let wide = ViewportBounds::new(2560, 1080).expect("viewport");
        assert_eq!(
            PresentationProfile::new(
                screen,
                wide,
                DevicePixelRatio::Quantized1,
                8,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                vec!["en".to_owned()],
                false
            ),
            Err(PresentationError::InconsistentIdentity)
        );

        // Trusted replay cannot reintroduce high-entropy arbitrary dimensions.
        let odd_screen = ScreenMetrics::new(1919, 1080).expect("bounded screen");
        assert_eq!(
            PresentationProfile::new(
                odd_screen,
                ViewportBounds::new(1024, 600).expect("viewport"),
                DevicePixelRatio::Quantized1,
                8,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                vec!["en".to_owned()],
                false
            ),
            Err(PresentationError::InvalidField)
        );
        let odd_viewport = ViewportBounds::new(1919, 900).expect("bounded viewport");
        assert_eq!(
            PresentationProfile::new(
                screen,
                odd_viewport,
                DevicePixelRatio::Quantized1,
                8,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                vec!["en".to_owned()],
                false
            ),
            Err(PresentationError::InvalidField)
        );
        let odd_viewport_height = ViewportBounds::new(1920, 899).expect("bounded viewport");
        assert_eq!(
            PresentationProfile::new(
                screen,
                odd_viewport_height,
                DevicePixelRatio::Quantized1,
                8,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                vec!["en".to_owned()],
                false
            ),
            Err(PresentationError::InvalidField)
        );

        // Processor count outside the enumerated set is rejected.
        assert_eq!(
            PresentationProfile::new(
                screen,
                viewport,
                DevicePixelRatio::Quantized1,
                3,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                vec!["en".to_owned()],
                false
            ),
            Err(PresentationError::InvalidField)
        );

        // Language validation flows through.
        assert_eq!(
            PresentationProfile::new(
                screen,
                viewport,
                DevicePixelRatio::Quantized1,
                8,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                Vec::new(),
                false
            ),
            Err(PresentationError::InvalidField)
        );
        for languages in [
            vec!["cy-GB".to_owned()],
            vec!["cy-GB".to_owned(), "en".to_owned()],
            vec!["ko-KR".to_owned(), "fr-FR".to_owned()],
            vec!["ko-KR".to_owned(), "en".to_owned(), "en-GB".to_owned()],
        ] {
            assert_eq!(
                PresentationProfile::new(
                    screen,
                    viewport,
                    DevicePixelRatio::Quantized1,
                    8,
                    PresentationTimeZone::Utc,
                    PresentationPlatform::Linux,
                    languages,
                    false
                ),
                Err(PresentationError::InvalidField)
            );
        }

        assert_eq!(
            PresentationProfile::new(
                screen,
                viewport,
                DevicePixelRatio::Quantized15,
                12,
                PresentationTimeZone::Utc,
                PresentationPlatform::MacOS,
                vec!["ko-KR".to_owned(), "en".to_owned()],
                true,
            ),
            Err(PresentationError::InconsistentIdentity)
        );
        let profile = PresentationProfile::new(
            screen,
            viewport,
            DevicePixelRatio::Quantized1,
            12,
            PresentationTimeZone::Utc,
            PresentationPlatform::MacOS,
            vec!["ko-KR".to_owned(), "en".to_owned()],
            true,
        )
        .expect("valid profile");
        assert_eq!(profile.device_pixel_ratio().value(), 1.0);
        assert_eq!(profile.hardware_concurrency(), 12);
        assert_eq!(profile.timezone_offset_minutes(), 0);
        assert_eq!(profile.timezone(), PresentationTimeZone::Utc);
        assert_eq!(profile.platform(), PresentationPlatform::MacOS);
        assert_eq!(profile.languages().len(), 2);
        assert!(profile.reduced_motion());
    }

    #[test]
    fn format_ratio_covers_each_quantized_class() {
        assert_eq!(format_ratio(DevicePixelRatio::Quantized1), "1");
        assert_eq!(format_ratio(DevicePixelRatio::Quantized15), "1.5");
        assert_eq!(format_ratio(DevicePixelRatio::Quantized2), "2");
    }

    #[test]
    fn hex_digit_lowercases_every_nibble() {
        for value in 0..16u8 {
            let expected = format!("{value:x}");
            assert_eq!(hex_digit(value).to_string(), expected);
        }
    }

    #[test]
    fn enumerated_sets_satisfy_their_public_validation_contracts() {
        // Every enumerated screen must pass the validating constructor, and
        // every enumerated viewport pair filtered to that screen likewise.
        for (screen_width, screen_height) in SCREEN_SET {
            assert!(
                VIEWPORT_WIDTH_SET
                    .into_iter()
                    .any(|width| width <= screen_width)
            );
            assert!(
                VIEWPORT_HEIGHT_SET
                    .into_iter()
                    .any(|height| height <= screen_height)
            );
            let screen = ScreenMetrics::new(screen_width, screen_height)
                .expect("enumerated screen satisfies the metric contract");
            assert_eq!(screen.width(), screen_width);
            assert_eq!(screen.height(), screen_height);
            for width in VIEWPORT_WIDTH_SET {
                if width > screen_width {
                    continue;
                }
                for height in VIEWPORT_HEIGHT_SET {
                    if height > screen_height {
                        continue;
                    }
                    let viewport = ViewportBounds::new(width, height)
                        .expect("filtered viewport satisfies the bounds contract");
                    assert_eq!((viewport.width(), viewport.height()), (width, height));
                }
            }
        }
        for concurrency in HARDWARE_CONCURRENCY_SET {
            let profile = PresentationProfile::new(
                ScreenMetrics::new(1920, 1080)
                    .expect("reference screen satisfies the metric contract"),
                ViewportBounds::new(1280, 720)
                    .expect("reference viewport satisfies the bounds contract"),
                DevicePixelRatio::Quantized1,
                concurrency,
                PresentationTimeZone::Utc,
                PresentationPlatform::Linux,
                vec!["en-US".to_owned()],
                false,
            )
            .expect("enumerated hardware concurrency satisfies the profile contract");
            assert_eq!(profile.hardware_concurrency(), concurrency);
        }
        for language in FIRST_LANGUAGE_SET {
            assert!((2..=35).contains(&language.len()));
            assert!(
                language
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            );
        }
        assert_eq!(SECOND_LANGUAGE, "en");
    }
}
