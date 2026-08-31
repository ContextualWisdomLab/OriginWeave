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

fn require_expected_document_epoch(
    registry: &BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    expected_previous: DocumentEpoch,
) -> Result<(), WebDriverBiDiNavigationCommittedDocumentAdvanceError> {
    let current = registry
        .current_context_epoch(browser_session, browsing_context)
        .map_err(
            |source| WebDriverBiDiNavigationCommittedDocumentAdvanceError::RegistryState {
                source,
            },
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
/// one through the canonical registry lifecycle.
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

    // The exclusive mutable borrow prevents an intervening registry mutation. After the preceding
    // revalidation, the context exists, belongs to this session, and its epoch is below u64::MAX,
    // so every documented failure of `advance_document` has already been ruled out.
    let current_epoch = registry
        .advance_document(browsing_context)
        .expect("validated document epoch must have one representable registry successor");

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

    #[test]
    fn epoch_validation_distinguishes_stale_and_exhausted_state() {
        let one = DocumentEpoch::new(1).expect("one is a valid document epoch");
        let two = DocumentEpoch::new(2).expect("two is a valid document epoch");
        let maximum = DocumentEpoch::new(u64::MAX).expect("u64::MAX is a valid document epoch");

        assert!(validate_document_epoch(one, one).is_ok());
        assert!(matches!(
            validate_document_epoch(one, two),
            Err(WebDriverBiDiNavigationCommittedDocumentAdvanceError::UnexpectedDocumentEpoch)
        ));
        assert!(matches!(
            validate_document_epoch(maximum, maximum),
            Err(WebDriverBiDiNavigationCommittedDocumentAdvanceError::DocumentEpochExhausted)
        ));
    }

    #[test]
    fn registry_revalidation_and_public_diagnostics_fail_closed() {
        let registry = BrowserAuthorityRegistry::new();
        let session = BrowserSessionId::new(1).expect("one is a valid browser session id");
        let context = BrowsingContextId::new(1).expect("one is a valid browsing context id");
        let epoch = DocumentEpoch::new(1).expect("one is a valid document epoch");
        let error = require_expected_document_epoch(&registry, session, context, epoch)
            .expect_err("unregistered authority must fail closed");

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

        let exhausted = WebDriverBiDiNavigationCommittedDocumentAdvanceError::DocumentEpochExhausted;
        assert_eq!(
            exhausted.to_string(),
            "WebDriver BiDi navigation document advance exhausted the document epoch space"
        );
        assert!(exhausted.source().is_none());
    }
}
