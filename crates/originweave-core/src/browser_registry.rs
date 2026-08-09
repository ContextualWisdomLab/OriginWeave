use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{BrowserSessionId, BrowsingContextId, DocumentEpoch, ObservedNodeHandle, Origin};

/// Maximum UTF-8 byte length of an opaque browser-protocol identifier retained by the registry.
pub const MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES: usize = 512;

/// A bounded in-memory mapping from untrusted adapter identifiers to OriginWeave authority values.
///
/// External WebDriver BiDi, CDP, renderer, frame, and DOM identifiers are retained only as
/// private lookup keys. Callers receive OriginWeave-owned numeric identities whose meaning is
/// scoped to this registry instance. Node identities are additionally scoped to one browsing
/// context, document epoch, and canonical origin.
pub struct BrowserAuthorityRegistry {
    session_by_external: BTreeMap<String, BrowserSessionId>,
    known_sessions: BTreeSet<BrowserSessionId>,
    context_by_external: BTreeMap<(BrowserSessionId, String), BrowsingContextId>,
    context_session: BTreeMap<BrowsingContextId, BrowserSessionId>,
    context_epoch: BTreeMap<BrowsingContextId, DocumentEpoch>,
    context_origin: BTreeMap<BrowsingContextId, Origin>,
    node_by_external: BTreeMap<(BrowsingContextId, DocumentEpoch, String), u64>,
    next_session_id: u64,
    next_context_id: u64,
    next_node_id: u64,
}

impl BrowserAuthorityRegistry {
    /// Create an empty registry whose first internal identities are one.
    #[must_use]
    pub fn new() -> Self {
        Self {
            session_by_external: BTreeMap::new(),
            known_sessions: BTreeSet::new(),
            context_by_external: BTreeMap::new(),
            context_session: BTreeMap::new(),
            context_epoch: BTreeMap::new(),
            context_origin: BTreeMap::new(),
            node_by_external: BTreeMap::new(),
            next_session_id: 1,
            next_context_id: 1,
            next_node_id: 1,
        }
    }

    /// Register one opaque external browser-session identifier.
    ///
    /// Re-registering the same identifier in this registry returns the same OriginWeave session.
    pub fn register_session(
        &mut self,
        external_identifier: &str,
    ) -> Result<BrowserSessionId, BrowserRegistryError> {
        validate_external_identifier(external_identifier)?;
        if let Some(existing) = self.session_by_external.get(external_identifier) {
            return Ok(*existing);
        }
        let identifier = take_identifier(&mut self.next_session_id)?;
        let session = browser_session_id(identifier)?;
        self.session_by_external
            .insert(external_identifier.to_owned(), session);
        self.known_sessions.insert(session);
        Ok(session)
    }

    /// Register one opaque external browsing-context identifier inside a known browser session.
    ///
    /// A newly registered context starts at document epoch one. The same external context text in
    /// another browser session receives a different OriginWeave context identity.
    pub fn register_context(
        &mut self,
        browser_session: BrowserSessionId,
        external_identifier: &str,
    ) -> Result<BrowsingContextId, BrowserRegistryError> {
        validate_external_identifier(external_identifier)?;
        if !self.known_sessions.contains(&browser_session) {
            return Err(BrowserRegistryError::UnknownBrowserSession);
        }
        let key = (browser_session, external_identifier.to_owned());
        if let Some(existing) = self.context_by_external.get(&key) {
            return Ok(*existing);
        }
        let identifier = take_identifier(&mut self.next_context_id)?;
        let context = browsing_context_id(identifier)?;
        self.context_by_external.insert(key, context);
        self.context_session.insert(context, browser_session);
        self.context_epoch.insert(context, document_epoch(1)?);
        Ok(context)
    }

    /// Return the currently active document epoch for a known browsing context.
    pub fn current_epoch(
        &self,
        browsing_context: BrowsingContextId,
    ) -> Result<DocumentEpoch, BrowserRegistryError> {
        self.context_epoch
            .get(&browsing_context)
            .copied()
            .ok_or(BrowserRegistryError::UnknownBrowsingContext)
    }

