use std::fmt;

use crate::{
    BrowserAuthorityRegistry, BrowserProtocolAdapterDescriptor, BrowserProtocolCapability,
    BrowserProtocolKind, BrowserProtocolUseValidationError, BrowserRegistryError, BrowserSessionId,
    BrowsingContextId, DocumentEpoch, Origin, OriginWeaveProtocolVersion,
    ValidatedBrowserProtocolUse,
};

/// Current runtime metadata sampled from the browser-protocol adapter about to perform I/O.
///
/// This value is untrusted descriptive input. Constructing it does not validate or authenticate an
/// adapter, browser, or protocol revision and grants no browser or Agent authority. The descriptor
/// validates every field against its reviewed metadata before a dispatch callback can run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrowserProtocolRuntimeMetadata<'a> {
    kind: BrowserProtocolKind,
    adapter_version: &'a str,
    protocol_revision: &'a str,
    browser_revision: &'a str,
}

impl<'a> BrowserProtocolRuntimeMetadata<'a> {
    /// Build one runtime metadata snapshot for immediate validation and dispatch.
    ///
    /// String syntax and descriptor equality are intentionally checked later by
    /// [`BrowserProtocolAdapterDescriptor::dispatch_if_runtime_matches`], so malformed caller data
    /// remains representable as input that the fail-closed boundary can reject deterministically.
    pub const fn new(
        kind: BrowserProtocolKind,
        adapter_version: &'a str,
        protocol_revision: &'a str,
        browser_revision: &'a str,
    ) -> Self {
        Self {
            kind,
            adapter_version,
            protocol_revision,
            browser_revision,
        }
    }
}

/// Exact OriginWeave browser session/context requested for one immediate protocol dispatch.
///
/// This value only keeps the two identifiers together so a caller cannot accidentally reorder or
/// independently substitute them at the dispatch boundary. Constructing or copying it does not
/// prove that either identifier is registered, current, or authorized; the authority registry must
/// validate the pair immediately before protocol metadata validation and callback invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrowserContextDispatchTarget {
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
}

impl BrowserContextDispatchTarget {
    /// Group one OriginWeave browser session and browsing context for immediate dispatch checking.
    #[must_use]
    pub const fn new(
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
    ) -> Self {
        Self {
            browser_session,
            browsing_context,
        }
    }

    /// Return the OriginWeave browser session requested for this dispatch.
    #[must_use]
    pub const fn browser_session(self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the OriginWeave browsing context requested for this dispatch.
    #[must_use]
    pub const fn browsing_context(self) -> BrowsingContextId {
        self.browsing_context
    }
}

/// Exact browser context plus the canonical origin expected immediately before protocol dispatch.
///
/// Grouping these values keeps one authority target explicit while avoiding a long positional
/// argument list at the dispatch boundary. Construction does not prove that the context is current
/// or that the origin is bound; [`BrowserProtocolAdapterDescriptor::dispatch_if_context_origin_current`]
/// performs those fail-closed checks immediately before protocol validation and callback execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrowserContextOriginDispatchTarget<'a> {
    context: BrowserContextDispatchTarget,
    expected_origin: &'a Origin,
}

impl<'a> BrowserContextOriginDispatchTarget<'a> {
    /// Group one browser context target with its freshly sampled canonical origin.
    #[must_use]
    pub const fn new(context: BrowserContextDispatchTarget, expected_origin: &'a Origin) -> Self {
        Self {
            context,
            expected_origin,
        }
    }

    /// Return the exact browser session/context pair requested for dispatch.
    #[must_use]
    pub const fn context(self) -> BrowserContextDispatchTarget {
        self.context
    }

    /// Return the canonical origin expected to remain current for the dispatch.
    #[must_use]
    pub const fn expected_origin(self) -> &'a Origin {
        self.expected_origin
    }
}

/// Exact browser context, canonical origin, and observed document epoch for one protocol dispatch.
///
/// This target is intended for actions whose authority was derived from a prior structured browser
/// observation. Construction grants no authority. The dispatch boundary must revalidate the
/// session/context/origin and prove that the registry is still at `expected_epoch` immediately
/// before protocol metadata validation and callback execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrowserContextOriginEpochDispatchTarget<'a> {
    context_origin: BrowserContextOriginDispatchTarget<'a>,
    expected_epoch: DocumentEpoch,
}

impl<'a> BrowserContextOriginEpochDispatchTarget<'a> {
    /// Bind one immediate-use context/origin target to the document epoch that was observed.
    #[must_use]
    pub const fn new(
        context_origin: BrowserContextOriginDispatchTarget<'a>,
        expected_epoch: DocumentEpoch,
    ) -> Self {
        Self {
            context_origin,
            expected_epoch,
        }
    }

