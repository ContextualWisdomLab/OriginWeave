use std::fmt;

use crate::{
    BrowserAuthorityRegistry, BrowserProtocolAdapterDescriptor, BrowserProtocolCapability,
    BrowserProtocolKind, BrowserProtocolUseValidationError, BrowserRegistryError, BrowserSessionId,
    BrowsingContextId, DocumentEpoch, OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
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
}

/// Failure to compose current browser context ownership with protocol validation before dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserContextProtocolDispatchError {
    /// The supplied browser session/context pair is not current in the authority registry.
    BrowserAuthority(BrowserRegistryError),
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
            Self::ProtocolValidation(error) => Some(error),
        }
    }
}
