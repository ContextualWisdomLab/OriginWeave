use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use crate::contracts::ObservedNodeHandle as NodeTuple;
use crate::{BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeHandleError, Origin};

/// Maximum UTF-8 byte length of an opaque browser-protocol identifier retained by the registry.
pub const MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES: usize = 512;

/// Default maximum number of authority identifiers allocated per registry namespace.
const DEFAULT_MAX_BROWSER_AUTHORITY_IDENTIFIERS: u64 = 1_000_000;

/// A node observation that can carry registry-local issuance authority.
///
/// [`ObservedNodeHandle::new`] creates a structurally valid but unregistered observation. Such a
/// value is useful for parsing and fail-closed validation but cannot become live browser authority
/// merely by reproducing session, context, origin, epoch, and node identifiers. Handles returned
/// by [`BrowserAuthorityRegistry::bind_node`] additionally carry an unforgeable in-process
/// registry-instance token. That token is never serialized or exposed through the public API.
#[derive(Debug, Clone)]
pub struct ObservedNodeHandle {
    observed: NodeTuple,
    registry_authority: Option<Arc<()>>,
}

impl ObservedNodeHandle {
    /// Create one structurally valid, unregistered observed node handle.
    ///
    /// Directly constructed handles deliberately carry no registry issuance authority and are
    /// rejected by [`BrowserAuthorityRegistry::validate_node_handle`].
    pub fn new(
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: Origin,
        document_epoch: DocumentEpoch,
        node_id: u64,
    ) -> Result<Self, NodeHandleError> {
        NodeTuple::new(
            browser_session,
            browsing_context,
            origin,
            document_epoch,
            node_id,
        )
        .map(|observed| Self {
            observed,
            registry_authority: None,
        })
    }

    fn registered(
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: Origin,
        document_epoch: DocumentEpoch,
        node_id: u64,
        registry_authority: Arc<()>,
    ) -> Result<Self, NodeHandleError> {
        NodeTuple::new(
            browser_session,
            browsing_context,
            origin,
            document_epoch,
            node_id,
        )
        .map(|observed| Self {
            observed,
            registry_authority: Some(registry_authority),
        })
    }

    /// Return the browser session that produced the node observation.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.observed.browser_session()
    }

    /// Return the browsing context that produced the node observation.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.observed.browsing_context()
    }

    /// Return the canonical origin that produced the node observation.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        self.observed.origin()
    }

    /// Return the document epoch that produced the node observation.
    #[must_use]
    pub const fn document_epoch(&self) -> DocumentEpoch {
        self.observed.document_epoch()
    }

    /// Return the registry-local nonzero node identifier.
    #[must_use]
    pub const fn node_id(&self) -> u64 {
        self.observed.node_id()
    }

    /// Reject use when the session, browsing context, origin, or document epoch has changed.
    pub fn validate_current(
        &self,
        current_session: BrowserSessionId,
        current_context: BrowsingContextId,
        current_origin: &Origin,
        current_epoch: DocumentEpoch,
    ) -> Result<(), NodeHandleError> {
        self.observed.validate_current(
            current_session,
            current_context,
            current_origin,
            current_epoch,
        )
    }

    fn belongs_to(&self, registry_authority: &Arc<()>) -> bool {
        self.registry_authority
            .as_ref()
            .is_some_and(|authority| Arc::ptr_eq(authority, registry_authority))
    }
}

impl PartialEq for ObservedNodeHandle {
    fn eq(&self, other: &Self) -> bool {
        if self.observed != other.observed {
            return false;
        }
        match (&self.registry_authority, &other.registry_authority) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }
}

impl Eq for ObservedNodeHandle {}

