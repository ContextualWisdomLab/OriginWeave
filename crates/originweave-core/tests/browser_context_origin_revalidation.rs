use std::error::Error;
use std::io;

use originweave_core::{BrowserAuthorityRegistry, BrowserRegistryError, Origin};

fn first_origin() -> Result<Origin, io::Error> {
    Origin::parse("http://127.0.0.1:43127")
        .map_err(|_error| io::Error::other("controlled first origin must be valid"))
}

fn second_origin() -> Result<Origin, io::Error> {
    Origin::parse("http://localhost:43127")
        .map_err(|_error| io::Error::other("controlled second origin must be valid"))
}

#[test]
fn current_context_origin_must_be_bound_before_revalidation() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let origin = first_origin()?;

    assert_eq!(
        registry.require_context_origin(session, context, &origin),
        Err(BrowserRegistryError::ContextOriginNotBound)
    );

    let epoch = registry.bind_context_origin(session, context, &origin)?;
    assert_eq!(
        registry.require_context_origin(session, context, &origin),
        Ok(epoch)
    );
    Ok(())
}

#[test]
fn current_context_origin_revalidation_fails_closed_on_mismatch() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let first = first_origin()?;
    let second = second_origin()?;

    registry.bind_context_origin(session, context, &first)?;
    assert_eq!(
        registry.require_context_origin(session, context, &second),
        Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance)
    );
    assert!(
        registry
            .require_context_origin(session, context, &first)
            .is_ok()
    );
    Ok(())
}

#[test]
fn document_rotation_requires_fresh_origin_binding() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let first = first_origin()?;
    let second = second_origin()?;

    registry.bind_context_origin(session, context, &first)?;
    let next_epoch = registry.advance_document(context)?;
    assert_eq!(
        registry.require_context_origin(session, context, &first),
        Err(BrowserRegistryError::ContextOriginNotBound)
    );

    registry.bind_context_origin(session, context, &second)?;
    assert_eq!(
        registry.require_context_origin(session, context, &second),
        Ok(next_epoch)
    );
    Ok(())
}

#[test]
fn context_origin_revalidation_preserves_session_ownership() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let owner = registry.register_session("owner-session")?;
    let attacker = registry.register_session("attacker-session")?;
    let context = registry.register_context(owner, "top-level-context")?;
    let origin = first_origin()?;

    registry.bind_context_origin(owner, context, &origin)?;
    assert_eq!(
        registry.require_context_origin(attacker, context, &origin),
        Err(BrowserRegistryError::ContextSessionMismatch {
            expected: owner,
            actual: attacker,
        })
    );
    Ok(())
}
