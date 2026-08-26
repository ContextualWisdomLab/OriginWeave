//! Seeded, internally consistent browser presentation identities for
//! OriginWeave agent sessions.
//!
//! Web pages can observe a high-entropy fingerprint derived from the host:
//! exact screen metrics, processor topology, locale chains, and timezone.
//! Longitudinal measurement research shows such surfaces are sufficient to
//! reidentify a browser without cookies (Laperdrix, Bielova, Baudry, & Avoine,
//! 2020; Cao, Li, Wijmans, & Song, 2017). This kernel gives every governed
//! session a *presentation identity* instead: a deterministic, internally
//! consistent Chromium-compatible profile whose values are quantized onto
//! enumerated plausible classes so the runtime stops leaking host-specific
//! uniqueness (W3C Fingerprinting Guidance, 2025).
//!
//! The kernel is a pure control-plane contract. It never touches the network,
//! never reads the real machine, and never claims to defeat an access-control
//! decision: defeating bot-management or consent gates remains prohibited by
//! the product policy (`docs/PRD.md`, PRD-CRAWL-003). What it provides is the
//! privacy-preserving, session-stable identity surface that adapters present
//! to pages, plus a lowercase SHA-256 digest for evidence binding.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

/// A validation or derivation failure for a presentation identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationError {
    /// A seed was the all-zero byte string and cannot be used.
    DegenerateSeed,
    /// A digest was not `sha256:` followed by 64 lowercase hexadecimal digits.
    InvalidDigest,
    /// A profile field violated its bounded plausibility contract.
    InvalidField,
    /// Cross-field consistency failed (for example viewport exceeds screen).
    InconsistentIdentity,
}

impl fmt::Display for PresentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::DegenerateSeed => "presentation seed must not be all zero",
            Self::InvalidDigest => "digest must be sha256: plus 64 lowercase hex digits",
            Self::InvalidField => "presentation field violates its bounded contract",
            Self::InconsistentIdentity => "presentation fields contradict each other",
        };
        formatter.write_str(message)
    }
}

impl Error for PresentationError {}

