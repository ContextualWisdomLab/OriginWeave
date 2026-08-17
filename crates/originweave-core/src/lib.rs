//! Shared security and governance contracts for OriginWeave.
//!
//! The crate deliberately contains no browser-engine integration. It defines
//! small, deterministic value types that can be reused by the browser shell,
//! headless runtime, MCP adapter, and enterprise policy service.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::BTreeSet;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

/// A normalized web origin accepted by the OriginWeave trust boundary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Origin {
    canonical: String,
}

impl Origin {
    /// Parse one origin and reject paths, credentials, fragments, insecure
    /// remote HTTP endpoints, and browser-special numeric host spellings.
    pub fn parse(input: &str) -> Result<Self, OriginError> {
        if input.trim() != input
            || input
                .chars()
                .any(|character| character.is_control() || character.is_whitespace())
        {
            return Err(OriginError::InvalidAuthority);
        }

        let Some((raw_scheme, authority)) = input.split_once("://") else {
            return Err(OriginError::MissingScheme);
        };
        let scheme = raw_scheme.to_ascii_lowercase();
        if scheme != "https" && scheme != "http" {
            return Err(OriginError::UnsupportedScheme);
        }
        if authority.is_empty() {
            return Err(OriginError::MissingAuthority);
        }
        if authority.contains('@') {
            return Err(OriginError::UserInfoNotAllowed);
        }
        if authority
            .chars()
            .any(|character| matches!(character, '/' | '?' | '#'))
        {
            return Err(OriginError::PathNotAllowed);
        }

        let (host, port, is_loopback) = parse_authority(authority)?;
        if scheme == "http" && !is_loopback {
            return Err(OriginError::InsecureRemoteOrigin);
        }
        let normalized_port = normalize_default_port(&scheme, port);
        let canonical = match normalized_port {
            Some(port_number) => format!("{scheme}://{host}:{port_number}"),
            None => format!("{scheme}://{host}"),
        };
        Ok(Self { canonical })
    }

    /// Return the normalized origin string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    /// Return the validated lowercase origin scheme.
    #[must_use]
    pub fn scheme(&self) -> &str {
        if self.canonical.starts_with("https://") {
            "https"
        } else {
            "http"
        }
    }