    /// Advance a browsing context to the next document epoch and invalidate old node bindings.
    ///
    /// Call this whenever navigation or document replacement invalidates actionable node identity.
    pub fn advance_document(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<DocumentEpoch, BrowserRegistryError> {
        let current = self
            .context_epoch
            .get(&browsing_context)
            .copied()
            .ok_or(BrowserRegistryError::UnknownBrowsingContext)?;
        let next_value = current
            .value()
            .checked_add(1)
            .ok_or(BrowserRegistryError::DocumentEpochExhausted)?;
        let next = document_epoch(next_value)?;
        self.context_epoch.insert(browsing_context, next);
        self.context_origin.remove(&browsing_context);
        self.node_by_external
            .retain(|(context, _epoch, _external), _node_id| *context != browsing_context);
        Ok(next)
    }

    /// Bind one opaque adapter-local node identifier to the exact current browser authority.
    ///
    /// Rebinding the same adapter node inside the same document returns a stable OriginWeave node
    /// identifier. A document advance discards that mapping, so adapter node-number reuse cannot
    /// revive stale authority. An origin change without a document advance fails closed.
    pub fn bind_node(
        &mut self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: &Origin,
        external_identifier: &str,
    ) -> Result<ObservedNodeHandle, BrowserRegistryError> {
        validate_external_identifier(external_identifier)?;
        if !self.known_sessions.contains(&browser_session) {
            return Err(BrowserRegistryError::UnknownBrowserSession);
        }
        let expected_session = self
            .context_session
            .get(&browsing_context)
            .copied()
            .ok_or(BrowserRegistryError::UnknownBrowsingContext)?;
        if expected_session != browser_session {
            return Err(BrowserRegistryError::ContextSessionMismatch {
                expected: expected_session,
                actual: browser_session,
            });
        }
        match self.context_origin.get(&browsing_context) {
            Some(expected_origin) if expected_origin != origin => {
                return Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance);
            }
            Some(_expected_origin) => {}
            None => {
                self.context_origin.insert(browsing_context, origin.clone());
            }
        }
        let epoch = self.current_epoch(browsing_context)?;
        let key = (browsing_context, epoch, external_identifier.to_owned());
        let node_id = if let Some(existing) = self.node_by_external.get(&key) {
            *existing
        } else {
            let allocated = take_identifier(&mut self.next_node_id)?;
            self.node_by_external.insert(key, allocated);
            allocated
        };
        observed_node_handle(browser_session, browsing_context, origin, epoch, node_id)
    }
}

impl Default for BrowserAuthorityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A fail-closed error produced while translating external browser identifiers into local authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserRegistryError {
    /// An external identifier was empty or exceeded the reviewed byte bound.
    InvalidExternalIdentifier,
    /// The supplied OriginWeave browser session is not registered in this registry.
    UnknownBrowserSession,
    /// The supplied OriginWeave browsing context is not registered in this registry.
    UnknownBrowsingContext,
    /// The browsing context belongs to another browser session.
    ContextSessionMismatch {
        /// Session that owns the registered context.
        expected: BrowserSessionId,
        /// Session supplied by the current caller.
        actual: BrowserSessionId,
    },
    /// The context origin changed without first rotating the document epoch.
    OriginChangedWithoutDocumentAdvance,
    /// The registry exhausted one of its monotonic internal identifier spaces.
    IdentifierSpaceExhausted,
    /// A document epoch reached the maximum representable value.
    DocumentEpochExhausted,
    /// An internal nonzero authority invariant was violated.
    InternalAuthorityInvariant,
}

impl fmt::Display for BrowserRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExternalIdentifier => formatter.write_str(
                "external browser identifier must contain 1 to 512 UTF-8 bytes",
            ),
            Self::UnknownBrowserSession => {
                formatter.write_str("browser session is not registered in this authority registry")
            }
            Self::UnknownBrowsingContext => formatter
                .write_str("browsing context is not registered in this authority registry"),
            Self::ContextSessionMismatch { expected, actual } => write!(
                formatter,
                "browsing context belongs to session {}, not session {}",
                expected.value(),
                actual.value()
            ),
            Self::OriginChangedWithoutDocumentAdvance => formatter.write_str(
                "browsing context origin changed without advancing the document epoch",
            ),
            Self::IdentifierSpaceExhausted => {
                formatter.write_str("browser authority identifier space is exhausted")
            }
            Self::DocumentEpochExhausted => {
                formatter.write_str("browser document epoch space is exhausted")
            }
            Self::InternalAuthorityInvariant => {
                formatter.write_str("browser authority registry violated a nonzero invariant")
            }
        }
    }
}

