//! Extension-to-Agent authority adapted onto the refactored core contracts.
//!
//! The browser-registry branch split long-lived contracts into a private
//! `contracts` module before protected main added origin and exclusive-expiry
//! binding to extension grants. This module preserves those protected-main
//! semantics without allowing raw Chromium permissions or identifiers to become
//! Agent authority.

use std::fmt;

use crate::contracts::{
    BrowserSessionId, BrowsingContextId, ExtensionAccessDecision as BaseExtensionAccessDecision,
    ExtensionAccessRequest as BaseExtensionAccessRequest, ExtensionAgentCapability,
    ExtensionAgentGrant as BaseExtensionAgentGrant, ExtensionId, Origin,
    evaluate_extension_access as evaluate_base_extension_access,
};

/// A nonzero host-assigned identity for one isolated Agent Task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentTaskId(u64);

impl AgentTaskId {
    /// Validate one host-assigned Agent Task identifier.
    pub const fn new(value: u64) -> Result<Self, AgentTaskIdError> {
        if value == 0 {
            return Err(AgentTaskIdError::InvalidAgentTaskId);
        }
        Ok(Self(value))
    }

    /// Return the validated Agent Task identifier.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A validation failure for an Agent Task identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskIdError {
    /// Agent Task identities are one-based and zero was supplied.
    InvalidAgentTaskId,
}

impl fmt::Display for AgentTaskIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAgentTaskId => {
                formatter.write_str("Agent Task identifier must be nonzero")
            }
        }
    }
}

impl std::error::Error for AgentTaskIdError {}

/// An explicit host-originated extension grant bound to task, session, context, origin, and expiry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionAgentGrant {
    base: BaseExtensionAgentGrant,
    agent_task: AgentTaskId,
    origin: Origin,
    expires_at_epoch_seconds: u64,
}

impl ExtensionAgentGrant {
    /// Build an exact extension-to-Agent grant for one task, session, context, origin, and expiry.
    #[must_use]
    pub fn new<I>(
        extension_id: ExtensionId,
        agent_task: AgentTaskId,
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
            base: BaseExtensionAgentGrant::new(
                extension_id,
                browser_session,
                browsing_context,
                capabilities,
            ),
            agent_task,
            origin,
            expires_at_epoch_seconds,
        }
    }
}

/// One task-bound extension request to use a bounded Agent capability at trusted evaluation time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionAccessRequest {
    base: BaseExtensionAccessRequest,
    agent_task: AgentTaskId,
    origin: Origin,
    now_epoch_seconds: u64,
}

impl ExtensionAccessRequest {
    /// Build one exact task-bound extension capability request without granting authority.
    ///
    /// `now_epoch_seconds` must come from trusted host evaluation time rather
    /// than a page, extension, model, or other caller-controlled clock.
    #[must_use]
    pub const fn new(
        extension_id: ExtensionId,
        agent_task: AgentTaskId,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: Origin,
        now_epoch_seconds: u64,
        capability: ExtensionAgentCapability,
    ) -> Self {
        Self {
            base: BaseExtensionAccessRequest::new(
                extension_id,
                browser_session,
                browsing_context,
                capability,
            ),
            agent_task,
            origin,
            now_epoch_seconds,
        }
    }
}

/// Result of evaluating one extension request against one explicit Agent grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionAccessDecision {
    /// Extension, task, session, context, origin, expiry, and capability all match.
    Allow,
    /// No explicit extension-to-Agent grant was supplied.
    DenyMissingGrant,
    /// The request belongs to a different extension identity.
    DenyExtensionMismatch,
    /// The request belongs to a different Agent Task identity.
    DenyAgentTaskMismatch,
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
/// Extension identity, session, context, and missing-grant checks reuse the pre-existing
/// deterministic contract. Exact Agent Task identity, origin, and exclusive-expiry checks are
/// then applied before a capability denial or allowance is returned, preserving fail-closed
/// authority ordering on the refactored branch.
#[must_use]
pub fn evaluate_extension_access(
    request: &ExtensionAccessRequest,
    grant: Option<&ExtensionAgentGrant>,
) -> ExtensionAccessDecision {
    let base_decision =
        evaluate_base_extension_access(&request.base, grant.map(|grant| &grant.base));
    match base_decision {
        BaseExtensionAccessDecision::DenyMissingGrant => ExtensionAccessDecision::DenyMissingGrant,
        BaseExtensionAccessDecision::DenyExtensionMismatch => {
            ExtensionAccessDecision::DenyExtensionMismatch
        }
        BaseExtensionAccessDecision::DenyBrowserSessionMismatch => {
            ExtensionAccessDecision::DenyBrowserSessionMismatch
        }
        BaseExtensionAccessDecision::DenyBrowsingContextMismatch => {
            ExtensionAccessDecision::DenyBrowsingContextMismatch
        }
        BaseExtensionAccessDecision::Allow
        | BaseExtensionAccessDecision::DenyCapabilityNotGranted => {
            grant.map_or(ExtensionAccessDecision::DenyMissingGrant, |grant| {
                if request.agent_task != grant.agent_task {
                    return ExtensionAccessDecision::DenyAgentTaskMismatch;
                }
                if request.origin != grant.origin {
                    return ExtensionAccessDecision::DenyOriginMismatch;
                }
                if request.now_epoch_seconds >= grant.expires_at_epoch_seconds {
                    return ExtensionAccessDecision::DenyExpired;
                }
                if base_decision == BaseExtensionAccessDecision::DenyCapabilityNotGranted {
                    return ExtensionAccessDecision::DenyCapabilityNotGranted;
                }
                ExtensionAccessDecision::Allow
            })
        }
    }
}