    /// Return the validated canonical host without IPv6 brackets.
    #[must_use]
    pub fn host(&self) -> &str {
        let authority = &self.canonical[self.scheme().len() + 3..];
        let bracketed = authority.starts_with('[');
        let host_start = usize::from(bracketed);
        let host_end = if bracketed {
            authority.find(']').unwrap_or(authority.len())
        } else {
            authority.find(':').unwrap_or(authority.len())
        };
        &authority[host_start..host_end]
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn normalize_default_port(scheme: &str, port: Option<u16>) -> Option<u16> {
    match (scheme, port) {
        ("https", Some(443)) | ("http", Some(80)) => None,
        (_, other) => other,
    }
}

fn parse_authority(authority: &str) -> Result<(String, Option<u16>, bool), OriginError> {
    if authority.starts_with('[') {
        return parse_bracketed_ipv6(authority);
    }
    if authority.matches(':').count() > 1 {
        return Err(OriginError::InvalidAuthority);
    }

    let (host_text, port) = match authority.rsplit_once(':') {
        Some((host, port_text)) => (host, Some(parse_port(port_text)?)),
        None => (authority, None),
    };
    let host = host_text.to_ascii_lowercase();
    if let Ok(address) = host.parse::<Ipv4Addr>() {
        return Ok((host, port, address.is_loopback()));
    }
    if looks_like_browser_ipv4_host(&host) {
        return Err(OriginError::AmbiguousNumericHost);
    }
    validate_dns_host(&host)?;
    Ok((host.clone(), port, host == "localhost"))
}

fn looks_like_browser_ipv4_host(host: &str) -> bool {
    host.rsplit('.')
        .next()
        .is_some_and(looks_like_browser_ipv4_number)
}

fn looks_like_browser_ipv4_number(label: &str) -> bool {
    if label.is_empty() {
        return false;
    }
    let lowercase = label.to_ascii_lowercase();
    if let Some(hexadecimal) = lowercase.strip_prefix("0x") {
        return !hexadecimal.is_empty() && hexadecimal.bytes().all(|byte| byte.is_ascii_hexdigit());
    }
    label.bytes().all(|byte| byte.is_ascii_digit())
}

fn parse_bracketed_ipv6(authority: &str) -> Result<(String, Option<u16>, bool), OriginError> {
    let Some(close_index) = authority.find(']') else {
        return Err(OriginError::InvalidAuthority);
    };
    let address_text = &authority[1..close_index];
    let address = address_text
        .parse::<Ipv6Addr>()
        .map_err(|_error| OriginError::InvalidAuthority)?;
    let remainder = &authority[close_index + 1..];
    let port = if remainder.is_empty() {
        None
    } else if let Some(port_text) = remainder.strip_prefix(':') {
        Some(parse_port(port_text)?)
    } else {
        return Err(OriginError::InvalidAuthority);
    };
    Ok((format!("[{address}]"), port, address.is_loopback()))
}

fn parse_port(port_text: &str) -> Result<u16, OriginError> {
    let port = port_text
        .parse::<u16>()
        .map_err(|_error| OriginError::InvalidPort)?;
    if port == 0 {
        return Err(OriginError::InvalidPort);
    }
    Ok(port)
}

fn validate_dns_host(host: &str) -> Result<(), OriginError> {
    if host.is_empty() {
        return Err(OriginError::InvalidAuthority);
    }
    if host.len() > 253 {
        return Err(OriginError::InvalidAuthority);
    }
    if !host.is_ascii() {
        return Err(OriginError::InvalidAuthority);
    }
    if host.starts_with('.') || host.ends_with('.') {
        return Err(OriginError::InvalidAuthority);
    }
    for label in host.split('.') {
        if label.is_empty() {
            return Err(OriginError::InvalidAuthority);
        }
        if label.len() > 63 {
            return Err(OriginError::InvalidAuthority);
        }
        let bytes = label.as_bytes();
        if !bytes[0].is_ascii_alphanumeric() {
            return Err(OriginError::InvalidAuthority);
        }
        if !bytes[bytes.len() - 1].is_ascii_alphanumeric() {
            return Err(OriginError::InvalidAuthority);
        }
        if !bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
        {
            return Err(OriginError::InvalidAuthority);
        }
    }
    Ok(())
}

/// A reason that an origin string could not enter the trust boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginError {
    /// The input did not contain a `scheme://` separator.
    MissingScheme,
    /// The scheme was neither HTTPS nor locally scoped HTTP.
    UnsupportedScheme,
    /// HTTP was requested for a non-loopback host.
    InsecureRemoteOrigin,
    /// No authority followed the scheme.
    MissingAuthority,
    /// User information appeared before the host.
    UserInfoNotAllowed,
    /// A path, query, or fragment was supplied where only an origin is valid.
    PathNotAllowed,
    /// The host or authority syntax was ambiguous or malformed.
    InvalidAuthority,
    /// A browser could reinterpret the host as a non-canonical IPv4 address.
    AmbiguousNumericHost,
    /// The explicit port was outside `1..=65535` or was not numeric.
    InvalidPort,
}

/// A nonzero identity for one active browser automation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrowserSessionId(u64);

impl BrowserSessionId {
    /// Validate one adapter-supplied browser-session identifier.
    pub const fn new(value: u64) -> Result<Self, NodeHandleError> {
        if value == 0 {
            return Err(NodeHandleError::InvalidBrowserSessionId);
        }
        Ok(Self(value))
    }

    /// Return the validated browser-session identifier.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A nonzero identity for one independently navigable browser context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrowsingContextId(u64);

impl BrowsingContextId {
    /// Validate one adapter-supplied browsing-context identifier.
    pub const fn new(value: u64) -> Result<Self, NodeHandleError> {
        if value == 0 {
            return Err(NodeHandleError::InvalidBrowsingContextId);
        }
        Ok(Self(value))
    }

