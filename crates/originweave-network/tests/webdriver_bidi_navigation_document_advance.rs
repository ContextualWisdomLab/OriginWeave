mod support;

use std::{error::Error, io};

use originweave_core::{BrowserAuthorityRegistry, BrowserRegistryError, Origin};
use originweave_network::{
    WebDriverBiDiNavigationCommittedDocumentAdvanceError,
    advance_webdriver_bidi_navigation_document_epoch,
};
use support::receive_subscribed_navigation_event;

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CONTEXT_ID: &str = "context-a";
const EXPECTED_URL: &str = "https://example.test/after";

#[test]
fn accepted_navigation_advances_only_the_exact_pre_action_document_epoch()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let before = registry.current_context_epoch(session, context)?;
    let previous_origin = Origin::parse("https://example.test")
        .map_err(|error| io::Error::other(format!("fixture origin parse failed: {error:?}")))?;
    registry.bind_context_origin(session, context, &previous_origin)?;

    let observation = receive_subscribed_navigation_event(
        &registry,
        session,
        context,
        SESSION_ID,
        CONTEXT_ID,
        EXPECTED_URL,
    )?;
    let advance =
        advance_webdriver_bidi_navigation_document_epoch(observation, &mut registry, before)?;

    assert_eq!(advance.browser_session(), session);
    assert_eq!(advance.browsing_context(), context);
    assert_eq!(advance.previous_epoch(), before);
    assert_eq!(
        advance.current_epoch(),
        registry.current_context_epoch(session, context)?
    );
    assert_ne!(advance.current_epoch(), before);
    assert_eq!(
        registry.require_context_origin(session, context, &previous_origin),
        Err(BrowserRegistryError::ContextOriginNotBound)
    );

    let replay = receive_subscribed_navigation_event(
        &registry,
        session,
        context,
        SESSION_ID,
        CONTEXT_ID,
        EXPECTED_URL,
    )?;
    let replay_error =
        advance_webdriver_bidi_navigation_document_epoch(replay, &mut registry, before)
            .err()
            .ok_or_else(|| io::Error::other("stale navigation unexpectedly advanced again"))?;
    assert!(matches!(
        replay_error,
        WebDriverBiDiNavigationCommittedDocumentAdvanceError::UnexpectedDocumentEpoch
    ));
    assert_eq!(
        replay_error.to_string(),
        "WebDriver BiDi navigation document advance does not match the expected pre-action document epoch"
    );
    assert!(replay_error.source().is_none());
    assert_eq!(
        registry.current_context_epoch(session, context)?,
        advance.current_epoch()
    );
    Ok(())
}

#[test]
fn retired_context_between_observation_and_advance_fails_closed_with_typed_source()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let before = registry.current_context_epoch(session, context)?;
    let observation = receive_subscribed_navigation_event(
        &registry,
        session,
        context,
        SESSION_ID,
        CONTEXT_ID,
        EXPECTED_URL,
    )?;

    registry.remove_context(context)?;
    let error =
        advance_webdriver_bidi_navigation_document_epoch(observation, &mut registry, before)
            .err()
            .ok_or_else(|| io::Error::other("retired context unexpectedly advanced"))?;
    assert!(matches!(
        error,
        WebDriverBiDiNavigationCommittedDocumentAdvanceError::RegistryState { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi navigation document advance cannot transition registered authority"
    );
    let source = error
        .source()
        .and_then(|source| source.downcast_ref::<BrowserRegistryError>())
        .ok_or_else(|| io::Error::other("registry failure source was not preserved"))?;
    assert_eq!(source, &BrowserRegistryError::UnknownBrowsingContext);
    assert_eq!(
        registry.current_context_epoch(session, context),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
    Ok(())
}
