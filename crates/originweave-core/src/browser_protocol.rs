use std::{fmt, str::FromStr};

/// Maximum UTF-8 byte length for browser protocol adapter metadata tokens.
pub const MAX_BROWSER_PROTOCOL_METADATA_BYTES: usize = 128;

/// One OriginWeave Protocol generation.
///
/// This value identifies the OriginWeave contract spoken by an adapter. It is
/// deliberately independent from the upstream WebDriver BiDi/CDP revision and
/// from the browser build. Constructing a version does not make that version
/// supported; callers must compare it with the exact version required by the
/// surrounding OriginWeave protocol boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginWeaveProtocolVersion {
    major: u16,
    minor: u16,
}

impl OriginWeaveProtocolVersion {
    /// Construct an OriginWeave Protocol generation identifier.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Return the protocol major version.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Return the protocol minor version.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

impl fmt::Display for OriginWeaveProtocolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "originweave/{}.{}", self.major, self.minor)
    }
}

impl FromStr for OriginWeaveProtocolVersion {
    type Err = OriginWeaveProtocolVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(remainder) = value.strip_prefix("originweave/") else {
            return Err(OriginWeaveProtocolVersionParseError::InvalidFormat);
        };
        let Some((major_text, minor_text)) = remainder.split_once('.') else {
            return Err(OriginWeaveProtocolVersionParseError::InvalidFormat);
        };
        if minor_text.contains('.') {
            return Err(OriginWeaveProtocolVersionParseError::InvalidFormat);
        }
        let Ok(major) = major_text.parse::<u16>() else {
            return Err(OriginWeaveProtocolVersionParseError::InvalidFormat);
        };
        let Ok(minor) = minor_text.parse::<u16>() else {
            return Err(OriginWeaveProtocolVersionParseError::InvalidFormat);
        };

        let version = Self::new(major, minor);
        if version.to_string() != value {
            return Err(OriginWeaveProtocolVersionParseError::InvalidFormat);
        }
        Ok(version)
    }
}

/// Failure to parse a canonical serialized OriginWeave Protocol generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginWeaveProtocolVersionParseError {
    /// The value did not use the exact canonical `originweave/<major>.<minor>` syntax.
    InvalidFormat,
}

impl fmt::Display for OriginWeaveProtocolVersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => formatter.write_str(
                "OriginWeave protocol version must use canonical originweave/<major>.<minor> syntax",
            ),
        }
    }
}

impl std::error::Error for OriginWeaveProtocolVersionParseError {}

/// Browser automation protocol family used by one versioned adapter.
///
/// The protocol family is descriptive metadata only. Selecting a kind does not
/// grant any OriginWeave capability or imply that a particular protocol
/// feature is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserProtocolKind {
    /// Standards-track WebDriver BiDi adapter.
    WebDriverBiDi,
    /// Chromium-specific Chrome DevTools Protocol adapter.
    ChromeDevToolsProtocol,
}

/// One browser operation surface explicitly implemented by an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserProtocolCapability {
    /// Navigate a controlled browser context through the adapter.
    Navigation,
    /// Produce bounded semantic browser observations.
    SemanticObservation,
    /// Dispatch typed browser input after OriginWeave policy authorization.
    TypedInput,
    /// Observe bounded network evidence needed by higher-level provenance.
    NetworkObservation,
}

/// Immutable version and capability metadata for one browser protocol adapter.
///
/// This value is deliberately not browser authority. It contains no browser
/// session, context, origin, node handle, action grant, credential, or network
/// permission. Higher layers may use it to fail closed when the adapter targets
/// the wrong OriginWeave Protocol generation or lacks a required browser
/// capability, while all OriginWeave authority remains separately validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserProtocolAdapterDescriptor {
    kind: BrowserProtocolKind,
    originweave_protocol_version: OriginWeaveProtocolVersion,
    adapter_version: String,
    protocol_revision: String,
    browser_revision: String,
    capabilities: Vec<BrowserProtocolCapability>,
}

