use std::{error::Error, fmt};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, BrowsingContextId,
    DocumentEpoch, Origin,
};

use crate::{
    WebDriverBiDiNavigationCommittedDocumentAdvanceError,
    WebDriverBiDiNavigationCommittedObservation, advance_webdriver_bidi_navigation_document_epoch,
};

/// Immutable evidence that one accepted navigation rotated its document and bound its observed origin.
///
/// The value records only the registry-local session/context/document transition and the canonical
/// origin derived from the exact serialized URL carried by the accepted WebDriver BiDi navigation
/// observation. It does not authenticate the browser adapter, prove which action caused the
/// navigation, authorize the destination, or grant browser, policy, node, credential, process, or
/// reusable Agent authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBiDiNavigationCommittedDocumentOrigin {
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    previous_epoch: DocumentEpoch,
    current_epoch: DocumentEpoch,
    origin: Origin,
}

impl WebDriverBiDiNavigationCommittedDocumentOrigin {
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

    /// Return the caller-captured document epoch that was current before the navigation advance.
    #[must_use]
    pub const fn previous_epoch(&self) -> DocumentEpoch {
        self.previous_epoch
    }

    /// Return the new document epoch to which the observed canonical origin was bound.
    #[must_use]
    pub const fn current_epoch(&self) -> DocumentEpoch {
        self.current_epoch
    }

    /// Borrow the canonical OriginWeave origin derived from the accepted serialized navigation URL.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }
}

/// Fail-closed failures while rotating document authority and binding the observed navigation origin.
#[derive(Debug)]
pub enum WebDriverBiDiNavigationCommittedDocumentOriginError {
    /// The accepted serialized navigation URL could not enter canonical OriginWeave origin authority.
    InvalidObservedOrigin,
    /// The accepted navigation could not rotate the exact caller-captured document epoch.
    DocumentAdvance {
        /// Underlying typed document-advance failure.
        source: WebDriverBiDiNavigationCommittedDocumentAdvanceError,
    },
    /// The canonical observed origin could not bind to the newly advanced registered document.
    RegistryState {
        /// Underlying browser-registry authority failure.
        source: BrowserRegistryError,
    },
}

impl fmt::Display for WebDriverBiDiNavigationCommittedDocumentOriginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObservedOrigin => formatter.write_str(
                "WebDriver BiDi committed navigation URL cannot enter canonical origin authority",
            ),
            Self::DocumentAdvance { .. } => formatter.write_str(
                "WebDriver BiDi committed navigation cannot rotate registered document authority",
            ),
            Self::RegistryState { .. } => formatter.write_str(
                "WebDriver BiDi committed navigation origin cannot bind registered document authority",
            ),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedDocumentOriginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidObservedOrigin => None,
            Self::DocumentAdvance { source } => Some(source),
            Self::RegistryState { source } => Some(source),
        }
    }
}

fn origin_from_serialized_navigation_url(serialized_url: &str) -> Option<Origin> {
    let (scheme, remainder) = serialized_url.split_once("://")?;
    let authority_end = remainder
        .find(|character| matches!(character, '/' | '?' | '#'))
        .unwrap_or(remainder.len());
    let authority = &remainder[..authority_end];
    Origin::parse(&format!("{scheme}://{authority}")).ok()
}

fn bind_advanced_document_origin(
    registry: &mut BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    origin: &Origin,
) -> Result<DocumentEpoch, WebDriverBiDiNavigationCommittedDocumentOriginError> {
    registry
        .bind_context_origin(browser_session, browsing_context, origin)
        .map_err(
            |source| WebDriverBiDiNavigationCommittedDocumentOriginError::RegistryState { source },
        )
}

