use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, BrowsingContextId,
    DocumentEpoch, Origin,
};

fn first_origin() -> Result<Origin, Box<dyn Error>> {
    Ok(Origin::parse("http://127.0.0.1:43127")?)
}

fn second_origin() -> Result<Origin, Box<dyn Error>> {
    Ok(Origin::parse("http://localhost:43127")?)
}

#[test]
fn context_origin_can_be_bound_before_node_discovery() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let origin = first_origin()?;

    let epoch = registry.bind_context_origin(session, context, &origin)?;
    assert_eq!(epoch, DocumentEpoch::new(1)?);
    assert_eq!(
        registry.bind_context_origin(session, context, &origin)?,
        epoch
    );

    let node = registry.bind_node(session, context, &origin, "backend-node-17")?;
    assert_eq!(node.document_epoch(), epoch);
    assert_eq!(node.origin(), &origin);
    Ok(())
}

#[test]
fn context_origin_change_requires_document_rotation() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let first = first_origin()?;
    let second = second_origin()?;

    registry.bind_context_origin(session, context, &first)?;
    assert_eq!(
        registry.bind_context_origin(session, context, &second),
        Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance)
    );

    let next_epoch = registry.advance_document(context)?;
    assert_eq!(next_epoch, DocumentEpoch::new(2)?);
    assert_eq!(
        registry.bind_context_origin(session, context, &second)?,
        next_epoch
    );
    Ok(())
}

#[test]
fn context_origin_binding_rejects_cross_session_and_unknown_authority() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let owner = registry.register_session("owner-session")?;
    let attacker = registry.register_session("attacker-session")?;
    let context = registry.register_context(owner, "top-level-context")?;
    let origin = first_origin()?;

    assert_eq!(
        registry.bind_context_origin(attacker, context, &origin),
        Err(BrowserRegistryError::ContextSessionMismatch {
            expected: owner,
            actual: attacker,
        })
    );

    let unknown_session = BrowserSessionId::new(999)?;
    assert_eq!(
        registry.bind_context_origin(unknown_session, context, &origin),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );

    let unknown_context = BrowsingContextId::new(999)?;
    assert_eq!(
        registry.bind_context_origin(owner, unknown_context, &origin),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
    Ok(())
}
