use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

use crate::browser_registry::BrowserAuthorityRegistry as RawBrowserAuthorityRegistry;
use crate::{
    BrowserRegistryError, BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeHandleError,
    ObservedNodeHandle, Origin,
};

/// A registry-issued node handle that carries opaque provenance in addition to descriptive node state.
///
/// The contained [`ObservedNodeHandle`] remains readable through [`Deref`], but only
/// [`BrowserAuthorityRegistry`] can construct this wrapper. Typed actions therefore can require
/// proof that a node came from the same live registry instance instead of trusting a publicly
/// reproducible session/context/origin/epoch/node tuple.
#[derive(Debug)]
pub struct AdmittedNodeHandle {
    observed: ObservedNodeHandle,
    registry_instance: Arc<()>,
}

impl Deref for AdmittedNodeHandle {
    type Target = ObservedNodeHandle;

    fn deref(&self) -> &Self::Target {
        &self.observed
    }
}

/// A fail-closed error while revalidating opaque registry-issued node authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmittedNodeAuthorityError {
    /// The supplied handle was issued by a different registry instance.
    ForeignRegistry,
    /// The registry no longer retains this exact admitted node binding.
    NotAdmitted,
    /// Current session, context, or canonical-origin authority no longer matches.
    BrowserAuthority(BrowserRegistryError),
    /// The handle no longer matches the registry's current document lifetime.
    NodeHandle(NodeHandleError),
}

impl Display for AdmittedNodeAuthorityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForeignRegistry => formatter
                .write_str("admitted node was issued by a different browser authority registry"),
            Self::NotAdmitted => formatter
                .write_str("admitted node authority is no longer retained by this registry"),
            Self::BrowserAuthority(error) => {
                write!(
                    formatter,
                    "admitted node browser authority rejected input: {error}"
                )
            }
            Self::NodeHandle(error) => {
                write!(
                    formatter,
                    "admitted node document authority rejected input: {error}"
                )
            }
        }
    }
}

impl Error for AdmittedNodeAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BrowserAuthority(error) => Some(error),
            Self::NodeHandle(error) => Some(error),
            Self::ForeignRegistry | Self::NotAdmitted => None,
        }
    }
}

/// Public browser-authority registry with raw node minting kept inside the crate.
///
/// Browser-session, browsing-context, document-epoch, and canonical-origin lifecycle operations are
/// public because trusted adapters need them to maintain current authority. Converting untrusted
/// browser-protocol node identifiers into descriptive [`ObservedNodeHandle`] values is deliberately
/// crate-private: external callers must use a reviewed semantic-observation admission boundary such
/// as [`crate::WebDriverBiDiAccessibilityQuery::bind_current_nodes`]. Action-capable paths use the
/// stricter registry-issued [`AdmittedNodeHandle`] wrapper so a copied descriptive tuple cannot
/// become typed-input authority.
pub struct BrowserAuthorityRegistry {
    inner: RawBrowserAuthorityRegistry,
    admitted_node_external_identifiers: BTreeMap<(u64, u64, u64, u64), String>,
    registry_instance: Arc<()>,
}