    /// Return the validated browsing-context identifier.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A nonzero identity for one observed browser document lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentEpoch(u64);

impl DocumentEpoch {
    /// Validate one adapter-supplied document epoch.
    pub const fn new(value: u64) -> Result<Self, NodeHandleError> {
        if value == 0 {
            return Err(NodeHandleError::InvalidDocumentEpoch);
        }
        Ok(Self(value))
    }

    /// Return the validated document epoch value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Rotate the epoch when a same-document mutation can change actionable identity.
    ///
    /// Re-observe the current document and use the new handle. Do not reuse a
    /// handle emitted at the previous epoch after a relevant mutation.
    pub const fn after_same_document_mutation(
        self,
        mutation: SameDocumentMutationKind,
    ) -> Result<Self, NodeHandleError> {
        if !mutation.requires_epoch_increment() {
            return Ok(self);
        }
        match self.0.checked_add(1) {
            Some(next) => Self::new(next),
            None => Err(NodeHandleError::DocumentEpochOverflow),
        }
    }
}

/// A same-document lifecycle event that may invalidate actionable node handles.
///
/// This is a control-plane decision input. It does not observe the live DOM,
/// accessibility tree, or browser process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameDocumentMutationKind {
    /// The previously observed target node was removed from the document.
    TargetRemoved,
    /// The previously observed target was replaced by a different node.
    TargetReplaced,
    /// The target's actionable role, accessible name, or actionability changed.
    RoleOrNameChanged,
    /// A relevant accessibility-tree invalidation changed the target's meaning.
    AccessibilityTreeInvalidated,
    /// A nested frame document was replaced while the parent document remained.
    FrameDocumentReplaced,
    /// A subtree mutation replaced the observed actionable target.
    ActionableSubtreeReplaced,
    /// A reviewed non-semantic change that cannot affect any emitted handle.
    NonSemanticUnrelated,
}

impl SameDocumentMutationKind {
    /// Return whether this mutation must increment the document epoch.
    ///
    /// Uncertainty must use a relevant variant so the old handle cannot be reused.
    #[must_use]
    pub const fn requires_epoch_increment(self) -> bool {
        match self {
            Self::NonSemanticUnrelated => false,
            Self::TargetRemoved
            | Self::TargetReplaced
            | Self::RoleOrNameChanged
            | Self::AccessibilityTreeInvalidated
            | Self::FrameDocumentReplaced
            | Self::ActionableSubtreeReplaced => true,
        }
    }
}

/// A node identity bound to the exact session, context, origin, and document that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedNodeHandle {
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    origin: Origin,
    document_epoch: DocumentEpoch,
    node_id: u64,
}

impl ObservedNodeHandle {
    /// Create one authority-bound observed node handle from a nonzero adapter node identifier.
    pub fn new(
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: Origin,
        document_epoch: DocumentEpoch,
        node_id: u64,
    ) -> Result<Self, NodeHandleError> {
        if node_id == 0 {
            return Err(NodeHandleError::InvalidNodeId);
        }
        Ok(Self {
            browser_session,
            browsing_context,
            origin,
            document_epoch,
            node_id,
        })
    }

    /// Return the browser session that produced the node observation.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the browsing context that produced the node observation.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.browsing_context
    }

    /// Return the canonical origin that produced the node observation.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the document epoch that produced the node observation.
    #[must_use]
    pub const fn document_epoch(&self) -> DocumentEpoch {
        self.document_epoch
    }

    /// Return the adapter-local nonzero node identifier.
    #[must_use]
    pub const fn node_id(&self) -> u64 {
        self.node_id
    }

