//! Bounded User-Agent Client Hints surfaces for browser presentation.
//!
//! A page can request high-entropy UA Client Hints — architecture, bitness,
//! platform, platform version, model — in addition to the low-entropy
//! brand/mobile hints a Chromium user agent sends on every request. If an
//! adapter presents a static profile but lets the real UA-CH surface leak,
//! a page reconciles the contradiction and the host is reidentified. The
//! User-Agent Client Hints specification (WICG, 2026) bounds the low-entropy
//! platform object and requires non-mobile user agents to report an empty
//! model. This module exposes the deterministic contract those hints must
//! satisfy while performing no evasion and never reading the host.

use std::error::Error;
use std::fmt;

/// The maximum accepted brand-name length in ASCII bytes.
const MAX_BRAND_NAME_LENGTH: usize = 32;

/// The maximum accepted brand-version length in ASCII bytes.
const MAX_BRAND_VERSION_LENGTH: usize = 32;

/// The maximum accepted mobile-model length in UTF-8 bytes.
const MAX_MOBILE_MODEL_LENGTH: usize = 64;

/// WICG GREASE-compatible separators admitted inside bounded brand names.
const BRAND_COMPATIBILITY_SEPARATORS: &[u8] = b" ()-./:;=?_";

fn is_valid_brand_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || BRAND_COMPATIBILITY_SEPARATORS.contains(&byte)
}

/// A validation failure when assembling a UA Client Hints surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientHintsError {
    /// A brand name exceeded the bounded ASCII length.
    BrandTooLong,
    /// A brand version exceeded the OriginWeave resource budget.
    BrandVersionTooLong,
    /// A brand name or version violated the bounded compatibility grammar.
    InvalidBrandName,
    /// A platform token was outside the enumerated low-entropy set.
    InvalidPlatform,
    /// A non-mobile user agent reported a non-empty model.
    ModelWithoutMobile,
    /// A mobile model exceeded the OriginWeave resource budget.
    ModelTooLong,
    /// A client-hints set carried no brand.
    MissingBrand,
}

impl fmt::Display for ClientHintsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrandTooLong => {
                formatter.write_str("brand name must be at most 32 ASCII characters")
            }
            Self::BrandVersionTooLong => {
                formatter.write_str("brand version must be at most 32 ASCII characters")
            }
            Self::InvalidBrandName => formatter.write_str(
                "brand name must use bounded UA-CH-compatible ASCII and version must be non-empty dotted ASCII alphanumeric",
            ),
            Self::InvalidPlatform => formatter.write_str(
                "platform must be one of the enumerated UA Client Hints platform values",
            ),
            Self::ModelWithoutMobile => {
                formatter.write_str("a non-mobile user agent must report an empty model")
            }
            Self::ModelTooLong => formatter.write_str("mobile model must be at most 64 bytes"),
            Self::MissingBrand => {
                formatter.write_str("a client-hints value must contain at least one brand")
            }
        }
    }
}

impl Error for ClientHintsError {}

/// One brand/version pair from a UA brand list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UaBrand {
    name: String,
    version: String,
}

impl UaBrand {
    /// Validate one brand/version token pair.
    ///
    /// Names must be non-empty ASCII and may contain alphanumerics plus the
    /// separator bytes used by the WICG GREASE brand algorithm. Versions must
    /// be non-empty dotted ASCII alphanumeric strings. The 32-byte name and
    /// version caps are OriginWeave resource bounds, not UA Client Hints
    /// specification limits.
    pub fn new(name: &str, version: &str) -> Result<Self, ClientHintsError> {
        if name.len() > MAX_BRAND_NAME_LENGTH {
            return Err(ClientHintsError::BrandTooLong);
        }
        if version.len() > MAX_BRAND_VERSION_LENGTH {
            return Err(ClientHintsError::BrandVersionTooLong);
        }
        if name.is_empty()
            || !name.bytes().all(is_valid_brand_name_byte)
            || version.is_empty()
            || !version
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.')
        {
            return Err(ClientHintsError::InvalidBrandName);
        }
        Ok(Self {
            name: name.to_owned(),
            version: version.to_owned(),
        })
    }

