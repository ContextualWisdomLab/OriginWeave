use std::fmt;

/// Maximum UTF-8 byte length for browser protocol adapter metadata tokens.
pub const MAX_BROWSER_PROTOCOL_METADATA_BYTES: usize = 128;

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
/// permission. Higher layers may use it to fail closed when a required adapter
/// capability is absent, while all OriginWeave authority remains separately
/// validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserProtocolAdapterDescriptor {
    kind: BrowserProtocolKind,
    adapter_version: String,
    protocol_revision: String,
    browser_revision: String,
    capabilities: Vec<BrowserProtocolCapability>,
}

impl BrowserProtocolAdapterDescriptor {
    /// Construct one explicit adapter descriptor.
    ///
    /// Adapter version, upstream protocol revision, and browser revision are
    /// separate bounded ASCII metadata tokens. This prevents an OriginWeave
    /// adapter release from being mistaken for the WebDriver BiDi/CDP revision
    /// or the pinned browser build it was validated against. The declared
    /// capability list must be non-empty and duplicate-free so the descriptor
    /// has one canonical interpretation.
    pub fn new(
        kind: BrowserProtocolKind,
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

        Ok(Self {
            kind,
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