    /// Reject use when the session, browsing context, origin, or document epoch has changed.
    pub fn validate_current(
        &self,
        current_session: BrowserSessionId,
        current_context: BrowsingContextId,
        current_origin: &Origin,
        current_epoch: DocumentEpoch,
    ) -> Result<(), NodeHandleError> {
        if self.browser_session != current_session {
            return Err(NodeHandleError::BrowserSessionMismatch {
                observed: self.browser_session,
                current: current_session,
            });
        }
        if self.browsing_context != current_context {
            return Err(NodeHandleError::BrowsingContextMismatch {
                observed: self.browsing_context,
                current: current_context,
            });
        }
        if &self.origin != current_origin {
            return Err(NodeHandleError::OriginMismatch);
        }
        if self.document_epoch != current_epoch {
            return Err(NodeHandleError::StaleDocumentEpoch {
                observed: self.document_epoch,
                current: current_epoch,
            });
        }
        Ok(())
    }
}

/// A failure to construct or reuse an authority- and document-bound node handle safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeHandleError {
    /// Browser-session identifiers are one-based and zero was supplied.
    InvalidBrowserSessionId,
    /// Browsing-context identifiers are one-based and zero was supplied.
    InvalidBrowsingContextId,
    /// Document epochs are one-based and zero was supplied.
    InvalidDocumentEpoch,
    /// Adapter-local node identifiers are one-based and zero was supplied.
    InvalidNodeId,
    /// The node handle belongs to a different browser automation session.
    BrowserSessionMismatch {
        /// Session that originally produced the node handle.
        observed: BrowserSessionId,
        /// Session currently active for the requested action.
        current: BrowserSessionId,
    },
    /// The node handle belongs to a different independently navigable context.
    BrowsingContextMismatch {
        /// Context that originally produced the node handle.
        observed: BrowsingContextId,
        /// Context currently active for the requested action.
        current: BrowsingContextId,
    },
    /// The browser context is now at a different canonical origin.
    OriginMismatch,
    /// The browser context is now at a different document epoch.
    StaleDocumentEpoch {
        /// Epoch that originally produced the node handle.
        observed: DocumentEpoch,
        /// Epoch currently active in the browser context.
        current: DocumentEpoch,
    },
    /// Incrementing the document epoch would wrap the identifier space.
    DocumentEpochOverflow,
}

impl fmt::Display for NodeHandleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBrowserSessionId => {
                formatter.write_str("browser session identifier must be nonzero")
            }
            Self::InvalidBrowsingContextId => {
                formatter.write_str("browsing context identifier must be nonzero")
            }
            Self::InvalidDocumentEpoch => formatter.write_str("document epoch must be nonzero"),
            Self::InvalidNodeId => formatter.write_str("observed node identifier must be nonzero"),
            Self::BrowserSessionMismatch { observed, current } => write!(
                formatter,
                "observed node browser session {} does not match current session {}",
                observed.value(),
                current.value()
            ),
            Self::BrowsingContextMismatch { observed, current } => write!(
                formatter,
                "observed node browsing context {} does not match current context {}",
                observed.value(),
                current.value()
            ),
            Self::OriginMismatch => {
                formatter.write_str("observed node origin does not match the current origin")
            }
            Self::StaleDocumentEpoch { observed, current } => write!(
                formatter,
                "observed node document epoch {} is stale; current epoch is {}",
                observed.value(),
                current.value()
            ),
            Self::DocumentEpochOverflow => {
                formatter.write_str("document epoch cannot wrap after a same-document mutation")
            }
        }
    }
}

impl std::error::Error for NodeHandleError {}

/// An immutable digest of the complete canonical action intent.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionIntentDigest {
    canonical: String,
}

impl ActionIntentDigest {
    /// Parse a lowercase `sha256:` digest of the complete canonical intent.
    pub fn parse(input: &str) -> Result<Self, ActionIntentDigestError> {
        let Some(hexadecimal) = input.strip_prefix("sha256:") else {
            return Err(ActionIntentDigestError::InvalidFormat);
        };
        if hexadecimal.len() != 64
            || !hexadecimal
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ActionIntentDigestError::InvalidFormat);
        }
        Ok(Self {
            canonical: input.to_owned(),
        })
    }

    /// Return the canonical lowercase digest.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
}

/// A validation error for an action-intent digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionIntentDigestError {
    /// The value was not `sha256:` followed by 64 lowercase hexadecimal digits.
    InvalidFormat,
}