/// Consume one accepted navigation, rotate the exact expected document, and bind its canonical origin.
///
/// Origin derivation is completed before any registry mutation. Only serialized HTTP(S) URLs whose
/// authority can enter [`Origin`] are accepted; credential-bearing, opaque, malformed, insecure
/// remote HTTP, and otherwise unsupported authorities therefore fail before document rotation.
/// The accepted observation is then consumed by the existing exact-epoch document-advance boundary,
/// which clears stale origin and node authority. Finally, the derived canonical origin is bound to
/// that newly advanced document through the canonical browser registry lifecycle.
///
/// The caller must still treat the returned value as immediate-use registry evidence rather than as
/// proof of action causality, browser authenticity, destination authorization, or reusable authority.
pub fn advance_and_bind_webdriver_bidi_navigation_document_origin(
    observation: WebDriverBiDiNavigationCommittedObservation,
    registry: &mut BrowserAuthorityRegistry,
    expected_previous_epoch: DocumentEpoch,
) -> Result<
    WebDriverBiDiNavigationCommittedDocumentOrigin,
    WebDriverBiDiNavigationCommittedDocumentOriginError,
> {
    let origin = origin_from_serialized_navigation_url(observation.url())
        .ok_or(WebDriverBiDiNavigationCommittedDocumentOriginError::InvalidObservedOrigin)?;
    let advance = advance_webdriver_bidi_navigation_document_epoch(
        observation,
        registry,
        expected_previous_epoch,
    )
    .map_err(|source| {
        WebDriverBiDiNavigationCommittedDocumentOriginError::DocumentAdvance { source }
    })?;
    bind_advanced_document_origin(
        registry,
        advance.browser_session(),
        advance.browsing_context(),
        &origin,
    )
    .map(
        |current_epoch| WebDriverBiDiNavigationCommittedDocumentOrigin {
            browser_session: advance.browser_session(),
            browsing_context: advance.browsing_context(),
            previous_epoch: advance.previous_epoch(),
            current_epoch,
            origin,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn browser_sessions(value: u64) -> Vec<BrowserSessionId> {
        BrowserSessionId::new(value).into_iter().collect()
    }

    fn browsing_contexts(value: u64) -> Vec<BrowsingContextId> {
        BrowsingContextId::new(value).into_iter().collect()
    }

    #[test]
    fn serialized_navigation_origin_is_canonical_and_rejects_non_authority_urls() {
        let canonical = origin_from_serialized_navigation_url(
            "HTTPS://EXAMPLE.TEST:443/path?query=value#fragment",
        );
        assert_eq!(
            canonical.as_ref().map(Origin::as_str),
            Some("https://example.test")
        );
        assert!(origin_from_serialized_navigation_url("https://user@example.test/path").is_none());
        assert!(origin_from_serialized_navigation_url("data:text/plain,originweave").is_none());
        assert!(origin_from_serialized_navigation_url("http://example.test/path").is_none());
        assert_eq!(
            origin_from_serialized_navigation_url("http://127.0.0.1:8080/path")
                .as_ref()
                .map(Origin::as_str),
            Some("http://127.0.0.1:8080")
        );
    }

    #[test]
    fn binding_helper_preserves_registry_failure_source() {
        let mut registry = BrowserAuthorityRegistry::new();
        let sessions = browser_sessions(1);
        let contexts = browsing_contexts(1);
        assert_eq!(sessions.len(), 1);
        assert_eq!(contexts.len(), 1);
        let origin = Origin::parse("https://example.test").expect("fixture origin is valid");
        let error = bind_advanced_document_origin(&mut registry, sessions[0], contexts[0], &origin)
            .expect_err("unknown registry session must fail closed");
        assert_eq!(
            error.to_string(),
            "WebDriver BiDi committed navigation origin cannot bind registered document authority"
        );
        assert_eq!(
            error
                .source()
                .and_then(|source| source.downcast_ref::<BrowserRegistryError>()),
            Some(&BrowserRegistryError::UnknownBrowserSession)
        );
    }

    #[test]
    fn public_diagnostics_preserve_only_causal_sources() {
        let invalid = WebDriverBiDiNavigationCommittedDocumentOriginError::InvalidObservedOrigin;
        assert_eq!(
            invalid.to_string(),
            "WebDriver BiDi committed navigation URL cannot enter canonical origin authority"
        );
        assert!(invalid.source().is_none());

        let advance = WebDriverBiDiNavigationCommittedDocumentOriginError::DocumentAdvance {
            source: WebDriverBiDiNavigationCommittedDocumentAdvanceError::UnexpectedDocumentEpoch,
        };
        assert_eq!(
            advance.to_string(),
            "WebDriver BiDi committed navigation cannot rotate registered document authority"
        );
        assert!(advance.source().is_some());
    }
}