    /// Return the brand name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the brand version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// A bounded CPU-architecture token from the UA Client Hints hint set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintsArchitecture {
    /// The `x86` architecture token.
    X86,
    /// The `arm` architecture token.
    Arm,
}

impl HintsArchitecture {
    /// Map a submitted hint token onto a bounded architecture class.
    ///
    /// Unknown architecture values fail closed rather than widening the set.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "x86" => Some(Self::X86),
            "arm" => Some(Self::Arm),
            _ => None,
        }
    }

    /// Return the exact architecture token this class represents.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::Arm => "arm",
        }
    }
}

/// A bounded CPU bitness token from the UA Client Hints hint set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintsBitness {
    /// The `32` bitness token.
    Bit32,
    /// The `64` bitness token.
    Bit64,
}

impl HintsBitness {
    /// Map a recognized bitness token onto a class, rejecting others.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "32" => Some(Self::Bit32),
            "64" => Some(Self::Bit64),
            _ => None,
        }
    }

    /// Return the canonical bitness token this class represents.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Bit32 => "32",
            Self::Bit64 => "64",
        }
    }
}

/// A low-entropy platform token a user agent reports by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintsPlatform {
    /// The `Windows` platform token.
    Windows,
    /// The `macOS` platform token.
    MacOs,
    /// The `Linux` platform token.
    Linux,
}

impl HintsPlatform {
    /// Normalize a reported platform token onto an enumerated class.
    pub fn normalize(token: &str) -> Result<Self, ClientHintsError> {
        match token {
            "Windows" => Ok(Self::Windows),
            "macOS" => Ok(Self::MacOs),
            "Linux" => Ok(Self::Linux),
            _ => Err(ClientHintsError::InvalidPlatform),
        }
    }

    /// Return the canonical platform token this class represents.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOs => "macOS",
            Self::Linux => "Linux",
        }
    }
}

/// A validated, bounded UA Client Hints surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UaClientHints {
    platform: HintsPlatform,
    architecture: HintsArchitecture,
    bitness: HintsBitness,
    mobile: bool,
    model: String,
    brands: Vec<UaBrand>,
}

impl UaClientHints {
    /// Validate and build a UA Client Hints surface.
    ///
    /// The model must be empty when `mobile` is false. Mobile model values are
    /// capped at 64 UTF-8 bytes by OriginWeave's local resource budget. The
    /// brand list must be non-empty, and every brand is validated by
    /// [`UaBrand::new`].
    pub fn new(
        platform: HintsPlatform,
        architecture: HintsArchitecture,
        bitness: HintsBitness,
        mobile: bool,
        model: &str,
        brands: Vec<UaBrand>,
    ) -> Result<Self, ClientHintsError> {
        if !mobile && !model.is_empty() {
            return Err(ClientHintsError::ModelWithoutMobile);
        }
        if model.len() > MAX_MOBILE_MODEL_LENGTH {
            return Err(ClientHintsError::ModelTooLong);
        }
        if brands.is_empty() {
            return Err(ClientHintsError::MissingBrand);
        }
        Ok(Self {
            platform,
            architecture,
            bitness,
            mobile,
            model: model.to_owned(),
            brands,
        })
    }

    /// Return the low-entropy platform token.
    #[must_use]
    pub const fn platform(&self) -> HintsPlatform {
        self.platform
    }

    /// Return the enumerated architecture class.
    #[must_use]
    pub const fn architecture(&self) -> HintsArchitecture {
        self.architecture
    }

    /// Return the enumerated bitness class.
    #[must_use]
    pub const fn bitness(&self) -> HintsBitness {
        self.bitness
    }

    /// Return whether this user agent prefers a mobile experience.
    #[must_use]
    pub const fn mobile(&self) -> bool {
        self.mobile
    }

    /// Return the model name, empty for non-mobile user agents.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Return the validated brand list.
    #[must_use]
    pub fn brands(&self) -> &[UaBrand] {
        &self.brands
    }
}