impl BrowserProtocolAdapterDescriptor {
    /// Construct one explicit adapter descriptor.
    ///
    /// The OriginWeave Protocol generation, adapter version, upstream protocol
    /// revision, and browser revision are distinct metadata. This prevents an
    /// OriginWeave contract version from being mistaken for the WebDriver
    /// BiDi/CDP revision or the pinned browser build it was validated against.
    /// The declared capability list must be non-empty and duplicate-free and is
    /// normalized into one stable order so caller ordering cannot change
    /// descriptor identity.
    pub fn new(
        kind: BrowserProtocolKind,
        originweave_protocol_version: OriginWeaveProtocolVersion,
        adapter_version: &str,
        protocol_revision: &str,
        browser_revision: &str,
        capabilities: &[BrowserProtocolCapability],
    ) -> Result<Self, BrowserProtocolDescriptorError> {
        if !metadata_token_is_valid(adapter_version) {
            return Err(BrowserProtocolDescriptorError::InvalidAdapterVersion);
        }
        if !metadata_token_is_valid(protocol_revision) {
            return Err(BrowserProtocolDescriptorError::InvalidProtocolRevision);
        }
        if !metadata_token_is_valid(browser_revision) {
            return Err(BrowserProtocolDescriptorError::InvalidBrowserRevision);
        }
        if capabilities.is_empty() {
            return Err(BrowserProtocolDescriptorError::EmptyCapabilities);
        }

        let mut canonical_capabilities = Vec::with_capacity(capabilities.len());
        for capability in capabilities {
            if canonical_capabilities.contains(capability) {
                return Err(BrowserProtocolDescriptorError::DuplicateCapability);
            }
            canonical_capabilities.push(*capability);
        }
        canonical_capabilities.sort_unstable_by_key(|capability| capability_rank(*capability));

        Ok(Self {
            kind,
            originweave_protocol_version,
            adapter_version: adapter_version.to_owned(),
            protocol_revision: protocol_revision.to_owned(),
            browser_revision: browser_revision.to_owned(),
            capabilities: canonical_capabilities,
        })
    }

    /// Return the explicitly declared browser protocol family.
    #[must_use]
    pub const fn kind(&self) -> BrowserProtocolKind {
        self.kind
    }

    /// Return the exact OriginWeave Protocol generation implemented by this adapter.
    #[must_use]
    pub const fn originweave_protocol_version(&self) -> OriginWeaveProtocolVersion {
        self.originweave_protocol_version
    }

    /// Return the bounded OriginWeave adapter-version metadata token.
    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    /// Return the bounded upstream browser-protocol revision metadata token.
    #[must_use]
    pub fn protocol_revision(&self) -> &str {
        &self.protocol_revision
    }

    /// Return the bounded pinned browser-revision metadata token.
    #[must_use]
    pub fn browser_revision(&self) -> &str {
        &self.browser_revision
    }

    /// Return the number of explicitly declared capabilities.
    #[must_use]
    pub fn capability_count(&self) -> usize {
        self.capabilities.len()
    }

    /// Return whether this descriptor explicitly declares one capability.
    #[must_use]
    pub fn supports(&self, capability: BrowserProtocolCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Require one exact OriginWeave Protocol generation before later adapter use.
    ///
    /// Pre-alpha compatibility is deliberately exact at this boundary. A caller
    /// may add a separately reviewed compatibility transform later, but this
    /// descriptor never silently treats a different major or minor generation
    /// as equivalent.
    pub fn require_originweave_protocol_version(
        &self,
        required: OriginWeaveProtocolVersion,
    ) -> Result<(), BrowserProtocolVersionRequirementError> {
        if self.originweave_protocol_version == required {
            Ok(())
        } else {
            Err(
                BrowserProtocolVersionRequirementError::ProtocolVersionMismatch {
                    required,
                    actual: self.originweave_protocol_version,
                },
            )
        }
    }

    /// Require exact runtime browser-protocol and browser revisions before use.
    ///
    /// The caller must derive both values from the trusted runtime adapter that
    /// is about to perform browser work. This deterministic comparison does not
    /// authenticate or attest that caller. It only prevents a descriptor pinned
    /// to one validated upstream-protocol/browser pair from being silently used
    /// when the supplied runtime evidence is malformed or has drifted.
    pub fn require_runtime_revisions(
        &self,
        protocol_revision: &str,
        browser_revision: &str,
    ) -> Result<(), BrowserProtocolRuntimeRequirementError> {
        if !metadata_token_is_valid(protocol_revision) {
            return Err(BrowserProtocolRuntimeRequirementError::InvalidProtocolRevision);
        }
        if !metadata_token_is_valid(browser_revision) {
            return Err(BrowserProtocolRuntimeRequirementError::InvalidBrowserRevision);
        }
        if self.protocol_revision != protocol_revision {
            return Err(BrowserProtocolRuntimeRequirementError::ProtocolRevisionMismatch);
        }
        if self.browser_revision != browser_revision {
            return Err(BrowserProtocolRuntimeRequirementError::BrowserRevisionMismatch);
        }
        Ok(())
    }

    /// Require one explicitly declared adapter capability before later use.
    ///
    /// This method never infers support from the browser protocol family. An
    /// absent capability fails closed with a typed error so a caller cannot
    /// silently fall back to another upstream protocol or a raw browser escape
    /// hatch merely because the selected adapter lacks the requested surface.
    pub fn require_capability(
        &self,
        capability: BrowserProtocolCapability,
    ) -> Result<(), BrowserProtocolCapabilityRequirementError> {
        if self.supports(capability) {
            Ok(())
        } else {
            Err(BrowserProtocolCapabilityRequirementError::UnsupportedCapability(capability))
        }
    }
}

const fn capability_rank(capability: BrowserProtocolCapability) -> u8 {
    match capability {
        BrowserProtocolCapability::Navigation => 0,
        BrowserProtocolCapability::SemanticObservation => 1,
        BrowserProtocolCapability::TypedInput => 2,
        BrowserProtocolCapability::NetworkObservation => 3,
    }
}

fn capability_name(capability: BrowserProtocolCapability) -> &'static str {
    match capability {
        BrowserProtocolCapability::Navigation => "navigation",
        BrowserProtocolCapability::SemanticObservation => "semantic-observation",
        BrowserProtocolCapability::TypedInput => "typed-input",
        BrowserProtocolCapability::NetworkObservation => "network-observation",
    }
}

