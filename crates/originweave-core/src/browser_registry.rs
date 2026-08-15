use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{BrowserSessionId, BrowsingContextId, DocumentEpoch, ObservedNodeHandle, Origin};

/// Maximum UTF-8 byte length of an opaque browser-protocol identifier retained by the registry.
pub const MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES: usize = 512;

/// Default maximum number of authority identifiers allocated per registry namespace.
const DEFAULT_MAX_BROWSER_AUTHORITY_IDENTIFIERS: u64 = 1_000_000;

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
    maximum_identifier: u64,
    next_session_id: u64,
    next_context_id: u64,
    next_node_id: u64,
}

impl BrowserAuthorityRegistry {
    /// Create an empty registry with the reviewed default per-namespace identifier capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_identifier_limit(DEFAULT_MAX_BROWSER_AUTHORITY_IDENTIFIERS)
    }

    /// Create an empty registry with a caller-selected per-namespace identifier capacity.
    ///
    /// Session, browsing-context, and node identifiers each have an independent monotonic
    /// namespace capped at `maximum_identifier`. A zero limit intentionally rejects every new
    /// allocation. Values above `u64::MAX - 1` are clamped so incrementing the next identifier
    /// never wraps to zero.
    #[must_use]
    pub fn with_identifier_limit(maximum_identifier: u64) -> Self {
        let maximum_identifier = maximum_identifier.min(u64::MAX - 1);
        Self {
            session_by_external: BTreeMap::new(),
            known_sessions: BTreeSet::new(),
            context_by_external: BTreeMap::new(),
            context_session: BTreeMap::new(),
            context_epoch: BTreeMap::new(),
            context_origin: BTreeMap::new(),
            node_by_external: BTreeMap::new(),
            maximum_identifier,
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
        let identifier = take_identifier(&mut self.next_session_id, self.maximum_identifier)?;
        browser_session_id(identifier).inspect(|&session| {
            self.session_by_external
                .insert(external_identifier.to_owned(), session);
            self.known_sessions.insert(session);
        })
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
        let identifier = take_identifier(&mut self.next_context_id, self.maximum_identifier)?;
        browsing_context_id(identifier).and_then(|context| {
            document_epoch(1).map(|initial_epoch| {
                self.context_by_external.insert(key, context);
                self.context_session.insert(context, browser_session);
                self.context_epoch.insert(context, initial_epoch);
                context
            })
        })
    }

    /// Retire one browsing context and all registry-local authority derived from it.
    ///
    /// Retirement removes external lookup state, the current document epoch and origin, and every
    /// node binding owned by the context. Monotonic context and node identifiers are never reused.
    /// This revokes only OriginWeave registry-local authority; it does not prove that an external
    /// browser context or process has terminated.
    pub fn remove_context(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<(), BrowserRegistryError> {
        if self.context_session.remove(&browsing_context).is_none() {
            return Err(BrowserRegistryError::UnknownBrowsingContext);
        }
        self.context_by_external
            .retain(|_key, context| *context != browsing_context);
        self.context_epoch.remove(&browsing_context);
        self.context_origin.remove(&browsing_context);
        self.node_by_external
            .retain(|(context, _epoch, _external), _node_id| *context != browsing_context);
        Ok(())
    }

    /// Retire one browser session and every registered context and node binding beneath it.
    ///
    /// Retirement removes only registry-local authority and external lookup state. Session,
    /// context, and node identifiers remain strictly monotonic so a later registration of the same
    /// opaque browser identifier cannot revive stale authority. External process termination is a
    /// separate adapter responsibility.
    pub fn remove_session(
        &mut self,
        browser_session: BrowserSessionId,
    ) -> Result<(), BrowserRegistryError> {
        if !self.known_sessions.remove(&browser_session) {
            return Err(BrowserRegistryError::UnknownBrowserSession);
        }
        self.session_by_external
            .retain(|_external, session| *session != browser_session);
        self.context_by_external
            .retain(|(session, _external), _context| *session != browser_session);
        self.context_session
            .retain(|_context, session| *session != browser_session);

        let live_contexts = &self.context_session;
        self.context_epoch
            .retain(|context, _epoch| live_contexts.contains_key(context));
        self.context_origin
            .retain(|context, _origin| live_contexts.contains_key(context));
        self.node_by_external
            .retain(|(context, _epoch, _external), _node_id| live_contexts.contains_key(context));
        Ok(())
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
        document_epoch(next_value).inspect(|&next| {
            self.context_epoch.insert(browsing_context, next);
            self.context_origin.remove(&browsing_context);
            self.node_by_external
                .retain(|(context, _epoch, _external), _node_id| *context != browsing_context);
        })
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
        self.current_epoch(browsing_context).and_then(|epoch| {
            let key = (browsing_context, epoch, external_identifier.to_owned());
            let node_id = if let Some(existing) = self.node_by_external.get(&key) {
                *existing
            } else {
                let allocated = take_identifier(&mut self.next_node_id, self.maximum_identifier)?;
                self.node_by_external.insert(key, allocated);
                allocated
            };
            observed_node_handle(browser_session, browsing_context, origin, epoch, node_id)
        })
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
            Self::InvalidExternalIdentifier => {
                formatter.write_str("external browser identifier must contain 1 to 512 UTF-8 bytes")
            }
            Self::UnknownBrowserSession => {
                formatter.write_str("browser session is not registered in this authority registry")
            }
            Self::UnknownBrowsingContext => {
                formatter.write_str("browsing context is not registered in this authority registry")
            }
            Self::ContextSessionMismatch { expected, actual } => write!(
                formatter,
                "browsing context belongs to session {}, not session {}",
                expected.value(),
                actual.value()
            ),
            Self::OriginChangedWithoutDocumentAdvance => formatter
                .write_str("browsing context origin changed without advancing the document epoch"),
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

fn take_identifier(next: &mut u64, maximum_identifier: u64) -> Result<u64, BrowserRegistryError> {
    if *next > maximum_identifier {
        return Err(BrowserRegistryError::IdentifierSpaceExhausted);
    }
    let identifier = *next;
    *next = identifier + 1;
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

    fn values<T, E>(result: Result<T, E>) -> Vec<T> {
        result.into_iter().collect()
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

        let sessions = values(BrowserSessionId::new(1));
        let contexts = values(BrowsingContextId::new(1));
        let epochs = values(DocumentEpoch::new(1));
        let origins = values(Origin::parse("http://127.0.0.1:43127"));
        assert_eq!(sessions.len(), 1);
        assert_eq!(contexts.len(), 1);
        assert_eq!(epochs.len(), 1);
        assert_eq!(origins.len(), 1);
        assert_eq!(
            observed_node_handle(sessions[0], contexts[0], &origins[0], epochs[0], 0),
            Err(BrowserRegistryError::InternalAuthorityInvariant)
        );
    }

    #[test]
    fn monotonic_identifier_exhaustion_is_fail_closed() {
        let mut next = 1;
        assert_eq!(take_identifier(&mut next, 1), Ok(1));
        assert_eq!(next, 2);
        assert_eq!(
            take_identifier(&mut next, 1),
            Err(BrowserRegistryError::IdentifierSpaceExhausted)
        );
    }

    #[test]
    fn registry_reports_all_resource_and_authority_failures() {
        let known_sessions = values(BrowserSessionId::new(1));
        let unknown_contexts = values(BrowsingContextId::new(1));
        let initial_epochs = values(DocumentEpoch::new(1));
        let origins = values(Origin::parse("http://127.0.0.1:43127"));
        assert_eq!(known_sessions.len(), 1);
        assert_eq!(unknown_contexts.len(), 1);
        assert_eq!(initial_epochs.len(), 1);
        assert_eq!(origins.len(), 1);
        let known_session = known_sessions[0];
        let unknown_context = unknown_contexts[0];
        let initial_epoch = initial_epochs[0];
        let origin = &origins[0];

        let mut limited_registry = BrowserAuthorityRegistry::with_identifier_limit(1);
        let limited_sessions = values(limited_registry.register_session("session-one"));
        assert_eq!(limited_sessions.len(), 1);
        let limited_session = limited_sessions[0];
        assert_eq!(
            limited_registry.register_session("session-two"),
            Err(BrowserRegistryError::IdentifierSpaceExhausted)
        );
        let limited_contexts =
            values(limited_registry.register_context(limited_session, "context-one"));
        assert_eq!(limited_contexts.len(), 1);
        let limited_context = limited_contexts[0];
        assert_eq!(
            limited_registry.register_context(limited_session, "context-two"),
            Err(BrowserRegistryError::IdentifierSpaceExhausted)
        );
        assert!(
            limited_registry
                .bind_node(limited_session, limited_context, origin, "node-one")
                .is_ok()
        );
        assert_eq!(
            limited_registry.bind_node(limited_session, limited_context, origin, "node-two"),
            Err(BrowserRegistryError::IdentifierSpaceExhausted)
        );

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
            registry.bind_node(known_session, unknown_context, origin, "node"),
            Err(BrowserRegistryError::UnknownBrowserSession)
        );

        let sessions = values(registry.register_session("session"));
        assert_eq!(sessions.len(), 1);
        let session = sessions[0];
        let contexts = values(registry.register_context(session, "context-a"));
        assert_eq!(contexts.len(), 1);
        let context = contexts[0];

        let maximum_epochs = values(DocumentEpoch::new(u64::MAX));
        assert_eq!(maximum_epochs.len(), 1);
        registry.context_epoch.insert(context, maximum_epochs[0]);
        assert_eq!(
            registry.advance_document(context),
            Err(BrowserRegistryError::DocumentEpochExhausted)
        );
        registry.context_epoch.insert(context, initial_epoch);

        let unknown_sessions = values(BrowserSessionId::new(999));
        let unknown_contexts = values(BrowsingContextId::new(999));
        assert_eq!(unknown_sessions.len(), 1);
        assert_eq!(unknown_contexts.len(), 1);
        assert_eq!(
            registry.bind_node(unknown_sessions[0], context, origin, "node"),
            Err(BrowserRegistryError::UnknownBrowserSession)
        );
        assert_eq!(
            registry.bind_node(session, unknown_contexts[0], origin, "node"),
            Err(BrowserRegistryError::UnknownBrowsingContext)
        );
    }

    #[test]
    fn origin_rotation_and_node_cleanup_are_explicit() {
        let mut registry = BrowserAuthorityRegistry::new();
        let sessions = values(registry.register_session("session"));
        assert_eq!(sessions.len(), 1);
        let session = sessions[0];
        let contexts = values(registry.register_context(session, "context"));
        let second_contexts = values(registry.register_context(session, "context-two"));
        assert_eq!(contexts.len(), 1);
        assert_eq!(second_contexts.len(), 1);
        let context = contexts[0];
        let second_context = second_contexts[0];
        assert_eq!(registry.register_context(session, "context"), Ok(context));

        let first_origins = values(Origin::parse("http://127.0.0.1:43127"));
        let second_origins = values(Origin::parse("http://localhost:43127"));
        assert_eq!(first_origins.len(), 1);
        assert_eq!(second_origins.len(), 1);
        let first_origin = &first_origins[0];
        let second_origin = &second_origins[0];
        assert!(
            registry
                .bind_node(session, context, first_origin, "node-a")
                .is_ok()
        );
        assert!(
            registry
                .bind_node(session, second_context, first_origin, "node-b")
                .is_ok()
        );
        assert_eq!(
            registry.bind_node(session, context, second_origin, "node-a"),
            Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance)
        );
        assert_eq!(registry.node_by_external.len(), 2);
        assert!(registry.advance_document(context).is_ok());
        assert_eq!(registry.node_by_external.len(), 1);
        assert!(
            registry
                .bind_node(session, context, second_origin, "node-a")
                .is_ok()
        );
    }

    #[test]
    fn invalid_node_and_context_inputs_are_rejected() {
        let mut registry = BrowserAuthorityRegistry::new();
        let sessions = values(registry.register_session("session"));
        assert_eq!(sessions.len(), 1);
        let session = sessions[0];
        assert_eq!(
            registry.register_context(session, ""),
            Err(BrowserRegistryError::InvalidExternalIdentifier)
        );
        assert_eq!(
            registry.register_context(
                session,
                &"x".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES + 1),
            ),
            Err(BrowserRegistryError::InvalidExternalIdentifier)
        );
        let contexts = values(registry.register_context(session, "context"));
        let origins = values(Origin::parse("http://127.0.0.1:43127"));
        assert_eq!(contexts.len(), 1);
        assert_eq!(origins.len(), 1);
        assert_eq!(
            registry.bind_node(session, contexts[0], &origins[0], ""),
            Err(BrowserRegistryError::InvalidExternalIdentifier)
        );
        assert_eq!(
            registry.bind_node(
                session,
                contexts[0],
                &origins[0],
                &"x".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES + 1),
            ),
            Err(BrowserRegistryError::InvalidExternalIdentifier)
        );
    }

    #[test]
    fn browser_registry_errors_have_non_sensitive_deterministic_text() {
        let expected_values = values(BrowserSessionId::new(1));
        let actual_values = values(BrowserSessionId::new(2));
        assert_eq!(expected_values.len(), 1);
        assert_eq!(actual_values.len(), 1);
        let errors = [
            BrowserRegistryError::InvalidExternalIdentifier,
            BrowserRegistryError::UnknownBrowserSession,
            BrowserRegistryError::UnknownBrowsingContext,
            BrowserRegistryError::ContextSessionMismatch {
                expected: expected_values[0],
                actual: actual_values[0],
            },
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