/// The browser execution mode that owns an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SessionMode {
    /// A person controls the browser without agent execution privileges.
    Human,
    /// An agent assists a person while write actions remain governed.
    Assist,
    /// An isolated task session is delegated to an agent.
    AgentTask,
    /// A read-only crawler performs policy-bounded collection.
    Crawler,
}

/// The declared business purpose of one browser execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExecutionPurpose {
    /// Public content is collected under crawler policy.
    PublicCrawl,
    /// A person delegated a bounded task in their own context.
    UserDelegatedTask,
    /// An enterprise policy authorized a managed task.
    EnterpriseAuthorizedTask,
    /// The action is running in a non-production test environment.
    TestingEnvironment,
}

/// The trust class of the instruction that proposed an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstructionSource {
    /// A human user supplied the instruction.
    User,
    /// A managed enterprise policy supplied the instruction.
    EnterprisePolicy,
    /// Untrusted page or document content supplied the instruction.
    WebContent,
}

/// The result of applying a robots-exclusion policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RobotsDecision {
    /// The requested crawl is explicitly allowed.
    Allowed,
    /// The requested crawl is explicitly disallowed.
    Disallowed,
    /// The policy could not be fetched or interpreted safely.
    Unknown,
    /// Robots policy was not evaluated for this execution purpose.
    NotApplicable,
}

/// How secret material is delivered to a browser action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecretDelivery {
    /// The action carries no secret material.
    None,
    /// A trusted broker resolves an opaque secret handle outside the model.
    BrokerHandle,
    /// A raw secret value would be exposed directly to the caller.
    RawValue,
}

/// The ordered risk class assigned to an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskClass {
    /// Read-only observation with no state change.
    R0,
    /// Low-risk navigation or local retrieval.
    R1,
    /// Reversible preparation such as creating a draft.
    R2,
    /// External submission or sensitive interaction requiring approval.
    R3,
    /// High-impact purchase, deletion, or permission change.
    R4,
    /// Legal or similarly non-delegable consent.
    R5,
}

impl RiskClass {
    /// Return whether the risk class requires approval before execution.
    #[must_use]
    pub const fn requires_approval(self) -> bool {
        matches!(self, Self::R3 | Self::R4 | Self::R5)
    }
}

/// A capability that may be granted to an isolated agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// Observe a page's governed semantic representation.
    Observe,
    /// Extract structured information from allowed evidence.
    Extract,
    /// Navigate to an allowed origin.
    Navigate,
    /// Download a resource from an allowed origin.
    Download,
    /// Prepare a reversible draft.
    Draft,
    /// Submit data to an allowed origin.
    Submit,
    /// Upload a pre-approved artifact.
    Upload,
    /// Fill a secret through the trusted secret broker.
    FillSecret,
    /// Complete a purchase after approval.
    Purchase,
    /// Delete a remote object after approval.
    Delete,
    /// Change a permission after approval.
    ManagePermission,
    /// Record legal consent, which agents cannot perform autonomously.
    LegalConsent,
}

/// A typed browser action exposed to policy evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActionKind {
    /// Observe governed page state.
    Observe,
    /// Extract structured data.
    Extract,
    /// Navigate the browser.
    Navigate,
    /// Download a resource.
    Download,
    /// Create or update a reversible draft.
    Draft,
    /// Submit data externally.
    Submit,
    /// Upload an approved file.
    Upload,
    /// Fill a secret using an opaque broker handle.
    FillSecret,
    /// Complete a purchase.
    Purchase,
    /// Delete remote state.
    Delete,
    /// Change access permissions.
    ManagePermission,
    /// Accept legally binding terms.
    LegalConsent,
}

impl ActionKind {
    /// Return the action's fixed risk classification.
    #[must_use]
    pub const fn risk_class(self) -> RiskClass {
        match self {
            Self::Observe | Self::Extract => RiskClass::R0,
            Self::Navigate | Self::Download => RiskClass::R1,
            Self::Draft => RiskClass::R2,
            Self::Submit | Self::Upload | Self::FillSecret => RiskClass::R3,
            Self::Purchase | Self::Delete | Self::ManagePermission => RiskClass::R4,
            Self::LegalConsent => RiskClass::R5,
        }
    }

