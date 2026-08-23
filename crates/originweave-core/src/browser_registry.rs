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
        va²È="25ÁÉ¥Ù…Ñ•}½¹Ñ•áÑ}Í•ÍÍ¥½¹}½ÉÉÕÁÑ¥½¸ ¤ì(€€€€€€€±•ÐµÕÐÉ•¥ÍÑÉä€ô	É½ÝÍ•ÉÕÑ¡½É¥ÑåI•¥ÍÑÉäèé¹•Ü ¤ì(€€€€€€€±•Ð½Ý¹•ÉÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}Í•ÍÍ¥½¸ ‰½ÉÉÕÁÐµ½Ý¹•ÈµÍ•ÍÍ¥½¸ˆ¤¤ì(€€€€€€€±•Ð…ÑÑ…­•ÉÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}Í•ÍÍ¥½¸ ‰½ÉÉÕÁÐµ…ÑÑ…­•ÈµÍ•ÍÍ¥½¸ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½Ý¹•ÉÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡…ÑÑ…­•ÉÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð½Ý¹•È€ô½Ý¹•ÉÍlÁtì(€€€€€€€±•Ð…ÑÑ…­•È€ô…ÑÑ…­•ÉÍlÁtì((€€€€€€€±•Ð½¹Ñ•áÑÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}½¹Ñ•áÐ¡½Ý¹•È°€‰½ÉÉÕÁÐµ½¹Ñ•áÐµÍ•ÍÍ¥½¸ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½¹Ñ•áÑÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð½¹Ñ•áÐ€ô½¹Ñ•áÑÍlÁtì(€€€€€€€±•Ð½É¥¥¹Ì€ôÙ…±Õ•Ì¡=É¥¥¸èéÁ…ÉÍ” ‰¡ÑÑÀè¼¼ÄÈÜ¸À¸À¸ÄèÐÌÄÈÜˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½É¥¥¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð¡…¹‘±•Ì€ô(€€€€€€€€€€€Ù…±Õ•Ì¡É•¥ÍÑÉä¹‰¥¹‘}¹½‘”¡½Ý¹•È°½¹Ñ•áÐ°€™½É¥¥¹ÍlÁt°€‰½ÉÉÕÁÐµ½¹Ñ•áÐµ¹½‘”ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡¡…¹‘±•Ì¹±•¸ ¤°€Ä¤ì((€€€€€€€É•¥ÍÑÉä¹½¹Ñ•áÑ}Í•ÍÍ¥½¸¹¥¹Í•ÉÐ¡½¹Ñ•áÐ°…ÑÑ…­•È¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„ (€€€€€€€€€€€É•¥ÍÑÉä¹Ù…±¥‘…Ñ•}¹½‘•}¡…¹‘±” ™¡…¹‘±•ÍlÁt¤°(€€€€€€€€€€€ÉÈ¡	É½ÝÍ•ÉI•¥ÍÑÉåÉÉ½ÈèéU¹­¹½Ý¹9½‘•ÕÑ¡½É¥Ñä¤(€€€€€€€€¤ì(€€€ô((€€€€mÑ•ÍÑt(€€€™¸¥ÍÍÕ•‘}¡…¹‘±•}É•©•ÑÍ}ÁÉ¥Ù…Ñ•}½¹Ñ•áÑ}½É¥¥¹}½ÉÉÕÁÑ¥½¸ ¤ì(€€€€€€€±•ÐµÕÐÉ•¥ÍÑÉä€ô	É½ÝÍ•ÉÕÑ¡½É¥ÑåI•¥ÍÑÉäèé¹•Ü ¤ì(€€€€€€€±•ÐÍ•ÍÍ¥½¹Ì€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}Í•ÍÍ¥½¸ ‰½ÉÉÕÁÐµ½É¥¥¸µÍ•ÍÍ¥½¸ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡Í•ÍÍ¥½¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•ÐÍ•ÍÍ¥½¸€ôÍ•ÍÍ¥½¹ÍlÁtì(€€€€€€€±•Ð½¹Ñ•áÑÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}½¹Ñ•áÐ¡Í•ÍÍ¥½¸°€‰½ÉÉÕÁÐµ½É¥¥¸µ½¹Ñ•áÐˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½¹Ñ•áÑÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð½¹Ñ•áÐ€ô½¹Ñ•áÑÍlÁtì(€€€€€€€±•Ð½É¥¥¹Ì€ôÙ…±Õ•Ì¡=É¥¥¸èéÁ…ÉÍ” ‰¡ÑÑÀè¼¼ÄÈÜ¸À¸À¸ÄèÐÌÄÈÜˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½É¥¥¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð¡…¹‘±•Ì€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹‰¥¹‘}¹½‘” (€€€€€€€€€€€Í•ÍÍ¥½¸°(€€€€€€€€€€€½¹Ñ•áÐ°(€€€€€€€€€€€€™½É¥¥¹ÍlÁt°(€€€€€€€€€€€€‰½ÉÉÕÁÐµ½É¥¥¸µ¹½‘”ˆ°(€€€€€€€€¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡¡…¹‘±•Ì¹±•¸ ¤°€Ä¤ì((€€€€€€€±•ÐÉ•Á±…•µ•¹Ñ}½É¥¥¹Ì€ôÙ…±Õ•Ì¡=É¥¥¸èéÁ…ÉÍ” ‰¡ÑÑÀè¼½±½…±¡½ÍÐèÐÌÄÈÜˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡É•Á±…•µ•¹Ñ}½É¥¥¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€É•¥ÍÑÉä(€€€€€€€€€€€€¹½¹Ñ•áÑ}½É¥¥¸(€€€€€€€€€€€€¹¥¹Í•ÉÐ¡½¹Ñ•áÐ°É•Á±…•µ•¹Ñ}½É¥¥¹ÍlÁt¹±½¹” ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„ (€€€€€€€€€€€É•¥ÍÑÉä¹Ù…±¥‘…Ñ•}¹½‘•}¡…¹‘±” ™¡…¹‘±•ÍlÁt¤°(€€€€€€€€€€€ÉÈ¡	É½ÝÍ•ÉI•¥ÍÑÉåÉÉ½ÈèéU¹­¹½Ý¹9½‘•ÕÑ¡½É¥Ñä¤(€€€€€€€€¤ì(€€€ô((€€€€mÑ•ÍÑt(€€€™¸Õ¹¥Ñ}™}•ÉÉ½É}ÁÉ½Á……Ñ¥½¹}½Ù•ÉÍ}ÁÉ¥Ù…Ñ•}™…¥±}±½Í•‘}‰½Õ¹‘…É¥•Ì ¤ì(€€€€€€€±•ÐµÕÐÉ•¥ÍÑÉä€ô	É½ÝÍ•ÉÕÑ¡½É¥ÑåI•¥ÍÑÉäèé¹•Ü ¤ì(€€€€€€€±•ÐÍ•ÍÍ¥½¹Ì€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}Í•ÍÍ¥½¸ ‰‰½Õ¹‘…ÉäµÍ•ÍÍ¥½¸ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡Í•ÍÍ¥½¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•ÐÍ•ÍÍ¥½¸€ôÍ•ÍÍ¥½¹ÍlÁtì(€€€€€€€…ÍÍ•ÉÑ}•Ä„ (€€€€€€€€€€€É•¥ÍÑÉä¹É•¥ÍÑ•É}½¹Ñ•áÐ¡Í•ÍÍ¥½¸°€ˆˆ¤°(€€€€€€€€€€€ÉÈ¡	É½ÝÍ•ÉI•¥ÍÑÉåÉÉ½Èèé%¹Ù…±¥‘áÑ•É¹…±%‘•¹Ñ¥™¥•È¤(€€€€€€€€¤ì((€€€€€€€±•Ð½¹Ñ•áÑÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}½¹Ñ•áÐ¡Í•ÍÍ¥½¸°€‰‰½Õ¹‘…Éäµ½¹Ñ•áÐˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½¹Ñ•áÑÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð½¹Ñ•áÐ€ô½¹Ñ•áÑÍlÁtì(€€€€€€€±•Ð½É¥¥¹Ì€ôÙ…±Õ•Ì¡=É¥¥¸èéÁ…ÉÍ” ‰¡ÑÑÀè¼¼ÄÈÜ¸À¸À¸ÄèÐÌÄÈÜˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½É¥¥¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð½É¥¥¸€ô€™½É¥¥¹ÍlÁtì(€€€€€€€…ÍÍ•ÉÑ}•Ä„ (€€€€€€€€€€€É•¥ÍÑÉä¹‰¥¹‘}¹½‘”¡Í•ÍÍ¥½¸°½¹Ñ•áÐ°½É¥¥¸°€ˆˆ¤°(€€€€€€€€€€€ÉÈ¡	É½ÝÍ•ÉI•¥ÍÑÉåÉÉ½Èèé%¹Ù…±¥‘áÑ•É¹…±%‘•¹Ñ¥™¥•È¤(€€€€€€€€¤ì((€€€€€€€±•Ð•Á½¡Ì€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹ÕÉÉ•¹Ñ}•Á½ ¡½¹Ñ•áÐ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡•Á½¡Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð•Á½ €ô•Á½¡ÍlÁtì(€€€€€€€É•¥ÍÑÉä¹½¹Ñ•áÑ}•Á½ ¹É•µ½Ù” ™½¹Ñ•áÐ¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„ (€€€€€€€€€€€É•¥ÍÑÉä¹‰¥¹‘}¹½‘”¡Í•ÍÍ¥½¸°½¹Ñ•áÐ°½É¥¥¸°€‰µ¥ÍÍ¥¹œµ•Á½ µ¹½‘”ˆ¤°(€€€€€€€€€€€ÉÈ¡	É½ÝÍ•ÉI•¥ÍÑÉåÉÉ½ÈèéU¹­¹½Ý¹	É½ÝÍ¥¹½¹Ñ•áÐ¤(€€€€€€€€¤ì((€€€€€€€É•¥ÍÑÉä¹½¹Ñ•áÑ}•Á½ ¹¥¹Í•ÉÐ¡½¹Ñ•áÐ°•Á½ ¤ì(€€€€€€€±•Ð¡…¹‘±•Ì€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹‰¥¹‘}¹½‘”¡Í•ÍÍ¥½¸°½¹Ñ•áÐ°½É¥¥¸°€‰±¥Ù”µ¹½‘”ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡¡…¹‘±•Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€É•¥ÍÑÉä¹½¹Ñ•áÑ}•Á½ ¹É•µ½Ù” ™½¹Ñ•áÐ¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„ (€€€€€€€€€€€É•¥ÍÑÉä¹Ù…±¥‘…Ñ•}¹½‘•}¡…¹‘±” ™¡…¹‘±•ÍlÁt¤°(€€€€€€€€€€€ÉÈ¡	É½ÝÍ•ÉI•¥ÍÑÉåÉÉ½ÈèéU¹­¹½Ý¹	É½ÝÍ¥¹½¹Ñ•áÐ¤(€€€€€€€€¤ì(€€€ô((€€€€mÑ•ÍÑt(€€€™¸¹½‘•}É•Ñ¥É•µ•¹Ñ}ÁÕÉ•Í}‘ÕÁ±¥…Ñ•}ÁÉ¥Ù…Ñ•}…±¥…Í•Í}™…¥±}±½Í• ¤ì(€€€€€€€±•ÐµÕÐÉ•¥ÍÑÉä€ô	É½ÝÍ•ÉÕÑ¡½É¥ÑåI•¥ÍÑÉäèé¹•Ü ¤ì(€€€€€€€±•ÐÍ•ÍÍ¥½¹Ì€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}Í•ÍÍ¥½¸ ‰É•Ñ¥É•µ•¹ÐµÍ•ÍÍ¥½¸ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡Í•ÍÍ¥½¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•ÐÍ•ÍÍ¥½¸€ôÍ•ÍÍ¥½¹ÍlÁtì(€€€€€€€±•Ð½¹Ñ•áÑÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}½¹Ñ•áÐ¡Í•ÍÍ¥½¸°€‰É•Ñ¥É•µ•¹Ðµ½¹Ñ•áÐˆ¤¤ì(€€€€€€€±•Ð½Ñ¡•É}½¹Ñ•áÑÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}½¹Ñ•áÐ¡Í•ÍÍ¥½¸°€‰½Ñ¡•ÈµÉ•Ñ¥É•µ•¹Ðµ½¹Ñ•áÐˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½¹Ñ•áÑÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½Ñ¡•É}½¹Ñ•áÑÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð½¹Ñ•áÐ€ô½¹Ñ•áÑÍlÁtì(€€€€€€€±•Ð½Ñ¡•É}½¹Ñ•áÐ€ô½Ñ¡•É}½¹Ñ•áÑÍlÁtì(€€€€€€€±•Ð½É¥¥¹Ì€ôÙ…±Õ•Ì¡=É¥¥¸èéÁ…ÉÍ” ‰¡ÑÑÀè¼¼ÄÈÜ¸À¸À¸ÄèÐÌÄÈÜˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½É¥¥¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð½É¥¥¸€ô€™½É¥¥¹ÍlÁtì(€€€€€€€±•ÐÑ…É•ÑÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹‰¥¹‘}¹½‘”¡Í•ÍÍ¥½¸°½¹Ñ•áÐ°½É¥¥¸°€‰Ñ…É•Ðµ¹½‘”ˆ¤¤ì(€€€€€€€±•ÐÍ¥‰±¥¹Ì€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹‰¥¹‘}¹½‘”¡Í•ÍÍ¥½¸°½¹Ñ•áÐ°½É¥¥¸°€‰Í¥‰±¥¹œµ¹½‘”ˆ¤¤ì(€€€€€€€±•Ð½Ñ¡•ÉÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹‰¥¹‘}¹½‘”¡Í•ÍÍ¥½¸°½Ñ¡•É}½¹Ñ•áÐ°½É¥¥¸°€‰½Ñ¡•Èµ¹½‘”ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡Ñ…É•ÑÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡Í¥‰±¥¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½Ñ¡•ÉÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€±•ÐÑ…É•Ð€ô€™Ñ…É•ÑÍlÁtì(€€€€€€€±•ÐÍ¥‰±¥¹œ€ô€™Í¥‰±¥¹ÍlÁtì(€€€€€€€±•Ð½Ñ¡•È€ô€™½Ñ¡•ÉÍlÁtì((€€€€€€€±•Ð•Á½¡Ì€ôÙ…±Õ•Ì¡½Õµ•¹ÑÁ½ èé¹•Ü¡Ñ…É•Ð¹‘½Õµ•¹Ñ}•Á½  ¤¹Ù…±Õ” ¤€¬€Ä¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡•Á½¡Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð™ÕÑÕÉ•}­•ä€ô€¡½¹Ñ•áÐ°•Á½¡ÍlÁt°€‰½ÉÉÕÁÐµ™ÕÑÕÉ”µ…±¥…Ìˆ¹Ñ½}½Ý¹• ¤¤ì(€€€€€€€±•ÐÉ½ÍÍ}½¹Ñ•áÑ}­•ä€ô€ (€€€€€€€€€€€½Ñ¡•É}½¹Ñ•áÐ°(€€€€€€€€€€€Ñ…É•Ð¹‘½Õµ•¹Ñ}•Á½  ¤°(€€€€€€€€€€€€‰½ÉÉÕÁÐµÉ½ÍÌµ½¹Ñ•áÐµ…±¥…Ìˆ¹Ñ½}½Ý¹• ¤°(€€€€€€€€¤ì(€€€€€€€É•¥ÍÑÉä(€€€€€€€€€€€€¹¹½‘•}‰å}•áÑ•É¹…°(€€€€€€€€€€€€¹¥¹Í•ÉÐ¡™ÕÑÕÉ•}­•ä¹±½¹” ¤°Ñ…É•Ð¹¹½‘•}¥ ¤¤ì(€€€€€€€É•¥ÍÑÉä(€€€€€€€€€€€€¹¹½‘•}‰å}•áÑ•É¹…°(€€€€€€€€€€€€¹¥¹Í•ÉÐ¡É½ÍÍ}½¹Ñ•áÑ}­•ä¹±½¹” ¤°Ñ…É•Ð¹¹½‘•}¥ ¤¤ì((€€€€€€€…ÍÍ•ÉÑ}•Ä„¡É•¥ÍÑÉä¹É•µ½Ù•}¹½‘”¡Ñ…É•Ð¤°=¬  ¤¤¤ì((€€€€€€€…ÍÍ•ÉÑ}•Ä„¡É•¥ÍÑÉä¹Ù…±¥‘…Ñ•}¹½‘•}¡…¹‘±”¡Í¥‰±¥¹œ¤°=¬  ¤¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡É•¥ÍÑÉä¹Ù…±¥‘…Ñ•}¹½‘•}¡…¹‘±”¡½Ñ¡•È¤°=¬  ¤¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡É•¥ÍÑÉä¹¹½‘•}‰å}•áÑ•É¹…°¹•Ð ™™ÕÑÕÉ•}­•ä¤°9½¹”¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡É•¥ÍÑÉä¹¹½‘•}‰å}•áÑ•É¹…°¹•Ð ™É½ÍÍ}½¹Ñ•áÑ}­•ä¤°9½¹”¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡É•¥ÍÑÉä¹¹½‘•}‰¥¹‘¥¹}‰å}¥¹•Ð ™Ñ…É•Ð¹¹½‘•}¥ ¤¤°9½¹”¤ì(€€€ô((€€€€mÑ•ÍÑt(€€€™¸‘½Õµ•¹Ñ}•Á½¡}•á¡…ÕÍÑ¥½¹}¥Í}™…¥±}±½Í• ¤ì(€€€€€€€±•ÐµÕÐÉ•¥ÍÑÉä€ô	É½ÝÍ•ÉÕÑ¡½É¥ÑåI•¥ÍÑÉäèé¹•Ü ¤ì(€€€€€€€±•ÐÍ•ÍÍ¥½¹Ì€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}Í•ÍÍ¥½¸ ‰•Á½ µÍ•ÍÍ¥½¸ˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡Í•ÍÍ¥½¹Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€±•ÐÍ•ÍÍ¥½¸€ôÍ•ÍÍ¥½¹ÍlÁtì(€€€€€€€±•Ð½¹Ñ•áÑÌ€ôÙ…±Õ•Ì¡É•¥ÍÑÉä¹É•¥ÍÑ•É}½¹Ñ•áÐ¡Í•ÍÍ¥½¸°€‰•Á½ µ½¹Ñ•áÐˆ¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡½¹Ñ•áÑÌ¹±•¸ ¤°€Ä¤ì(€€€€€€€±•Ð½¹Ñ•áÐ€ô½¹Ñ•áÑÍlÁtì(€€€€€€€±•Ðµ…á¥µÕµ}•Á½¡Ì€ôÙ…±Õ•Ì¡½Õµ•¹ÑÁ½ èé¹•Ü¡ÔØÐèé5`¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡µ…á¥µÕµ}•Á½¡Ì¹±•¸ ¤°€Ä¤ì(€€€€€€€É•¥ÍÑÉä¹½¹Ñ•áÑ}•Á½ ¹¥¹Í•ÉÐ¡½¹Ñ•áÐ°µ…á¥µÕµ}•Á½¡ÍlÁt¤ì((€€€€€€€…ÍÍ•ÉÑ}•Ä„ (€€€€€€€€€€€É•¥ÍÑÉä¹…‘Ù…¹•}‘½Õµ•¹Ð¡½¹Ñ•áÐ¤°(€€€€€€€€€€€ÉÈ¡	É½ÝÍ•ÉI•¥ÍÑÉåÉÉ½Èèé½Õµ•¹ÑÁ½¡á¡…ÕÍÑ•¤(€€€€€€€€¤ì(€€€ô((€€€€mÑ•ÍÑt(€€€™¸µ½¹½Ñ½¹¥}¥‘•¹Ñ¥™¥•É}•á¡…ÕÍÑ¥½¹}¥Í}™…¥±}±½Í• ¤ì(€€€€€€€±•ÐµÕÐ¹•áÐ€ô€Äì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡Ñ…­•}¥‘•¹Ñ¥™¥•È ™µÕÐ¹•áÐ°€Ä¤°=¬ Ä¤¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡¹•áÐ°€È¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„ (€€€€€€€€€€€Ñ…­•}¥‘•¹Ñ¥™¥•È ™µÕÐ¹•áÐ°€Ä¤°(€€€€€€€€€€€ÉÈ¡	É½ÝÍ•ÉI•¥ÍÑÉåÉÉ½Èèé%‘•¹Ñ¥™¥•ÉMÁ…•á¡…ÕÍÑ•¤(€€€€€€€€¤ì(€€€ô((€€€€mÑ•ÍÑt(€€€™¸µ…á¥µÕµ}¥‘•¹Ñ¥™¥•É}±¥µ¥Ñ}¥Í}±…µÁ•‘}Ý¥Ñ¡½ÕÑ}ÝÉ…ÁÁ¥¹œ ¤ì(€€€€€€€±•ÐÉ•¥ÍÑÉä€ô	É½ÝÍ•ÉÕÑ¡½É¥ÑåI•¥ÍÑÉäèéÝ¥Ñ¡}¥‘•¹Ñ¥™¥•É}±¥µ¥Ð¡ÔØÐèé5`¤ì(€€€€€€€…ÍÍ•ÉÑ}•Ä„¡É•¥ÍÑÉä¹µ…á¥µÕµ}¥‘•¹Ñ¥™¥•È°ÔØÐèé5`€´€Ä¤ì(€€€ô)ô(