fn metadata_token_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_BROWSER_PROTOCOL_METADATA_BYTES
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        && value.bytes().any(|byte| byte.is_ascii_alphanumeric())
}

/// Failure to require one exact OriginWeave Protocol generation from an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserProtocolVersionRequirementError {
    /// The adapter targets a different OriginWeave Protocol generation.
    ProtocolVersionMismatch {
        /// Exact OriginWeave Protocol generation required by the caller.
        required: OriginWeaveProtocolVersion,
        /// Exact OriginWeave Protocol generation declared by the adapter.
        actual: OriginWeaveProtocolVersion,
    },
}

impl fmt::Display for BrowserProtocolVersionRequirementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtocolVersionMismatch { required, actual } => write!(
                formatter,
                "browser protocol adapter targets {actual} but {required} is required"
            ),
        }
    }
}

impl std::error::Error for BrowserProtocolVersionRequirementError {}

/// Failure to require exact pinned runtime revision evidence from an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserProtocolRuntimeRequirementError {
    /// The runtime upstream-protocol revision token was malformed.
    InvalidProtocolRevision,
    /// The runtime browser revision token was malformed.
    InvalidBrowserRevision,
    /// The runtime upstream-protocol revision differs from the pinned descriptor.
    ProtocolRevisionMismatch,
    /// The runtime browser revision differs from the pinned descriptor.
    BrowserRevisionMismatch,
}

impl fmt::Display for BrowserProtocolRuntimeRequirementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProtocolRevision => formatter.write_str(
                "runtime browser protocol revision must be a bounded ASCII metadata token",
            ),
            Self::InvalidBrowserRevision => formatter
                .write_str("runtime browser revision must be a bounded ASCII metadata token"),
            Self::ProtocolRevisionMismatch => formatter.write_str(
                "runtime browser protocol revision does not match the pinned adapter revision",
            ),
            Self::BrowserRevisionMismatch => formatter.write_str(
                "runtime browser revision does not match the pinned adapter browser revision",
            ),
        }
    }
}

impl std::error::Error for BrowserProtocolRuntimeRequirementError {}

/// Failure to require one browser protocol capability from an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserProtocolCapabilityRequirementError {
    /// The adapter did not explicitly declare the required capability.
    UnsupportedCapability(BrowserProtocolCapability),
}

impl fmt::Display for BrowserProtocolCapabilityRequirementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCapability(capability) => write!(
                formatter,
                "browser protocol adapter does not declare required {} capability",
                capability_name(*capability)
            ),
        }
    }
}

impl std::error::Error for BrowserProtocolCapabilityRequirementError {}

/// Failure to construct canonical browser protocol adapter metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserProtocolDescriptorError {
    /// The adapter-version token was empty, oversized, non-ASCII, or malformed.
    InvalidAdapterVersion,
    /// The upstream protocol-revision token was empty, oversized, non-ASCII, or malformed.
    InvalidProtocolRevision,
    /// The browser-revision token was empty, oversized, non-ASCII, or malformed.
    InvalidBrowserRevision,
    /// The adapter declared no supported browser capability.
    EmptyCapabilities,
    /// The adapter declared the same capability more than once.
    DuplicateCapability,
}

impl fmt::Display for BrowserProtocolDescriptorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAdapterVersion => formatter.write_str(
                "browser protocol adapter version must be a bounded ASCII metadata token",
            ),
            Self::InvalidProtocolRevision => formatter
                .write_str("browser protocol revision must be a bounded ASCII metadata token"),
            Self::InvalidBrowserRevision => {
                formatter.write_str("browser revision must be a bounded ASCII metadata token")
            }
            Self::EmptyCapabilities => {
                formatter.write_str("browser protocol adapter must declare at least one capability")
            }
            Self::DuplicateCapability => {
                formatter.write_str("browser protocol adapter capabilities must be unique")
            }
        }
    }
}

impl std::error::Error for BrowserProtocolDescriptorError {}