/// A bounded in-memory mapping from untrusted adapter identifiers to OriginWeave authority values.
///
/// External WebDriver BiDi, CDP, renderer, frame, and DOM identifiers are retained only as
/// private lookup keys. Callers receive OriginWeave-owned numeric identities whose meaning is
/// scoped to this registry instance. Node identities are additionally scoped to one browsing
/// context, document epoch, canonical origin, and registry-instance issuance token.
pub struct BrowserAuthorityRegistry {
    session_by_external: BTreeMap<String, BrowserSessionId>,
    known_sessions: BTreeSet<BrowserSessionId>,
    context_by_external: BTreeMap<(BrowserSessionId, String), BrowsingContextId>,
    context_session: BTreeMap<BrowsingContextId, BrowserSessionId>,
    context_epoch: BTreeMap<BrowsingContextId, DocumentEpoch>,
    context_origin: BTreeMap<BrowsingContextId, Origin>,
    node_by_external: BTreeMap<(BrowsingContextId, DocumentEpoch, String), u64>,
    node_binding_by_id: BTreeMap<u64, (BrowsingContextId, DocumentEpoch)>,
    registry_authority: Arc<()>,
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
            node_binding_by_id: BTreeMap::new(),
            registry_authority: Arc::new(()),
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
        self.node_binding_by_id
            .retain(|_node_id, (context, _epoch)| *context != browsing_context);
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
        self.node_binding_by_id
            .retain(|_node_id, (context, _epoch)| live_contexts.contains_key(context));
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
            self.node_binding_by_id
                .retain(|_node_id, (context, _epoch)| *context != browsing_context);
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
        let origin_is_unbound = match self.context_origin.get(&browsing_context) {
            Some(expected_origin) if expected_origin != origin => {
                return Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance);
            }
            Some(_expected_origin) => false,
            None => true,
        };
        let epoch = self.current_epoch(browsing_context)?;
        let key = (browsing_context, epoch, external_identifier.to_owned());
        let existing = self.node_by_external.get(&key).copied();
        let node_id = match existing {
            Some(node_id) => node_id,
            None => take_identifier(&mut self.next_node_id, self.maximum_identifier)?,
        };
        if let Some(binding) = self.node_binding_by_id.get(&node_id) {
            if *binding != (browsing_context, epoch) {
                return Err(BrowserRegistryError::InternalAuthorityInvariant);
            }
        } else if existing.is_some() {
            return Err(BrowserRegistryError::InternalAuthorityInvariant);
        }

        let handle = registered_node_handle(
            browser_session,
            browsing_context,
            origin,
            epoch,
            node_id,
            Arc::clone(&self.registry_authority),
        )?;
        if origin_is_unbound {
            self.context_origin.insert(browsing_context, origin.clone());
        }
        if existing.is_none() {
            self.node_by_external.insert(key, node_id);
            self.node_binding_by_id
                .insert(node_id, (browsing_context, epoch));
        }
        Ok(handle)
    }

    /// Retire one exact live node handle without advancing the document epoch.
    ///
    /// This revokes only registry-local node authority. It is intended for relevant same-document
    /// mutations that invalidate one actionable node while leaving the surrounding browsing
    /// context and document epoch current. The node identifier is globally unique inside one
    /// registry, so retirement purges every external alias that refers to that identifier; this
    /// also fails safe if private lookup state was duplicated or corrupted. Retirement does not
    /// claim that Chromium destroyed the underlying DOM/backend node, and the monotonic node
    /// identifier is never reused.
    pub fn remove_node(&mut self, handle: &ObservedNodeHandle) -> Result<(), BrowserRegistryError> {
        self.validate_node_handle(handle)?;
        let node_id = handle.node_id();
        self.node_binding_by_id.remove(&node_id);
        self.node_by_external
            .retain(|_key, bound_node_id| *bound_node_id != node_id);
        Ok(())
    }

    /// Verify that an observed node handle is still live authority in this registry.
    ///
    /// This check must run immediately before a node-local browser action. It re-derives the
    /// current session, context, origin, and document epoch from registry-owned state, requires the
    /// handle to have been issued by this exact registry instance, and resolves the node through a
    /// reverse index rather than scanning every live binding. Caller-constructed, cross-registry,
    /// or retired handles therefore cannot manufacture authority from a self-consistent tuple.
    pub fn validate_node_handle(
        &self,
        handle: &ObservedNodeHandle,
    ) -> Result<(), BrowserRegistryError> {
        if !handle.belongs_to(&self.registry_authority) {
            return Err(BrowserRegistryError::UnknownNodeAuthority);
        }
        if !self.known_sessions.contains(&handle.browser_session()) {
            return Err(BrowserRegistryError::UnknownBrowserSession);
        }
        let context = handle.browsing_context();
        let expected_session = self
            .context_session
            .get(&context)
            .copied()
            .ok_or(BrowserRegistryError::UnknownBrowsingContext)?;
        if expected_session != handle.browser_session() {
            return Err(BrowserRegistryError::UnknownNodeAuthority);
        }
        let epoch = self.current_epoch(context)?;
        let origin = self
            .context_origin
            .get(&context)
            .ok_or(BrowserRegistryError::UnknownNodeAuthority)?;
        handle
            .validate_current(expected_session, context, origin, epoch)
            .map_err(|_error| BrowserRegistryError::UnknownNodeAuthority)?;
        if self.node_binding_by_id.get(&handle.node_id()) != Some(&(context, epoch)) {
            return Err(BrowserRegistryError::UnknownNodeAuthority);
        }
        Ok(())
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
    /// The observed node handle is not a current node binding owned by this registry.
    UnknownNodeAuthority,
    /// The registry exhausted one of its monotonic internal identifier spaces.
    IdentifierSpaceExhausted,
    /// A document epoch reached the maximum representable value.
    DocumentEpochExhausted,
    /// A private registry consistency invariant was violated.
    InternalAuthorityInvariant,
}