    /// Return the exact browser context and canonical origin requested for dispatch.
    #[must_use]
    pub const fn context_origin(self) -> BrowserContextOriginDispatchTarget<'a> {
        self.context_origin
    }

    /// Return the exact document epoch whose observation authorized the requested action.
    #[must_use]
    pub const fn expected_epoch(self) -> DocumentEpoch {
        self.expected_epoch
    }
}

impl BrowserProtocolAdapterDescriptor {
    /// Validate current browser-protocol metadata and immediately invoke one dispatch callback.
    ///
    /// `runtime_metadata` must be sampled from the trusted adapter that is about to perform the
    /// operation. Validation occurs before `dispatch` is invoked, and the callback receives the
    /// resulting non-cloneable [`ValidatedBrowserProtocolUse`] by ownership so this boundary does
    /// not turn successful validation into reusable ambient authority.
    ///
    /// A successful callback invocation does not authenticate the adapter process, authorize a
    /// browser session, browsing context, origin, destination, secret, or approval, or prove a
    /// browser post-condition. Those remain separate higher-level execution boundaries.
    pub fn dispatch_if_runtime_matches<R, F>(
        &self,
        required_originweave_protocol_version: OriginWeaveProtocolVersion,
        runtime_metadata: BrowserProtocolRuntimeMetadata<'_>,
        required_capability: BrowserProtocolCapability,
        dispatch: F,
    ) -> Result<R, BrowserProtocolUseValidationError>
    where
        F: FnOnce(ValidatedBrowserProtocolUse) -> R,
    {
        let validated = self.validate_use(
            required_originweave_protocol_version,
            runtime_metadata.kind,
            runtime_metadata.adapter_version,
            runtime_metadata.protocol_revision,
            runtime_metadata.browser_revision,
            required_capability,
        )?;
        Ok(dispatch(validated))
    }

    /// Revalidate exact browser session/context ownership and runtime metadata before dispatch.
    ///
    /// The registry check occurs first and returns its current document epoch. The exact protocol
    /// generation, runtime protocol family, adapter version, upstream/browser revisions, and
    /// required capability are then validated before `dispatch` can run. The callback receives the
    /// non-cloneable protocol-use proof plus the registry epoch sampled for this immediate use.
    ///
    /// This is a composition prerequisite, not complete browser-action authority. In particular,
    /// typed input still requires separate current origin/document/node and deterministic policy
    /// authorization, while navigation still requires destination/network/TLS/HTTP authority.
    /// The caller remains responsible for sampling runtime metadata from the adapter about to
    /// perform I/O and for preventing registry mutation across its larger execution transaction.
    pub fn dispatch_if_context_current<R, F>(
        &self,
        authority_registry: &BrowserAuthorityRegistry,
        target: BrowserContextDispatchTarget,
        required_originweave_protocol_version: OriginWeaveProtocolVersion,
        runtime_metadata: BrowserProtocolRuntimeMetadata<'_>,
        required_capability: BrowserProtocolCapability,
        dispatch: F,
    ) -> Result<R, BrowserContextProtocolDispatchError>
    where
        F: FnOnce(ValidatedBrowserProtocolUse, DocumentEpoch) -> R,
    {
        let current_epoch = authority_registry
            .current_context_epoch(target.browser_session(), target.browsing_context())
            .map_err(BrowserContextProtocolDispatchError::BrowserAuthority)?;
        self.dispatch_if_runtime_matches(
            required_originweave_protocol_version,
            runtime_metadata,
            required_capability,
            |validated| dispatch(validated, current_epoch),
        )
        .map_err(BrowserContextProtocolDispatchError::ProtocolValidation)
    }

