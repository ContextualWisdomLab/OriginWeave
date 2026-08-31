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
    /// The observation's registered session/context state could not be revalidated.
    RegistryState {
        /// Underlying browser-registry authority failure.
        source: BrowserRegistryError,
    },
    /// The current document epoch did not equal the caller-captured pre-action epoch.
    UnexpectedDocumentEpoch,
    /// The current document epoch has no representable successor.
    DocumentEpochExhausted,
}

impl fmt::Display for WebDriverBiDiNavigationCommittedDocumentAdvanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegistryState { .. } => formatter.write_str(
                "WebDriver BiDi navigation document advance cannot revalidate registered authority",
            ),
            Self::UnexpectedDocumentEpoch => formatter.write_str(
                "WebDriver BiDi navigation document advance does not match the expected pre-action document epoch",
            ),
            Self::DocumentEpochExhausted => formatter.write_str(
                "WebDriver BiDi navigation document advance exhausted the document epoch space",
            ),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedDocumentAdvanceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RegistryState { source } => Some(source),
            Self::UnexpectedDocumentEpoch | Self::DocumentEpochExhausted => None,
        }
    }
}

fn validate_document_epoch(
    current: DocumentEpoch,
    expected_previous: DocumentEpoch,
) -> Result<(), WebDriverBiDiNavigationCommittedDocumentAdvanceError> {
    if current != expected_previous {
        return Err(WebDriverBiDiNavigationCommittedDocumentAdvanceError::UnexpectedDocumentEpoch);
    }
    if current.value() == u64::MAX {
        return Err(WebDriverBiDiNavigationCommittedDocumentAdvanceError::DocumentEpochExhausted);
    }
    Ok(())
}

fn map_advance_document_error(
    source: BrowserRegistryError,
) -> WebDriverBiDiNavigationCommittedDocumentAdvanceError {
    if source == BrowserRegistryError::DocumentEpochExhausted {
        WebDriverBiDiNavigationCommittedDocumentAdvanceError::DocumentEpochExhausted
    } else {
        WebDriverBiDiNavigationCommittedDocumentAdvanceError::RegistryState { source }
    }
}

fn require_expected_document_epoch(
    registry: &BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    expected_previous: DocumentEpoch,
) -> Result<(), WebDriverBiDiNavigationCommittedDocumentAdvanceError> {
    let current = registry
        .current_context_epoch(browser_session, browsing_context)
        .map_err(
            |source| WebDriverBiDiNavigationCommittedDocumentAdvanceError::RegistryState { source },
        )?;
    validate_document_epoch(current, expected_previous)
}

/// Consume one exact accepted navigation observation and rotate that context's document authority.
///
/// The caller must supply the document epoch captured before dispatching the action whose
/// post-condition is being evaluated. The observation is consumed so one admitted event cannot be
/// reused to rotate the registry twice. The exact session/context pair is revalidated immediately
/// before mutation, and a stale expected epoch fails closed without changing registry state.
///
/// A successful advance delegates to [`BrowserAuthorityRegistry::advance_document`], which clears
/// the previous canonical-origin binding and all node bindings owned by the context. The new
/// document still has no origin binding until a separately trusted browser observation establishes
/// one through the canonical registry lifecycle. Any registry failure remains typed rather than
/// being converted into a panic or a successful authority transition.
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
    require_expected_document_epoch(
        registry,
        browser_session,
        browsing_context,
        expected_previous_epoch,
    )?;

    let current_epoch = registry
        .advance_document(browsing_context)
        .map_err(map_advance_document_error)?;

    Ok(WebDriverBiDiNavigationCommittedDocumentAdvance {
        browser_session,
        browsing_context,
        previous_epoch: expected_previous_epoch,
        current_epoch,
    })
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
    fn epoch_validation_distinguishes_stale_and_exhausted_state() {
        let one = document_epochs(1);
        let two = document_epochs(2);
        let maximum = document_epochs(u64::MAX);
        assert_eq!(one.len(), 1);
        assert_eq!(two.len(), 1);
        assert_eq!(maximum.len(), 1);

        assert!(validate_document_epoch(one[0], one[0]).is_ok());
        assert!(matches!(
            validate_document_epoch(one[0], two[0]),
            Err(WebDriverBiDiNavigationCommittedDocumentAdvanceError::UnexpectedDocumentEpoch)
        ));
        assert!(matches!(
            validate_document_epoch(maximum[0], maximum[0]),
            Err(WebDriverBiDiNavigationCommittedDocumentAdvanceError::DocumentEpochExhausted)
        ));
    }

    #[test]
    fn registry_errors_map_without_panicking_or_becoming_success() {
        assert!(matches!(
            map_advance_document_error(BrowserRegistryError::DocumentEpochExhausted),
            WebDriverBiDiNavigationCommittedDocumentAdvanceError::DocumentEpochExhausted
        ));
        assert!(matches!(
            map_advance_document_error(BrowserRegistryError::UnknownBrowsingContext),
            WebDriverBiDiNavigationCommittedDocumentAdvanceError::RegistryState {
                source: BrowserRegistryError::UnknownBrowsingContext
            }
        ));
    }

    #[test]
    fn registry_revalidation_and_public_diagnostics_fail_closed() {
        let registry = BrowserAuthorityRegistry::new();
        let session = browser_sessions(1);
        let context = browsing_contexts(1);
        let epoch = document_epochs(1);
        assert_eq!(session.len(), 1);
        assert_eq!(context.len(), 1);
        assert_eq!(epoch.len(), 1);

        let error = match require_expected_document_epoch(&registry, session[0], context[0], epoch[0]) {
            Err(error) => error,
            Ok(()) => return,
        };

        assert!(matches!(
            error,
            WebDriverBiDiNavigationCommittedDocumentAdvanceError::RegistryState { .. }
        ));
        assert_eq!(
            error.to_string(),
            "WebDriver BiDi navigation document advance cannot revalidate registered authority"
        );
        assert!(error.source().is_some());

        let stale = WebDriverBiDiNavigationCommittedDocumentAdvanceError::UnexpectedDocumentEpoch;
        assert_eq!(
            stale.to_string(),
            "WebDriver BiDi navigation document advance does not match the expected pre-action document epoch"
        );
        assert!(stale.source().is_none());

        let exhausted =
            WebDriverBiDiNavigationCommittedDocumentAdvanceError::DocumentEpochExhausted;
        assert_eq!(
            exhausted.to_string(),
            "WebDriver BiDi navigation document advance exhausted the document epoch space"
        );
        assert!(exhausted.source().is_none());
    }
}
