use crate::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolUseValidationError, OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
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
}