/// Domain-separation tag for derivation stream expansion.
const DERIVE_DOMAIN: &[u8] = b"originweave-presentation/v1";

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

    /// Assemble metrics from an enumerated pair already known to satisfy
    /// the public validating constructor.
    const fn from_enumerated(width_px: u32, height_px: u32) -> Self {
        Self {
            width_px,
            height_px,
            color_depth_bits: COLOR_DEPTH_BITS,
        }
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

    /// Assemble bounds from an enumerated pair already known to satisfy the
    /// public validating constructor.
    const fn from_enumerated(width_px: u32, height_px: u32) -> Self {
        Self {
            width_px,
            height_px,
        }
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

/// A validated 32-byte session seed for presentation derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PresentationSeed([u8; 32]);

impl PresentationSeed {
    /// Validate one seed; the all-zero seed cannot drive derivation.
    pub const fn new(bytes: [u8; 32]) -> Result<Self, PresentationError> {
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] != 0 {
                return Ok(Self(bytes));
            }
            index += 1;
        }
        Err(PresentationError::DegenerateSeed)
    }

    /// Return the seed bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
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

/// The maximum number of accepted language tags on one identity.
const MAX_LANGUAGE_TAGS: usize = 4;

impl PresentationProfile {
    /// Construct and fully validate one profile from explicit fields.
    ///
    /// Adapters use this when replaying a previously issued identity; the
    /// digest is recomputed from the canonical serialization so stored
    /// evidence always matches the presented values.
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
        if !HARDWARE_CONCURRENCY_SET.contains(&hardware_concurrency) {
            return Err(PresentationError::InvalidField);
        }
        validate_languages(&languages)?;

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

    /// Derive one deterministic profile from a session seed.
    ///
    /// The same seed always yields the identical profile and digest, so a
    /// session keeps a stable identity across navigations; rotating identity
    /// requires issuing a new seed at the control plane. Derivation is total:
    /// every selected value comes from a validated enumerated set.
    #[must_use]
    pub fn derive(seed: &PresentationSeed) -> Self {
        let screen_index = select_index(seed, 0, SCREEN_SET.len());
        let (screen_width, screen_height) = SCREEN_SET[screen_index];

        let ratio_index = select_index(seed, 1, 3);
        let device_pixel_ratio = [
            DevicePixelRatio::Quantized1,
            DevicePixelRatio::Quantized15,
            DevicePixelRatio::Quantized2,
        ][ratio_index];

        let eligible_widths: Vec<u32> = VIEWPORT_WIDTH_SET
            .into_iter()
            .filter(|width| *width <= screen_width)
            .collect();
        let eligible_heights: Vec<u32> = VIEWPORT_HEIGHT_SET
            .into_iter()
            .filter(|height| *height <= screen_height)
            .collect();
        let width_index = select_index(seed, 2, eligible_widths.len());
        let height_index = select_index(seed, 3, eligible_heights.len());

        let concurrency_index = select_index(seed, 4, HARDWARE_CONCURRENCY_SET.len());
        let hardware_concurrency = HARDWARE_CONCURRENCY_SET[concurrency_index];

        let platform_index = select_index(seed, 6, 3);
        let platform = [
            PresentationPlatform::Windows,
            PresentationPlatform::MacOS,
            PresentationPlatform::Linux,
        ][platform_index];

        let language_index = select_index(seed, 7, FIRST_LANGUAGE_SET.len());
        let mut languages = vec![FIRST_LANGUAGE_SET[language_index].to_owned()];
        if select_index(seed, 8, 2) == 1 {
            languages.push(SECOND_LANGUAGE.to_owned());
        }

        let screen = ScreenMetrics::from_enumerated(screen_width, screen_height);
        let viewport = ViewportBounds::from_enumerated(
            eligible_widths[width_index],
            eligible_heights[height_index],
        );

        Self::assemble(
            screen,
            viewport,
            device_pixel_ratio,
            hardware_concurrency,
            PresentationTimeZone::Utc,
            platform,
            languages,
            select_index(seed, 9, 2) == 1,
        )
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

fn validate_languages(languages: &[String]) -> Result<(), PresentationError> {
    if languages.is_empty() || languages.len() > MAX_LANGUAGE_TAGS {
        return Err(PresentationError::InvalidField);
    }
    for tag in languages {
        let valid = (2..=35).contains(&tag.len())
            && tag
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
        if !valid {
            return Err(PresentationError::InvalidField);
        }
    }
    Ok(())
}

fn canonical_serialization(profile: &PresentationProfile) -> String {
    format!(
        "originweave-presentation/v1|screen={}x{}x{}|viewport={}x{}|dpr={}|hw={}|tz={}|platform={}|langs={}|reduced_motion={}",
        profile.screen.width_px,
        profile.screen.height_px,
        profile.screen.color_depth_bits,
        profile.viewport.width_px,
        profile.viewport.height_px,
        format_ratio(profile.device_pixel_ratio.value()),
        profile.hardware_concurrency,
        profile.timezone.iana_name(),
        profile.platform.user_agent_token(),
        profile.languages.join(","),
        profile.reduced_motion
    )
}

fn format_ratio(value: f64) -> String {
    if value == 1.5 {
        "1.5".to_owned()
    } else if value == 2.0 {
        "2".to_owned()
    } else {
        "1".to_owned()
    }
}

const fn hex_digit(value: u8) -> char {
    if value < 10 {
        (b'0' + value) as char
    } else {
        (b'a' + value - 10) as char
    }
}

/// Select one uniform index from a counter-expanded SHA-256 stream block.
///
/// Modulo selection over `u64` keeps relative bias below 2^-53 for every
/// enumerated set used here because each set size stays far below 2^53.
fn select_index(seed: &PresentationSeed, slot: usize, set_size: usize) -> usize {
    let stream = expand_stream(seed, slot as u32);
    let word = u64::from_be_bytes(stream);
    (word % set_size as u64) as usize
}

fn expand_stream(seed: &PresentationSeed, slot: u32) -> [u8; 8] {
    let mut hasher_input = [0u8; 32 + DERIVE_DOMAIN.len() + 4];
    let mut cursor = 0;
    while cursor < DERIVE_DOMAIN.len() {
        hasher_input[cursor] = DERIVE_DOMAIN[cursor];
        cursor += 1;
    }
    while cursor < 32 + DERIVE_DOMAIN.len() {
        hasher_input[cursor] = seed.0[cursor - DERIVE_DOMAIN.len()];
        cursor += 1;
    }
    let slot_bytes = slot.to_le_bytes();
    hasher_input[cursor] = slot_bytes[0];
    hasher_input[cursor + 1] = slot_bytes[1];
    hasher_input[cursor + 2] = slot_bytes[2];
    hasher_input[cursor + 3] = slot_bytes[3];

    // The constant-size input lets this run without heap allocation while the
    // caller still receives the first eight bytes of one SHA-256 evaluation.
    let mut state = Sha256::new();
    state.update(hasher_input);
    let finalized = state.finalize();
    let mut output = [0u8; 8];
    let mut index = 0;
    while index < 8 {
        output[index] = finalized[index];
        index += 1;
    }
    output
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    const SEED: [u8; 32] = [7u8; 32];

    fn seed() -> PresentationSeed {
        PresentationSeed::new(SEED).expect("valid seed")
    }

    #[test]
    fn presentation_error_display_covers_every_variant() {
        assert_eq!(
            PresentationError::DegenerateSeed.to_string(),
            "presentation seed must not be all zero"
        );
        assert_eq!(
            PresentationError::InvalidDigest.to_string(),
            "digest must be sha256: plus 64 lowercase hex digits"
        );
        assert_eq!(
            PresentationError::InvalidField.to_string(),
            "presentation field violates its bounded contract"
        );
        assert_eq!(
            PresentationError::InconsistentIdentity.to_string(),
            "presentation fields contradict each other"
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
                "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
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
    fn language_validation_rejects_empty_oversized_and_bad_tags() {
        assert_eq!(
            validate_languages(&[]),
            Err(PresentationError::InvalidField)
        );
        let too_many = vec![
            "en".to_owned(),
            "de".to_owned(),
            "fr".to_owned(),
            "es".to_owned(),
            "it".to_owned(),
        ];
        assert_eq!(
            validate_languages(&too_many),
            Err(PresentationError::InvalidField)
        );
        assert_eq!(
            validate_languages(&["e".to_owned()]),
            Err(PresentationError::InvalidField)
        );
        let oversized = "a".repeat(36);
        assert_eq!(
            validate_languages(&[oversized]),
            Err(PresentationError::InvalidField)
        );
        assert_eq!(
            validate_languages(&["en US".to_owned()]),
            Err(PresentationError::InvalidField)
        );
        assert!(validate_languages(&["zh-Hant-TW".to_owned()]).is_ok());
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

        let profile = PresentationProfile::new(
            screen,
            viewport,
            DevicePixelRatio::Quantized15,
            12,
            PresentationTimeZone::Utc,
            PresentationPlatform::MacOS,
            vec!["ko-KR".to_owned(), "en".to_owned()],
            true,
        )
        .expect("valid profile");
        assert_eq!(profile.device_pixel_ratio().value(), 1.5);
        assert_eq!(profile.hardware_concurrency(), 12);
        assert_eq!(profile.timezone_offset_minutes(), 0);
        assert_eq!(profile.timezone(), PresentationTimeZone::Utc);
        assert_eq!(profile.platform(), PresentationPlatform::MacOS);
        assert_eq!(profile.languages().len(), 2);
        assert!(profile.reduced_motion());
    }

    #[test]
    fn format_ratio_covers_each_quantized_class() {
        assert_eq!(format_ratio(1.0), "1");
        assert_eq!(format_ratio(1.5), "1.5");
        assert_eq!(format_ratio(2.0), "2");
    }

    #[test]
    fn hex_digit_lowercases_every_nibble() {
        for value in 0..16u8 {
            let expected = format!("{value:x}");
            assert_eq!(hex_digit(value).to_string(), expected);
        }
    }

    #[test]
    fn select_index_stays_within_bounds_for_small_and_large_sets() {
        for slot in 0..12usize {
            for size in [1usize, 2, 3, 8, 27] {
                let index = select_index(&seed(), slot, size);
                assert!(index < size);
            }
        }
        // A degenerate set of one collapses deterministically to zero.
        assert_eq!(select_index(&seed(), 0, 1), 0);
    }

    #[test]
    fn enumerated_sets_satisfy_their_public_validation_contracts() {
        // Every enumerated screen must pass the validating constructor, and
        // every enumerated viewport pair filtered to that screen likewise.
        for (screen_width, screen_height) in SCREEN_SET {
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
            assert!(HARDWARE_CONCURRENCY_SET.contains(&concurrency));
        }
        for language in FIRST_LANGUAGE_SET {
            assert!(validate_languages(&[language.to_owned()]).is_ok());
        }
        assert_eq!(SECOND_LANGUAGE, "en");
    }

    #[test]
    fn derive_is_stable_across_all_slots_of_two_seeds() {
        let other = PresentationSeed::new([1u8; 32]).expect("seed");
        let left = PresentationProfile::derive(&seed());
        let right = PresentationProfile::derive(&other);
        assert_ne!(left.digest(), right.digest());
        // Re-derivation reproduces the exact same digest text.
        assert_eq!(
            PresentationProfile::derive(&seed()).digest().as_str(),
            left.digest().as_str()
        );
    }

    #[test]
    fn derivation_exercises_optional_second_language() {
        assert_eq!(
            PresentationSeed::new([0; 32]),
            Err(PresentationError::DegenerateSeed)
        );
        let mut observed_lengths = std::collections::BTreeSet::new();
        for last_byte in 0..=u8::MAX {
            let mut bytes = [1u8; 32];
            bytes[31] = last_byte;
            let seed = PresentationSeed::new(bytes).expect("nonzero seed");
            observed_lengths.insert(PresentationProfile::derive(&seed).languages().len());
        }
        assert_eq!(observed_lengths, std::collections::BTreeSet::from([1, 2]));
    }
}
