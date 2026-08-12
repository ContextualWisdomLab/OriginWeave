use crate::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolUseValidationError, OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
};

impl BrowserProtocolAdapterDescriptor {
    /// Validate current browser-protocol metadata and immediately invoke one dispatch callback.
    ///
    /// The runtime protocol family, adapter version, protocol revision, and browser revision must
    /// be sampled from the trusted adapter that is about to perform the operation. Validation
    /// occurs before `dispatch` is invoked, and the callback receives the resulting non-cloneable
    /// [`ValidatedBrowserProtocolUse`] by ownership so this boundary does not turn successful
    /// validation into reusable ambient authority.
    ///
    /// A successful callback invocation does not authenticate the adapter process, authorize a
    /// browser session, browsing context, origin, destination, secret, or approval, or prove a
    /// browser post-condition. Those remain separate higher-level execution boundaries.
    pub fn dispatch_if_runtime_matches<R, F>(
        &self,
        required_originweave_protocol_version: OriginWeaveProtocolVersion,
        runtime_kind: BrowserProtocolKind,
        runtime_adapter_version: &str,
        runtime_protocol_revision: &str,
        runtime_browser_revision: &str,
        required_capability: BrowserProtocolCapability,
        dispatch: F,
    ) -> Result<R, BrowserProtocolUseValidationError>
    where
        F: FnOnce(ValidatedBrowserProtocolUse) -> R,
    {
        let validated = self.validate_use(
            required_originweave_protocol_version,
            runtime_kind,
            runtime_adapter_version,
            runtime_protocol_revision,
            runtime_browser_revision,
            required_capability,
        )?;
        Ok(dispatch(validated))
    }
}