    /// Revalidate exact browser session/context/origin authority and protocol metadata before I/O.
    ///
    /// The registry first proves that `target.expected_origin()` is the canonical origin currently
    /// bound to the supplied browser session and browsing context and returns that document's
    /// current epoch. Only then are the exact protocol generation, runtime protocol family, adapter
    /// version, upstream/browser revisions, and required capability validated. `dispatch` receives
    /// both the non-cloneable protocol-use proof and the epoch sampled by that origin revalidation.
    ///
    /// This method does not derive the origin from Chromium, authenticate the adapter process,
    /// authorize a destination/network/TLS/HTTP operation, grant Agent capability or approval, or
    /// prove a post-condition. The caller must construct `target` from the origin freshly sampled
    /// from the trusted adapter about to perform I/O, sample `runtime_metadata` from that same
    /// adapter, and prevent intervening registry mutation across its larger execution transaction.
    pub fn dispatch_if_context_origin_current<R, F>(
        &self,
        authority_registry: &BrowserAuthorityRegistry,
        target: BrowserContextOriginDispatchTarget<'_>,
        required_originweave_protocol_version: OriginWeaveProtocolVersion,
        runtime_metadata: BrowserProtocolRuntimeMetadata<'_>,
        required_capability: BrowserProtocolCapability,
        dispatch: F,
    ) -> Result<R, BrowserContextProtocolDispatchError>
    where
        F: FnOnce(ValidatedBrowserProtocolUse, DocumentEpoch) -> R,
    {
        let context = target.context();
        let current_epoch = authority_registry
            .require_context_origin(
                context.browser_session(),
                context.browsing_context(),
                target.expected_origin(),
            )
            .map_err(BrowserContextProtocolDispatchError::BrowserAuthority)?;
        self.dispatch_if_runtime_matches(
            required_originweave_protocol_version,
            runtime_metadata,
            required_capability,
            |validated| dispatch(validated, current_epoch),
        )
        .map_err(BrowserContextProtocolDispatchError::ProtocolValidation)
    }

    /// Revalidate exact browser session/context/origin/document authority before protocol I/O.
    ///
    /// This stronger action boundary first proves the exact current session/context/origin through
    /// the authority registry, then compares the registry's current document epoch with the epoch
    /// that produced the caller's observation. A same-origin navigation therefore fails closed
    /// before protocol validation or callback execution even when the canonical origin is rebound.
    /// Exact protocol generation, family, adapter version, protocol/browser revisions and required
    /// capability are validated only after the document remains current.
    ///
    /// The caller remains responsible for deriving the origin and observed epoch from the trusted
    /// adapter/observation that produced the action, sampling runtime protocol metadata from the
    /// adapter about to perform I/O, and preventing intervening mutation across the larger
    /// transaction. This method does not authenticate Chromium, authorize destination/network
    /// authority or policy approval, validate semantic node state, perform I/O, or prove success.
    pub fn dispatch_if_context_origin_epoch_current<R, F>(
        &self,
        authority_registry: &BrowserAuthorityRegistry,
        target: BrowserContextOriginEpochDispatchTarget<'_>,
        required_originweave_protocol_version: OriginWeaveProtocolVersion,
        runtime_metadata: BrowserProtocolRuntimeMetadata<'_>,
        required_capability: BrowserProtocolCapability,
        dispatch: F,
    ) -> Result<R, BrowserContextProtocolDispatchError>
    where
        F: FnOnce(ValidatedBrowserProtocolUse, DocumentEpoch) -> R,
    {
        let context_origin = target.context_origin();
        let context = context_origin.context();
        let current_epoch = authority_registry
            .require_context_origin(
                context.browser_session(),
                context.browsing_context(),
                context_origin.expected_origin(),
            )
            .map_err(BrowserContextProtocolDispatchError::BrowserAuthority)?;
        if current_epoch != target.expected_epoch() {
            return Err(BrowserContextProtocolDispatchError::DocumentEpochMismatch {
                expected: target.expected_epoch(),
                current: current_epoch,
            });
        }
        self.dispatch_if_runtime_matches(
            required_originweave_protocol_version,
            runtime_metadata,
            required_capability,
            |validated| dispatch(validated, current_epoch),
        )
        .map_err(BrowserContextProtocolDispatchError::ProtocolValidation)
    }
}

/// Failure to compose current browser context ownership with protocol validation before dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserContextProtocolDispatchError {
    /// The supplied browser session/context pair is not current in the authority registry.
    BrowserAuthority(BrowserRegistryError),
    /// The observed document epoch no longer matches the registry's current document.
    DocumentEpochMismatch {
        /// The document epoch that produced the action's observation.
        expected: DocumentEpoch,
        /// The document epoch currently active in the registry.
        current: DocumentEpoch,
    },
    /// The current browser-protocol metadata or required capability failed validation.
    ProtocolValidation(BrowserProtocolUseValidationError),
}

impl fmt::Display for BrowserContextProtocolDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrowserAuthority(error) => {
                write!(
                    formatter,
                    "browser context authority denied protocol dispatch: {error}"
                )
            }
            Self::DocumentEpochMismatch { expected, current } => write!(
                formatter,
                "browser document epoch {} no longer matches observed epoch {}",
                current.value(),
                expected.value()
            ),
            Self::ProtocolValidation(error) => {
                write!(
                    formatter,
                    "browser protocol validation denied context dispatch: {error}"
                )
            }
        }
    }
}

impl std::error::Error for BrowserContextProtocolDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BrowserAuthority(error) => Some(error),
            Self::DocumentEpochMismatch { .. } => None,
            Self::ProtocolValidation(error) => Some(error),
        }
    }
}
