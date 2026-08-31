mod support;

use std::{error::Error, io};

use originweave_core::{BrowserAuthorityRegistry, Origin};
use originweave_network::{
    WebDriverBiDiNavigationCommittedDocumentAdvanceError,
    advance_and_bind_webdriver_bidi_navigation_document_origin,
};
use support::receive_subscribed_navigation_event;

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const CONTEXT_ID: &str = "context-a";

fn fixture_origin(value: &str) -> Result<Origin, Box<dyn Error>> {
    Origin::parse(value)
        .map_err(|error| io::Error::other(format!("fixture origin parse failed: {error:?}")).into())
}

#[test]
fn committed_navigation_rotates_document_and_binds_canonical_observed_origin()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let before = registry.current_context_epoch(session, context)?;
    let previous_origin = fixture_origin("https://before.example")?;
    registry.bind_context_origin(session, context, &previous_origin)?;

    let observed_url = "https://EXAMPLE.TEST:443/after?from=originweave#done";
    let observation = receive_subscribed_navigation_event(
        &registry,
        session,
        context,
        SESSION_ID,
        CONTEXT_ID,
        observed_url,
    )?;
    let binding = advance_and_bind_webdriver_bidi_navigation_document_origin(
        observation,
        &mut registry,
        before,
    )?;

    let expected_origin = fixture_origin("https://example.test")?;
    assert_eq!(binding.browser_session(), session);
    assert_eq!(binding.browsing_context(), context);
    assert_eq!(binding.previous_epoch(), before);
    assert_ne!(binding.current_epoch(), before);
    assert_eq!(binding.origin(), &expected_origin);
    assert_eq!(
        registry.require_context_origin(session, context, &expected_origin)?,
        binding.current_epoch()
    );
    Ok(())
}

#[test]
fn invalid_observed_origin_fails_before_document_authority_is_rotated() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let before = registry.current_context_epoch(session, context)?;
    let previous_origin = fixture_origin("https://before.example")?;
    registry.bind_context_origin(session, context, &previous_origin)?;

    let observed_url = "https://user@example.test/after";
    let observation = receive_subscribed_navigation_event(
        &registry,
        session,
        context,
        SESSION_ID,
        CONTEXT_ID,
        observed_url,
    )?;
    let error = advance_and_bind_webdriver_bidi_navigation_document_origin(
        observation,
        &mut registry,
        before,
    )
    .err()
    .ok_or_else(|| io::Error::other("credential-bearing observed URL unexpectedly bound origin"))?;

    assert_eq!(
        error.to_string(),
        "WebDriver BiDi committed navigation URL cannot enter canonical origin authority"
    );
    assert!(error.source().is_none());
    assert_eq!(registry.current_context_epoch(session, context)?, before);
    assert_eq!(
        registry.require_context_origin(session, context, &previous_origin)?,
        before
    );
    Ok(())
}

#[test]
fn stale_pre_action_epoch_fails_before_observed_origin_is_bound() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session(SESSION_ID)?;
    let context = registry.register_context(session, CONTEXT_ID)?;
    let before = registry.current_context_epoch(session, context)?;
    let previous_origin = fixture_origin("https://before.example")?;
    registry.bind_context_origin(session, context, &previous_origin)?;

    let observed_url = "https://example.test/after";
    let observation = receive_subscribed_navigation_event(
        &registry,
        session,
        context,
        SESSION_ID,
        CONTEXT_ID,
        observed_url,
    )?;

    let intervening_epoch = registry.advance_document(context)?;
    let intervening_origin = fixture_origin("https://intervening.example")?;
    registry.bind_context_origin(session, context, &intervening_origin)?;

    let error = advance_and_bind_webdriver_bidi_navigation_document_origin(
        observation,
        &mut registry,
        before,
    )
    .err()
    .ok_or_else(|| io::Error::other("stale pre-action epoch unexpectedly advanced document"))?;

    assert_eq!(
        error.to_string(),
        "WebDriver BiDi committed navigation cannot rotate registered document authority"
    );
    assert_eq!(
        error
            .source()
            .and_then(|source| {
                source.downcast_ref::<WebDriverBiDiNavigationCommittedDocumentAdvanceError>()
            })
            .map(ToString::to_string)
            .as_deref(),
        Some(
            "WebDriver BiDi navigation document advance does not match the expected pre-action document epoch"
        )
    );
    assert_eq!(
        registry.current_context_epoch(session, context)?,
        intervening_epoch
    );
    assert_eq!(
        registry.require_context_origin(session, context, &intervening_origin)?,
        intervening_epoch
    );
    Ok(())
}