impl fmt::Display for BrowserRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExternalIdentifier => write!(
                formatter,
                "external browser identifier must contain 1 to {MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES} UTF-8 bytes"
            ),
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
            Self::UnknownNodeAuthority => formatter
                .write_str("observed node handle is not registered as current browser authority"),
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

fn registered_node_handle(
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    origin: &Origin,
    document_epoch: DocumentEpoch,
    node_id: u64,
    registry_authority: Arc<()>,
) -> Result<ObservedNodeHandle, BrowserRegistryError> {
    ObservedNodeHandle::registered(
        browser_session,
        browsing_context,
        origin.clone(),
        document_epoch,
        node_id,
        registry_authority,
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
    fn unregistered_handle_equality_covers_all_authority_states() {
        let sessions = values(BrowserSessionId::new(1));
        let contexts = values(BrowsingContextId::new(1));
        let epochs = values(DocumentEpoch::new(1));
        let origins = values(Origin::parse("http://127.0.0.1:43127"));
        assert_eq!(sessions.len(), 1);
        assert_eq!(contexts.len(), 1);
        assert_eq!(epochs.len(), 1);
        assert_eq!(origins.len(), 1);
        let session = sessions[0];
        let context = contexts[0];
        let epoch = epochs[0];
        let origin = origins[0].clone();

        let first = values(ObservedNodeHandle::new(
            session,
            context,
            origin.clone(),
            epoch,
            1,
        ));
        let same = values(ObservedNodeHandle::new(
            session,
            context,
            origin.clone(),
            epoch,
            1,
        ));
        let different = values(ObservedNodeHandle::new(
            session,
            context,
            origin.clone(),
            epoch,
            2,
        ));
        assert_eq!(first.len(), 1);
        assert_eq!(same.len(), 1);
        assert_eq!(different.len(), 1);
        assert_eq!(first[0], same[0]);
        assert_ne!(first[0], different[0]);

        let registered = values(ObservedNodeHandle::registered(
            session,
            context,
            origin,
            epoch,
            1,
            Arc::new(()),
        ));
        assert_eq!(registered.len(), 1);
        assert_ne!(first[0], registered[0]);
    }

    #[test]
    fn helper_invariants_and_reverse_index_corruption_fail_closed() {
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
        assert!(
            registered_node_handle(
                sessions[0],
                contexts[0],
                &origins[0],
                epochs[0],
                0,
                Arc::new(())
            )
            .is_err()
        );

        let mut registry = BrowserAuthorityRegistry::new();
        let registered_sessions = values(registry.register_session("corrupt-session"));
        assert_eq!(registered_sessions.len(), 1);
        let session = registered_sessions[0];
        let registered_contexts = values(registry.register_context(session, "corrupt-context"));
        assert_eq!(registered_contexts.len(), 1);
        let context = registered_contexts[0];
        let origin = &origins[0];
        let handles = values(registry.bind_node(session, context, origin, "node"));
        assert_eq!(handles.len(), 1);
        let handle = &handles[0];
        registry.node_binding_by_id.remove(&handle.node_id());
        assert_eq!(
            registry.bind_node(session, context, origin, "node"),
            Err(BrowserRegistryError::InternalAuthorityInvariant)
        );

        registry
            .node_binding_by_id
            .insert(handle.node_id(), (context, epochs[0]));
        registry.node_by_external.clear();
        let other_contexts = values(registry.register_context(session, "other-context"));
        assert_eq!(other_contexts.len(), 1);
        let other_context = other_contexts[0];
        registry.node_by_external.insert(
            (other_context, epochs[0], "other-node".to_owned()),
            handle.node_id(),
        );
        assert_eq!(
            registry.bind_node(session, other_context, origin, "other-node"),
            Err(BrowserRegistryError::InternalAuthorityInvariant)
        );

        let zero_epochs = values(registry.current_epoch(context));
        assert_eq!(zero_epochs.len(), 1);
        let zero_epoch = zero_epochs[0];
        registry
            .node_by_external
            .insert((context, zero_epoch, "zero-node".to_owned()), 0);
        registry.node_binding_by_id.insert(0, (context, zero_epoch));
        assert_eq!(
            registry.bind_node(session, context, origin, "zero-node"),
            Err(BrowserRegistryError::InternalAuthorityInvariant)
        );
    }

    #[test]
    fn validation_reverse_index_rejects_missing_binding() {
        let mut registry = BrowserAuthorityRegistry::new();
        let sessions = values(registry.register_session("session"));
        assert_eq!(sessions.len(), 1);
        let session = sessions[0];
        let contexts = values(registry.register_context(session, "context"));
        assert_eq!(contexts.len(), 1);
        let context = contexts[0];
        let origins = values(Origin::parse("http://127.0.0.1:43127"));
        assert_eq!(origins.len(), 1);
        let handles = values(registry.bind_node(session, context, &origins[0], "node"));
        assert_eq!(handles.len(), 1);
        let handle = &handles[0];
        registry.node_binding_by_id.remove(&handle.node_id());
        assert_eq!(
            registry.validate_node_handle(handle),
            Err(BrowserRegistryError::UnknownNodeAuthority)
        );
    }

    #[test]
    fn issued_handle_rejects_private_context_session_corruption() {
        let mut registry = BrowserAuthorityRegistry::new();
        let owners = values(registry.register_session("corrupt-owner-session"));
        let attackers = values(registry.register_session("corrupt-attacker-session"));
        assert_eq!(owners.len(), 1);
        assert_eq!(attackers.len(), 1);
        let owner = owners[0];
        let attacker = attackers[0];

        let contexts = values(registry.register_context(owner, "corrupt-context-session"));
        assert_eq!(contexts.len(), 1);
        let context = contexts[0];
        let origins = values(Origin::parse("http://127.0.0.1:43127"));
        assert_eq!(origins.len(), 1);
        let handles = values(registry.bind_node(
            owner,
            context,
            &origins[0],
            "corrupt-context-node",
        ));
        assert_eq!(handles.len(), 1);

        registry.context_session.insert(context, attacker);
        assert_eq!(
            registry.validate_node_handle(&handles[0]),
            Err(BrowserRegistryError::UnknownNodeAuthority)
        );
    }

    #[test]
    fn unit_cfg_error_propagation_covers_private_fail_closed_boundaries() {
        let mut registry = BrowserAuthorityRegistry::new();
        let sessions = values(registry.register_session("boundary-session"));
        assert_eq!(sessions.len(), 1);
        let session = sessions[0];
        assert_eq!(
            registry.register_context(session, ""),
            Err(BrowserRegistryError::InvalidExternalIdentifier)
        );

        let contexts = values(registry.register_context(session, "boundary-context"));
        assert_eq!(contexts.len(), 1);
        let context = contexts[0];
        let origins = values(Origin::parse("http://127.0.0.1:43127"));
        assert_eq!(origins.len(), 1);
        let origin = &origins[0];
        assert_eq!(
            registry.bind_node(session, context, origin, ""),
            Err(BrowserRegistryError::InvalidExternalIdentifier)
        );

        let epochs = values(registry.current_epoch(context));
        assert_eq!(epochs.len(), 1);
        let epoch = epochs[0];
        registry.context_epoch.remove(&context);
        assert_eq!(
            registry.bind_node(session, context, origin, "missing-epoch-node"),
            Err(BrowserRegistryError::UnknownBrowsingContext)
        );

        registry.context_epoch.insert(context, epoch);
        let handles = values(registry.bind_node(session, context, origin, "live-node"));
        assert_eq!(handles.len(), 1);
        registry.context_epoch.remove(&context);
        assert_eq!(
            registry.validate_node_handle(&handles[0]),
            Err(BrowserRegistryError::UnknownBrowsingContext)
        );
    }

    #[test]
    fn node_retirement_purges_duplicate_private_aliases_fail_closed() {
        let mut registry = BrowserAuthorityRegistry::new();
        let sessions = values(registry.register_session("retirement-session"));
        assert_eq!(sessions.len(), 1);
        let session = sessions[0];
        let contexts = values(registry.register_context(session, "retirement-context"));
        let other_contexts = values(registry.register_context(session, "other-retirement-context"));
        assert_eq!(contexts.len(), 1);
        assert_eq!(other_contexts.len(), 1);
        let context = contexts[0];
        let other_context = other_contexts[0];
        let origins = values(Origin::parse("http://127.0.0.1:43127"));
        assert_eq!(origins.len(), 1);
        let origin = &origins[0];
        let targets = values(registry.bind_node(session, context, origin, "target-node"));
        let siblings = values(registry.bind_node(session, context, origin, "sibling-node"));
        let others = values(registry.bind_node(session, other_context, origin, "other-node"));
        assert_eq!(targets.len(), 1);
        assert_eq!(siblings.len(), 1);
        assert_eq!(others.len(), 1);
        let target = &targets[0];
        let sibling = &siblings[0];
        let other = &others[0];

        let epochs = values(DocumentEpoch::new(target.document_epoch().value() + 1));
        assert_eq!(epochs.len(), 1);
        let future_key = (context, epochs[0], "corrupt-future-alias".to_owned());
        let cross_context_key = (
            other_context,
            target.document_epoch(),
            "corrupt-cross-context-alias".to_owned(),
        );
        registry
            .node_by_external
            .insert(future_key.clone(), target.node_id());
        registry
            .node_by_external
            .insert(cross_context_key.clone(), target.node_id());

        assert_eq!(registry.remove_node(target), Ok(()));

        assert_eq!(registry.validate_node_handle(sibling), Ok(()));
        assert_eq!(registry.validate_node_handle(other), Ok(()));
        assert_eq!(registry.node_by_external.get(&future_key), None);
        assert_eq!(registry.node_by_external.get(&cross_context_key), None);
        assert_eq!(registry.node_binding_by_id.get(&target.node_id()), None);
    }

    #[test]
    fn document_epoch_exhaustion_is_fail_closed() {
        let mut registry = BrowserAuthorityRegistry::new();
        let sessions = values(registry.register_session("epoch-session"));
        assert_eq!(sessions.len(), 1);
        let session = sessions[0];
        let contexts = values(registry.register_context(session, "epoch-context"));
        assert_eq!(contexts.len(), 1);
        let context = contexts[0];
        let maximum_epochs = values(DocumentEpoch::new(u64::MAX));
        assert_eq!(maximum_epochs.len(), 1);
        registry.context_epoch.insert(context, maximum_epochs[0]);

        assert_eq!(
            registry.advance_document(context),
            Err(BrowserRegistryError::DocumentEpochExhausted)
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
    fn maximum_identifier_limit_is_clamped_without_wrapping() {
        let registry = BrowserAuthorityRegistry::with_identifier_limit(u64::MAX);
        assert_eq!(registry.maximum_identifier, u64::MAX - 1);
    }
}