impl std::error::Error for BrowserRegistryError {}

fn validate_external_identifier(identifier: &str) -> Result<(), BrowserRegistryError> {
    if identifier.is_empty() || identifier.len() > MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES {
        return Err(BrowserRegistryError::InvalidExternalIdentifier);
    }
    Ok(())
}

fn take_identifier(next: &mut u64) -> Result<u64, BrowserRegistryError> {
    if *next == 0 {
        return Err(BrowserRegistryError::IdentifierSpaceExhausted);
    }
    let identifier = *next;
    *next = identifier.wrapping_add(1);
    Ok(identifier)
}

fn browser_session_id(value: u64) -> Result<BrowserSessionId, BrowserRegistryError> {
    BrowserSessionId::new(value).map_err(|_error| BrowserRegistryError::InternalAuthorityInvariant)
}

fn browsing_context_id(value: u64) -> Result<BrowsingContextId, BrowserRegistryError> {
    BrowsingContextId::new(value).map_err(|_error| BrowserRegistryError::InternalAuthorityInvariant)
}

fn document_epoch(value: u64) -> Result<DocumentEpoch, BrowserRegistryError> {
    DocumentEpoch::new(value).map_err(|_error| BrowserRegistryError::InternalAuthorityInvariant)
}

fn observed_node_handle(
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    origin: &Origin,
    document_epoch: DocumentEpoch,
    node_id: u64,
) -> Result<ObservedNodeHandle, BrowserRegistryError> {
    ObservedNodeHandle::new(
        browser_session,
        browsing_context,
        origin.clone(),
        document_epoch,
        node_id,
    )
    .map_err(|_error| BrowserRegistryError::InternalAuthorityInvariant)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> Option<(BrowserSessionId, BrowsingContextId, DocumentEpoch)> {
        let session = BrowserSessionId::new(1).ok()?;
        let context = BrowsingContextId::new(1).ok()?;
        let epoch = DocumentEpoch::new(1).ok()?;
        Some((session, context, epoch))
    }

    fn loopback_origin() -> Option<Origin> {
        Origin::parse("http://127.0.0.1:43127").ok()
    }

    #[test]
    fn helper_invariants_fail_closed() {
        assert_eq!(
            browser_session_id(0),
            Err(BrowserRegistryError::InternalAuthorityInvariant)
        );
        assert_eq!(
            browsing_context_id(0),
            Err(BrowserRegistryError::InternalAuthorityInvariant)
        );
        assert_eq!(
            document_epoch(0),
            Err(BrowserRegistryError::InternalAuthorityInvariant)
        );
        let Some((session, context, epoch)) = ids() else {
            return;
        };
        let Some(origin) = loopback_origin() else {
            return;
        };
        assert_eq!(
            observed_node_handle(session, context, &origin, epoch, 0),
            Err(BrowserRegistryError::InternalAuthorityInvariant)
        );
    }

    #[test]
    fn monotonic_identifier_exhaustion_is_fail_closed() {
        let mut next = u64::MAX;
        assert_eq!(take_identifier(&mut next), Ok(u64::MAX));
        assert_eq!(next, 0);
        assert_eq!(
            take_identifier(&mut next),
            Err(BrowserRegistryError::IdentifierSpaceExhausted)
        );
    }

    #[test]
    fn registry_reports_all_resource_and_authority_failures() {
        let Some((known_session, unknown_context, initial_epoch)) = ids() else {
            return;
        };
        let Some(origin) = loopback_origin() else {
            return;
        };
        let mut registry = BrowserAuthorityRegistry::default();
        assert_eq!(
            registry.current_epoch(unknown_context),
            Err(BrowserRegistryError::UnknownBrowsingContext)
        );
        assert_eq!(
            registry.advance_document(unknown_context),
            Err(BrowserRegistryError::UnknownBrowsingContext)
        );
        assert_eq!(
            registry.bind_node(known_session, unknown_context, &origin, "node"),
            Err(BrowserRegistryError::UnknownBrowserSession)
        );

        registry.next_session_id = 0;
        assert_eq!(
            registry.register_session("new-session"),
            Err(BrowserRegistryError::IdentifierSpaceExhausted)
        );
        registry.next_session_id = 1;
        let Ok(session) = registry.register_session("session") else {
            return;
        };

        registry.next_context_id = 0;
        assert_eq!(
            registry.register_context(session, "context-a"),
            Err(BrowserRegistryError::IdentifierSpaceExhausted)
        );
        registry.next_context_id = 1;
        let Ok(context) = registry.register_context(session, "context-a") else {
            return;
        };

        let Ok(max_epoch) = DocumentEpoch::new(u64::MAX) else {
            return;
        };
        registry.context_epoch.insert(context, max_epoch);
        assert_eq!(
            registry.advance_document(context),
            Err(BrowserRegistryError::DocumentEpochExhausted)
        );
        registry.context_epoch.insert(context, initial_epoch);

        registry.next_node_id = 0;
        assert_eq!(
            registry.bind_node(session, context, &origin, "node-a"),
            Err(BrowserRegistryError::IdentifierSpaceExhausted)
        );

        let Ok(unknown_known_session) = BrowserSessionId::new(999) else {
            return;
        };
        assert_eq!(
            registry.bind_node(unknown_known_session, context, &origin, "node"),
            Err(BrowserRegistryError::UnknownBrowserSession)
        );
        let Ok(unknown_known_context) = BrowsingContextId::new(999) else {
            return;
        };
        assert_eq!(
            registry.bind_node(session, unknown_known_context, &origin, "node"),
            Err(BrowserRegistryError::UnknownBrowsingContext)
        );
    }

    #[test]
    fn origin_rotation_and_node_cleanup_are_explicit() {
        let mut registry = BrowserAuthorityRegistry::new();
        let Ok(session) = registry.register_session("session") else {
            return;
        };
        let Ok(context) = registry.register_context(session, "context") else {
            return;
        };
        let Ok(second_context) = registry.register_context(session, "context-two") else {
            return;
        };
        assert_eq!(registry.register_context(session, "context"), Ok(context));

        let Some(first_origin) = loopback_origin() else {
            return;
        };
        let Ok(second_origin) = Origin::parse("http://localhost:43127") else {
            return;
        };
        assert!(
            registry
                .bind_node(session, context, &first_origin, "node-a")
                .is_ok()
        );
        assert!(
            registry
                .bind_node(session, second_context, &first_origin, "node-b")
                .is_ok()
        );
        assert_eq!(
            registry.bind_node(session, context, &second_origin, "node-a"),
            Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance)
        );
        assert_eq!(registry.node_by_external.len(), 2);
        assert!(registry.advance_document(context).is_ok());
        assert_eq!(registry.node_by_external.len(), 1);
        assert!(
            registry
                .bind_node(session, context, &second_origin, "node-a")
                .is_ok()
        );
    }

    #[test]
    fn invalid_node_and_context_inputs_are_rejected() {
        let mut registry = BrowserAuthorityRegistry::new();
        let Ok(session) = registry.register_session("session") else {
            return;
        };
        assert_eq!(
            registry.register_context(session, ""),
            Err(BrowserRegistryError::InvalidExternalIdentifier)
        );
        let Ok(context) = registry.register_context(session, "context") else {
            return;
        };
        let Some(origin) = loopback_origin() else {
            return;
        };
        assert_eq!(
            registry.bind_node(session, context, &origin, ""),
            Err(BrowserRegistryError::InvalidExternalIdentifier)
        );
    }

    #[test]
    fn browser_registry_errors_have_non_sensitive_deterministic_text() {
        let Some((expected, _context, _epoch)) = ids() else {
            return;
        };
        let Ok(actual) = BrowserSessionId::new(2) else {
            return;
        };
        let errors = [
            BrowserRegistryError::InvalidExternalIdentifier,
            BrowserRegistryError::UnknownBrowserSession,
            BrowserRegistryError::UnknownBrowsingContext,
            BrowserRegistryError::ContextSessionMismatch { expected, actual },
            BrowserRegistryError::OriginChangedWithoutDocumentAdvance,
            BrowserRegistryError::IdentifierSpaceExhausted,
            BrowserRegistryError::DocumentEpochExhausted,
            BrowserRegistryError::InternalAuthorityInvariant,
        ];
        for error in errors {
            let text = error.to_string();
            assert!(!text.is_empty());
            assert!(!text.contains("webdriver-session"));
        }
    }
}