    /// Return the capability required to request this action.
    #[must_use]
    pub const fn required_capability(self) -> Capability {
        match self {
            Self::Observe => Capability::Observe,
            Self::Extract => Capability::Extract,
            Self::Navigate => Capability::Navigate,
            Self::Download => Capability::Download,
            Self::Draft => Capability::Draft,
            Self::Submit => Capability::Submit,
            Self::Upload => Capability::Upload,
            Self::FillSecret => Capability::FillSecret,
            Self::Purchase => Capability::Purchase,
            Self::Delete => Capability::Delete,
            Self::ManagePermission => Capability::ManagePermission,
            Self::LegalConsent => Capability::LegalConsent,
        }
    }

    /// Return whether execution can mutate browser or remote state.
    #[must_use]
    pub const fn mutates_state(self) -> bool {
        !matches!(
            self,
            Self::Observe | Self::Extract | Self::Navigate | Self::Download
        )
    }

    /// Return whether this action is designed to resolve a brokered secret.
    #[must_use]
    pub const fn uses_secret(self) -> bool {
        matches!(self, Self::FillSecret)
    }
}

/// The exact action, target origin, and complete intent covered by an approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalScope {
    action: ActionKind,
    target_origin: Origin,
    intent_digest: ActionIntentDigest,
}

impl ApprovalScope {
    /// Create one exact approval scope.
    #[must_use]
    pub const fn new(
        action: ActionKind,
        target_origin: Origin,
        intent_digest: ActionIntentDigest,
    ) -> Self {
        Self {
            action,
            target_origin,
            intent_digest,
        }
    }

    /// Return the approved action kind.
    #[must_use]
    pub const fn action(&self) -> ActionKind {
        self.action
    }

    /// Return the approved target origin.
    #[must_use]
    pub const fn target_origin(&self) -> &Origin {
        &self.target_origin
    }

    /// Return the approved complete-intent digest.
    #[must_use]
    pub const fn intent_digest(&self) -> &ActionIntentDigest {
        &self.intent_digest
    }
}

/// Evidence that a high-risk action was approved for an exact scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalEvidence {
    /// No approval was supplied.
    None,
    /// A person confirmed the exact action, target, and complete intent.
    UserConfirmed(ApprovalScope),
    /// A managed policy approved the exact action, target, and complete intent.
    EnterprisePolicy(ApprovalScope),
}

impl ApprovalEvidence {
    /// Return whether this evidence authorizes the exact required scope.
    #[must_use]
    pub fn authorizes(&self, required: &ApprovalScope) -> bool {
        match self {
            Self::None => false,
            Self::UserConfirmed(scope) | Self::EnterprisePolicy(scope) => scope == required,
        }
    }
}

/// A complete typed request presented to the policy engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRequest {
    action: ActionKind,
    source_origin: Origin,
    target_origin: Origin,
    instruction_source: InstructionSource,
    secret_delivery: SecretDelivery,
    intent_digest: ActionIntentDigest,
}

impl ActionRequest {
    /// Create one action request without executing it.
    #[must_use]
    pub const fn new(
        action: ActionKind,
        source_origin: Origin,
        target_origin: Origin,
        instruction_source: InstructionSource,
        secret_delivery: SecretDelivery,
        intent_digest: ActionIntentDigest,
    ) -> Self {
        Self {
            action,
            source_origin,
            target_origin,
            instruction_source,
            secret_delivery,
            intent_digest,
        }
    }

    /// Return the requested action.
    #[must_use]
    pub const fn action(&self) -> ActionKind {
        self.action
    }

    /// Return the origin that currently owns the browser context.
    #[must_use]
    pub const fn source_origin(&self) -> &Origin {
        &self.source_origin
    }

    /// Return the origin affected by the action.
    #[must_use]
    pub const fn target_origin(&self) -> &Origin {
        &self.target_origin
    }

