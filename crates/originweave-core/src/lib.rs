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
    /// Parse one origin and reject paths, credentials, fragments, and insecure
    /// remote HTTP endpoints.
    pub fn parse(input: &str) -> Result<Self, OriginError> {
        if input.trim() != input
            || input
                .chars()
                .any(|character| character.is_control() || character.is_whitespace())
        {
            return Err(OriginError::InvalidAuthority);
        }

        let Some((scheme, authority)) = input.split_once("://") else {
            return Err(OriginError::MissingScheme);
        };
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

        let canonical = match port {
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
}

impl fmt::Display for Origin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
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
    validate_dns_host(host_text)?;
    let host = host_text.to_ascii_lowercase();
    let is_loopback = host == "localhost"
        || host
            .parse::<Ipv4Addr>()
            .is_ok_and(|address| address.is_loopback());
    Ok((host, port, is_loopback))
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
    Ok((
        format!("[{address}]"),
        port,
        address.is_loopback(),
    ))
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
    if host.is_empty()
        || host.len() > 253
        || !host.is_ascii()
        || host.starts_with('.')
        || host.ends_with('.')
    {
        return Err(OriginError::InvalidAuthority);
    }
    for label in host.split('.') {
        let valid = !label.is_empty()
            && label.len() <= 63
            && label
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
        if !valid {
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
    /// The explicit port was outside `1..=65535` or was not numeric.
    InvalidPort,
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

/// The exact action and target origin covered by one approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalScope {
    action: ActionKind,
    target_origin: Origin,
}

impl ApprovalScope {
    /// Create one exact approval scope.
    #[must_use]
    pub const fn new(action: ActionKind, target_origin: Origin) -> Self {
        Self {
            action,
            target_origin,
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
}

/// Evidence that a high-risk action was approved for an exact scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalEvidence {
    /// No approval was supplied.
    None,
    /// A person confirmed the exact action and target.
    UserConfirmed(ApprovalScope),
    /// A managed enterprise policy approved the exact action and target.
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
    ) -> Self {
        Self {
            action,
            source_origin,
            target_origin,
            instruction_source,
            secret_delivery,
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
