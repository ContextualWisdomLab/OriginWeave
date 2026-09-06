use crate::{BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, BrowsingContextId};

impl BrowserAuthorityRegistry {
    /// Require an external browser-protocol context identifier to name one exact registered context.
    ///
    /// This is a read-only adapter boundary. It reuses the registry's existing validation and
    /// session/context ownership checks and never registers a context as a side effect of untrusted
    /// protocol evidence. Success does not authenticate a browser process, authorize an action,
    /// advance a document epoch, bind an origin, or grant reusable browser authority.
    pub fn require_registered_context_external_identifier(
        &self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        external_identifier: &str,
    ) -> Result<(), BrowserRegistryError> {
        self.require_context_external_identifier(
            browser_session,
            browsing_context,
            external_identifier,
        )
    }
}