    /// Return the trust class of the proposing instruction.
    #[must_use]
    pub const fn instruction_source(&self) -> InstructionSource {
        self.instruction_source
    }

    /// Return how secret material would be delivered.
    #[must_use]
    pub const fn secret_delivery(&self) -> SecretDelivery {
        self.secret_delivery
    }

    /// Return the digest of the complete canonical action intent.
    #[must_use]
    pub const fn intent_digest(&self) -> &ActionIntentDigest {
        &self.intent_digest
    }
}

/// Immutable grants and mutable evidence used for one policy decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyContext {
    mode: SessionMode,
    purpose: ExecutionPurpose,
    capabilities: BTreeSet<Capability>,
    read_origins: BTreeSet<Origin>,
    write_origins: BTreeSet<Origin>,
    robots_decision: RobotsDecision,
    approval: ApprovalEvidence,
}

impl PolicyContext {
    /// Create one policy context from explicitly granted capabilities and origins.
    #[must_use]
    pub const fn new(
        mode: SessionMode,
        purpose: ExecutionPurpose,
        capabilities: BTreeSet<Capability>,
        read_origins: BTreeSet<Origin>,
        write_origins: BTreeSet<Origin>,
        robots_decision: RobotsDecision,
        approval: ApprovalEvidence,
    ) -> Self {
        Self {
            mode,
            purpose,
            capabilities,
            read_origins,
            write_origins,
            robots_decision,
            approval,
        }
    }

    /// Return the browser execution mode.
    #[must_use]
    pub const fn mode(&self) -> SessionMode {
        self.mode
    }

    /// Return the declared execution purpose.
    #[must_use]
    pub const fn purpose(&self) -> ExecutionPurpose {
        self.purpose
    }

    /// Return the granted capabilities.
    #[must_use]
    pub const fn capabilities(&self) -> &BTreeSet<Capability> {
        &self.capabilities
    }

    /// Return the origins that may be read.
    #[must_use]
    pub const fn read_origins(&self) -> &BTreeSet<Origin> {
        &self.read_origins
    }

    /// Return the origins that may be mutated.
    #[must_use]
    pub const fn write_origins(&self) -> &BTreeSet<Origin> {
        &self.write_origins
    }

    /// Return the robots-exclusion decision.
    #[must_use]
    pub const fn robots_decision(&self) -> RobotsDecision {
        self.robots_decision
    }

    /// Replace robots evidence after a fresh policy lookup.
    pub const fn set_robots_decision(&mut self, decision: RobotsDecision) {
        self.robots_decision = decision;
    }

    /// Return the supplied approval evidence.
    #[must_use]
    pub const fn approval(&self) -> &ApprovalEvidence {
        &self.approval
    }

    /// Replace approval evidence after a user or enterprise decision.
    pub fn set_approval(&mut self, approval: ApprovalEvidence) {
        self.approval = approval;
    }
}

/// A canonical Chromium extension identifier admitted to OriginWeave policy.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtensionId {
    canonical: String,
}

impl ExtensionId {
    /// Parse one canonical 32-character lowercase Chromium extension identifier.
    ///
    /// Chromium extension identifiers use only the lowercase `a` through `p`
    /// alphabet. OriginWeave rejects any non-canonical spelling rather than
    /// normalizing caller-controlled identity text.
    pub fn parse(input: &str) -> Result<Self, ExtensionIdError> {
        if input.len() != 32 {
            return Err(ExtensionIdError::InvalidExtensionId);
        }
        if !input.bytes().all(|byte| (b'a'..=b'p').contains(&byte)) {
            return Err(ExtensionIdError::InvalidExtensionId);
        }
        Ok(Self {
            canonical: input.to_owned(),
        })
    }

    /// Return the canonical extension identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
}

/// A validation error for a Chromium extension identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionIdError {
    /// The value was not exactly 32 lowercase characters from `a` through `p`.
    InvalidExtensionId,
}

/// An OriginWeave Agent capability that a browser extension may request explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtensionAgentCapability {
    /// Observe the governed semantic representation of the exact current context.
    ObserveCurrentContext,
    /// Propose a typed action for independent OriginWeave policy evaluation.
    ProposeTypedAction,
}