impl BrowserAuthorityRegistry {
    /// Create an empty registry with the reviewed default per-namespace identifier capacity.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RawBrowserAuthorityRegistry::new(),
            admitted_node_external_identifiers: BTreeMap::new(),
            registry_instance: Arc::new(()),
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
            admitted_node_external_identifiers: BTreeMap::new(),
            registry_instance: Arc::new(()),
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
        self.inner.remove_context(browsing_context)?;
        let browsing_context_value = browsing_context.value();
        self.admitted_node_external_identifiers.retain(
            |(_session, context, _epoch, _node), _external| *context != browsing_context_value,
        );
        Ok(())
    }

    /// Retire one browser session and every registered context and node binding beneath it.
    pub fn remove_session(
        &mut self,
        browser_session: BrowserSessionId,
    ) -> Result<(), BrowserRegistryError> {
        self.inner.remove_session(browser_session)?;
        let browser_session_value = browser_session.value();
        self.admitted_node_external_identifiers.retain(
            |(session, _context, _epoch, _node), _external| *session != browser_session_value,
        );
        Ok(())
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

    /// Require an opaque external browsing-context identifier to name this exact context.
    pub(crate) fn require_context_external_identifier(
        &self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        external_identifier: &str,
    ) -> Result<(), BrowserRegistryError> {
        self.inner.require_context_external_identifier(
            browser_session,
            browsing_context,
            external_identifier,
        )
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

    /// Revalidate one exact registry-issued admitted node before later typed dispatch.
    ///
    /// This check preserves opaque registry-instance provenance and verifies that this registry still
    /// retains the exact admitted node key under the current session, context, origin, and document
    /// epoch. It deliberately does not validate the adapter-local wire identifier; the final typed
    /// adapter constructor must still bind the exact `sharedId` immediately before browser I/O.
    pub fn validate_admitted_node_handle(
        &self,
        handle: &AdmittedNodeHandle,
    ) -> Result<(), AdmittedNodeAuthorityError> {
        if !Arc::ptr_eq(&self.registry_instance, &handle.registry_instance) {
            return Err(AdmittedNodeAuthorityError::ForeignRegistry);
        }
        if !self
            .admitted_node_external_identifiers
            .contains_key(&node_authority_key(&handle.observed))
        {
            return Err(AdmittedNodeAuthorityError::NotAdmitted);
        }

        let current_epoch = self
            .require_context_origin(
                handle.browser_session(),
                handle.browsing_context(),
                handle.origin(),
            )
            .map_err(AdmittedNodeAuthorityError::BrowserAuthority)?;
        handle
            .validate_current(
                handle.browser_session(),
                handle.browsing_context(),
                handle.origin(),
                current_epoch,
            )
            .map_err(AdmittedNodeAuthorityError::NodeHandle)
    }

    /// Advance a browsing context to the next document epoch and invalidate old node bindings.
    pub fn advance_document(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<DocumentEpoch, BrowserRegistryError> {
        let next_epoch = self.inner.advance_document(browsing_context)?;
        let browsing_context_value = browsing_context.value();
        self.admitted_node_external_identifiers.retain(
            |(_session, context, _epoch, _node), _external| *context != browsing_context_value,
        );
        Ok(next_epoch)
    }

    /// Bind one admitted batch of adapter-local identifiers as descriptive current-node evidence.
    ///
    /// This operation is intentionally crate-private. The raw registry commits the batch only when
    /// every identifier can be bound; a later failure rolls back node identifiers and any origin
    /// binding created by the batch before the error is returned. The exact external identifier is
    /// retained for later action admission, but the returned [`ObservedNodeHandle`] values remain
    /// descriptive and publicly reproducible rather than typed-input authority.
    pub(crate) fn bind_nodes(
        &mut self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: &Origin,
        external_identifiers: &[&str],
    ) -> Result<Vec<ObservedNodeHandle>, BrowserRegistryError> {
        let handles = self.inner.bind_nodes(
            browser_session,
            browsing_context,
            origin,
            external_identifiers,
        )?;
        for (handle, external_identifier) in handles.iter().zip(external_identifiers) {
            self.admitted_node_external_identifiers.insert(
                node_authority_key(handle),
                (*external_identifier).to_owned(),
            );
        }
        Ok(handles)
    }

    /// Bind one admitted batch and attach opaque registry-instance provenance for typed actions.
    pub(crate) fn bind_admitted_nodes(
        &mut self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        origin: &Origin,
        external_identifiers: &[&str],
    ) -> Result<Vec<AdmittedNodeHandle>, BrowserRegistryError> {
        self.bind_nodes(
            browser_session,
            browsing_context,
            origin,
            external_identifiers,
        )
        .map(|handles| {
            handles
                .into_iter()
                .map(|observed| AdmittedNodeHandle {
                    observed,
                    registry_instance: Arc::clone(&self.registry_instance),
                })
                .collect()
        })
    }

    /// Return whether this registry issued the handle under the exact supplied wire identifier.
    pub(crate) fn node_external_identifier_matches(
        &self,
        handle: &AdmittedNodeHandle,
        external_identifier: &str,
    ) -> bool {
        if !Arc::ptr_eq(&self.registry_instance, &handle.registry_instance) {
            return false;
        }
        self.admitted_node_external_identifiers
            .get(&node_authority_key(&handle.observed))
            .is_some_and(|admitted| admitted == external_identifier)
    }
}

impl Default for BrowserAuthorityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn node_authority_key(handle: &ObservedNodeHandle) -> (u64, u64, u64, u64) {
    (
        handle.browser_session().value(),
        handle.browsing_context().value(),
        handle.document_epoch().value(),
        handle.node_id(),
    )
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{AdmittedNodeAuthorityError, BrowserAuthorityRegistry};
    use crate::{BrowserRegistryError, Origin};

    #[test]
    #[cfg_attr(coverage, coverage(off))]
    fn admitted_node_revalidation_preserves_broken_registry_authority() -> Result<(), Box<dyn Error>>
    {
        let mut registry = BrowserAuthorityRegistry::new();
        let session = registry.register_session("broken-authority-session")?;
        let context = registry.register_context(session, "broken-authority-context")?;
        let origin = Origin::parse("https://example.com").map_err(|error| {
            std::io::Error::other(format!("fixture origin rejected: {error:?}"))
        })?;
        let handle = registry
            .bind_admitted_nodes(session, context, &origin, &["node"])?
            .pop()
            .ok_or("fixture did not bind its node")?;
        registry.inner.remove_context(context)?;

        assert_eq!(
            registry.validate_admitted_node_handle(&handle),
            Err(AdmittedNodeAuthorityError::BrowserAuthority(
                BrowserRegistryError::UnknownBrowsingContext
            ))
        );
        Ok(())
    }
}
