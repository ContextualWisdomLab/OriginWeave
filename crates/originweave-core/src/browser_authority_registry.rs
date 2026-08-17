use crate::browser_registry::BrowserAuthorityRegistry as RawBrowserAuthorityRegistry;
use crate::{
    BrowserRegistryError, BrowserSessionId, BrowsingContextId, DocumentEpoch, ObservedNodeHandle,
    Origin,
};

/// Public browser-authority registry with raw node minting kept inside the crate.
///
/// Browser-session, browsing-context, document-epoch, and canonical-origin lifecycle operations are
/// public because trusted adapters need them to maintain current authority. Converting untrusted
/// browser-protocol node identifiers into [`ObservedNodeHandle`] values is deliberately
/// crate-private: external callers must use a reviewed semantic-observation admission boundary such
/// as [`crate::WebDriverBiDiAccessibilityQuery::bind_current_nodes`], which consumes the required
/// protocol-use proof, validates the complete batch, and revalidates the exact current document
/// before atomically minting handles.
pub struct BrowserAuthorityRegistry {
    inner: RawBrowserAuthorityRegistry,
}

impl BrowserAuthorityRegistry {
    /// Create an empty registry with the reviewed default per-namespace identifier capacity.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RawBrowserAuthorityRegistry::new(),
        }
    }

    /// Create an empty registry with a caller-selected per-namespace identifier capacity.
    ///
    /// Session, browsing-context, and node identifiers retain independent monotonic namespaces.
    /// The node namespace is still reachable only through crate-owned semantic admission.
    #[must_use]
    pub fn with_identifier_limit(maximum_identifier: u64) -> Self {
        Self {
            inner: RawBrowserAuthorityRegistry::with_identifier_limit(maximum_identifier),
        }
    }

    /// Register one opaque external browser-session identifier.
    pub fn register_session(
        &mut self,
        external_identifier: &str,
    ) -> Result<BrowserSessionId, BrowserRegistryError> {
        self.inner.register_session(external_identifier)
    }

    /// Register one opaque external browsing-context identifier inside a known browser session.
    pub fn register_context(
        &mut self,
        browser_session: BrowserSessionId,
        external_identifier: &str,
    ) -> Result<BrowsingContextId, BrowserRegistryError> {
        self.inner
            .register_context(browser_session, external_identifier)
    }

    /// Retire one browsing context and all registry-local authority derived from it.
    pub fn remove_context(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<(), BrowserRegistryError> {
        self.inner.remove_context(browsing_context)
    }

    /// Retire one browser session and every registered context and node binding beneath it.
    pub fn remove_session(
        &mut self,
        browser_session: BrowserSessionId,
    ) -> Result<(), BrowserRegistryError> {
        self.inner.remove_session(browser_session)
    }

    /// Return the currently active document epoch for a known browsing context.
    pub fn current_epoch(
        &self,
        browsing_context: BrowsingContextId,
    ) -> Result<DocumentEpoch, BrowserRegistryError> {
        self.inner.current_epoch(browsing_context)
    }

    /// Return the current document epoch only when the supplied session owns the context.
    pub fn current_context_epoch(
        &self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
    ) -> Result<DocumentEpoch, BrowserRegistryError> {
        self.inner
            .current_context_epoch(browser_session, browsing_context)
    }

    /// Bind the canonical origin observed for the exact current browser document.
    pub fn bind_context_origin(
        &mut self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: &Origin,
    ) -> Result<DocumentEpoch, BrowserRegistryError> {
        self.inner
            .bind_context_origin(browser_session, browsing_context, origin)
    }

    /// Revalidate the canonical origin bound to the exact current browser document.
    pub fn require_context_origin(
        &self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: &Origin,
    ) -> Result<DocumentEpoch, BrowserRegistryError> {
        self.inner
            .require_context_origin(browser_session, browsing_context, origin)
    }

    /// Advance a browsing context to the next document epoch and invalidate old node bindings.
    pub fn advance_document(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<DocumentEpoch, BrowserRegistryError> {
        self.inner.advance_document(browsing_context)
    }

    /// Bind one admitted batch of adapter-local node identifiers to current browser authority.
    ///
    /// This operation is intentionally crate-private. The raw registry commits the batch only when
    /// every identifier can be bound; a later failure rolls back node identifiers and any origin
    /// binding created by the batch before the error is returned. Production callers outside this
    /// crate therefore cannot bypass semantic admission or observe partial authority from a failed
    /// `locateNodes` result.
    pub(crate) fn bind_nodes(
        &mut self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: &Origin,
        external_identifiers: &[&str],
    ) -> Result<Vec<ObservedNodeHandle>, BrowserRegistryError> {
        self.inner.bind_nodes(
            browser_session,
            browsing_context,
            origin,
            external_identifiers,
        )
    }
}

impl Default for BrowserAuthorityRegistry {
    fn default() -> Self {
        Self::new()
    }
}