/// An explicit host-originated grant from one extension to bounded Agent capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionAgentGrant {
    extension_id: ExtensionId,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    origin: Origin,
    expires_at_epoch_seconds: u64,
    capabilities: BTreeSet<ExtensionAgentCapability>,
}

impl ExtensionAgentGrant {
    /// Build an exact extension-to-Agent grant for one session, context, origin, and exclusive expiry.
    #[must_use]
    pub fn new<I>(
        extension_id: ExtensionId,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: Origin,
        expires_at_epoch_seconds: u64,
        capabilities: I,
    ) -> Self
    where
        I: IntoIterator<Item = ExtensionAgentCapability>,
    {
        Self {
            extension_id,
            browser_session,
            browsing_context,
            origin,
            expires_at_epoch_seconds,
            capabilities: capabilities.into_iter().collect(),
        }
    }
}

/// One extension request to use a bounded OriginWeave Agent capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionAccessRequest {
    extension_id: ExtensionId,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    origin: Origin,
    now_epoch_seconds: u64,
    capability: ExtensionAgentCapability,
}

impl ExtensionAccessRequest {
    /// Build one exact extension capability request without granting authority.
    ///
    /// `now_epoch_seconds` must be trusted evaluation time supplied by the host,
    /// not a page, extension, or model clock.
    #[must_use]
    pub const fn new(
        extension_id: ExtensionId,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: Origin,
        now_epoch_seconds: u64,
        capability: ExtensionAgentCapability,
    ) -> Self {
        Self {
            extension_id,
            browser_session,
            browsing_context,
            origin,
            now_epoch_seconds,
            capability,
        }
    }
}

/// Result of evaluating an extension request against one explicit Agent grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionAccessDecision {
    /// The exact extension, session, context, origin, unexpired grant, and capability are explicitly granted.
    Allow,
    /// No explicit extension-to-Agent grant was supplied.
    DenyMissingGrant,
    /// The request belongs to a different extension identity.
    DenyExtensionMismatch,
    /// The request belongs to a different browser automation session.
    DenyBrowserSessionMismatch,
    /// The request belongs to a different independently navigable browser context.
    DenyBrowsingContextMismatch,
    /// The request belongs to a different canonical origin than the grant.
    DenyOriginMismatch,
    /// Trusted evaluation time is at or after the grant's exclusive expiry.
    DenyExpired,
    /// The extension grant does not contain the requested OriginWeave capability.
    DenyCapabilityNotGranted,
}

/// Evaluate extension Agent access without inheriting ambient Chrome permissions.
///
/// A Chrome extension permission, installation state, or page capability is never
/// consulted here. A future Chromium adapter must construct a host-originated
/// [`ExtensionAgentGrant`] explicitly and re-evaluate the exact session, context,
/// canonical origin, and exclusive expiry at the boundary where Agent authority
/// would otherwise cross.
#[must_use]
pub fn evaluate_extension_access(
    request: &ExtensionAccessRequest,
    grant: Option<&ExtensionAgentGrant>,
) -> ExtensionAccessDecision {
    let Some(grant) = grant else {
        return ExtensionAccessDecision::DenyMissingGrant;
    };
    if request.extension_id != grant.extension_id {
        return ExtensionAccessDecision::DenyExtensionMismatch;
    }
    if request.browser_session != grant.browser_session {
        return ExtensionAccessDecision::DenyBrowserSessionMismatch;
    }
    if request.browsing_context != grant.browsing_context {
        return ExtensionAccessDecision::DenyBrowsingContextMismatch;
    }
    if request.origin != grant.origin {
        return ExtensionAccessDecision::DenyOriginMismatch;
    }
    if request.now_epoch_seconds >= grant.expires_at_epoch_seconds {
        return ExtensionAccessDecision::DenyExpired;
    }
    if !grant.capabilities.contains(&request.capability) {
        return ExtensionAccessDecision::DenyCapabilityNotGranted;
    }
    ExtensionAccessDecision::Allow
}
