use std::{error::Error, fmt};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, BrowsingContextId,
    DocumentEpoch,
};

use crate::WebDriverBiDiNavigationCommittedObservation;

/// Immutable evidence that one accepted navigation observation rotated one exact document epoch.
///
/// The value records only the registry-local session/context transition. It does not bind the new
/// document origin, authenticate the browser adapter, prove which action caused the navigation, or
/// grant browser, policy, node, destination, credential, process, or reusable Agent authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebDriverBiDiNavigationCommittedDocumentAdvance {
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    previous_epoch: DocumentEpoch,
    current_epoch: DocumentEpoch,
}

impl WebDriverBiDiNavigationCommittedDocumentAdvance {
    /// Return the exact OriginWeave browser session whose context advanced.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the exact OriginWeave browsing context whose document advanced.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.browsing_context
    }

    /// Return the caller-captured document epoch that was current immediately before the advance.
    #[must_use]
    pub const fn previous_epoch(&self) -> DocumentEpoch {
        self.previous_epoch
    }

    /// Return the new registry-local document epoch after stale node/origin authority was cleared.
    #[must_use]
    pub const fn current_epoch(&self) -> DocumentEpoch {
        self.current_epoch
    }
}

/// Fail-closed failures while rotating document authority from an accepted navigation observation.
#[derive(Debug)]
pub enum WebDriverBiDiNavigationCommittedDocumentAdvanceError {
    /// Registered session/context state could not be revalidated or advanced.
    RegistryState {
        /// Underlying browser-registry authority failure.
        source: BrowserRegistryError,
    },
    /// The current document epoch did not equal the caller-captured pre-action epoch.
    UnexpectedDocumentEpoch,
}

impl fmt::Display for WebDriverBiDiNavigationCommittedDocumentAdvanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegistryState { .. } => formatter.write_str(
                "WebDriver BiDi navigation document advance cannot transition registered authority",
            ),
            Self::UnexpectedDocumentEpoch => formatter.write_str(
                "WebDriver BiDi navigation document advance does not match the expected pre-action document epoch",
            ),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedDocumentAdvanceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RegistryState { source } => Some(source),
            Self::UnexpectedDocumentEpoch => None,
        }
    }
}

fn advance_registered_document_if_expected(
    registry: &mut BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    expected_previous: DocumentEpoch,
) -> Result<Option<DocumentEpoch>, BrowserRegistryError> {
    let current = registry.current_context_epoch(browser_session, browsing_context)?;
    if current != expected_previous {
        return Ok(None);
    }
    registry.advance_document(browsing_context).map(Some)
}

/// Consume one exact accepted navigation observation and rotate that context's document authority.
///
/// The caller must supply the document epoch captured before dispatching the action whose
/// post-condition is being evaluated. The observation is consumed so one admitted event cannot be
/// reused to rotate the registry twice. The exact session/context pair and caller-captured epoch
/// are revalidated immediately before mutation, and stale state fails closed without mutation.
///
/// A successful advance delegates to [`BrowserAuthorityRegistry::advance_document`], which clears
/// the previous canonical-origin binding and all node bindings owned by the context. The new
/// document still has no origin binding until a separately trusted browser observation establishes
/// one through the canonical registry lifecycle. Registry failures, including document-epoch
/// exhaustion, remain available as the typed [`BrowserRegistryError`] source instead of being
/// converted into a panic or successful authority transition.
pub fn advance_webdriver_bidi_navigation_document_epoch(
    observation: WebDriverBiDiNavigationCommittedObservation,
    registry: &mut BrowserAuthorityRegistry,
    expected_previous_epoch: DocumentEpoch,
) -> Result<
    WebDriverBiDiNavigationCommittedDocumentAdvance,
    WebDriverBiDiNavigationCommittedDocumentAdvanceError,
> {
    let browser_session = observation.browser_session();
    let browsing_context = observation.browsing_context();
    match advance_registered_document_if_expected(
        registry,
        browser_session,
        browsing_context,
        expected_previous_epoch,
    ) {
        Ok(Some(current_epoch)) => Ok(WebDriverBiDiNavigationCommittedDocumentAdvance {
            browser_session,
            browsing_context,
            previous_epoch: expected_previous_epoch,
            current_epoch,
        }),
        Ok(None) => {
            Err(WebDriverBiDiNavigationCommittedDocumentAdvanceError::UnexpectedDocumentEpoch)
        }
        Err(source) => {
            Err(WebDriverBiDiNavigationCommittedDocumentAdvanceError::RegistryState { source })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document_epochs(value: u64) -> Vec<DocumentEpoch> {
        DocumentEpoch::new(value).into_iter().collect()
    }

    fn browser_sessions(value: u64) -> Vec<BrowserSessionId> {
        BrowserSessionId::new(value).into_iter().collect()
    }

    fn browsing_contexts(value: u64) -> Vec<BrowsingContextId> {
        BrowsingContextId::new(value).into_iter().collect()
    }

    #[test]
    fn registered_document_helper_distinguishes_missing_stale_and_current_state() {
        let mut registry = BrowserAuthorityRegistry::new();
        let synthetic_session = browser_sessions(1);
        let synthetic_context = browsing_contexts(1);
        let epoch_one = document_epochs(1);
        let epoch_two = document_epochs(2);
        assert_eq!(synthetic_session.len(), 1);
        assert_eq!(synthetic_context.len(), 1);
        assert_eq!(epoch_one.len(), 1);
        assert_eq!(epoch_two.len(), 1);

        assert_eq!(
            advance_registered_document_if_expected(
                &mut registry,
                synthetic_session[0],
                synthetic_context[0],
                epoch_one[0],
            ),
            Err(BrowserRegistryError::UnknownBrowserSession)
        );

        let session = registry.register_session("session-a");
        assert!(session.is_ok());
        let session = session.ok();
        assert!(session.is_some());
        let session = session.into_iter().collect::<Vec<_>>();
        assert_eq!(session.len(), 1);
        let context = registry.register_context(session[0], "context-a");
        assert!(context.is_ok());
        let context = context.ok();
        assert!(context.is_some());
        let context = context.into_iter().collect::<Vec<_>>();
        assert_eq!(context.len(), 1);

        assert_eq!(
            advance_registered_document_if_expected(
                &mut registry,
                session[0],
                context[0],
                epoch_two[0],
            ),
            Ok(None)
        );
        assert_eq!(
            advance_registered_document_if_expected(
                &mut registry,
                session[0],
                context[0],
                epoch_one[0],
            ),
            Ok(Some(epoch_two[0]))
        );
    }

    #[test]
    fn public_diagnostics_preserve_registry_sources() {
        let registry_error = WebDriverBiDiNavigationCommittedDocumentAdvanceError::RegistryState {
            source: BrowserRegistryError::DocumentEpochExhausted,
        };
        assert_eq!(
            registry_error.to_string(),
            "WebDriver BiDi navigation document advance cannot transition registered authority"
        );
        assert_eq!(
            registry_error
                .source()
                .and_then(|source| source.downcast_ref::<BrowserRegistryError>()),
            Some(&BrowserRegistryError::DocumentEpochExhausted)
        );

        let stale = WebDriverBiDiNavigationCommittedDocumentAdvanceError::UnexpectedDocumentEpoch;
        assert_eq!(
            stale.to_string(),
            "WebDriver BiDi navigation document advance does not match the expected pre-action document epoch"
        );
        assert!(stale.source().is_none());
    }
}
