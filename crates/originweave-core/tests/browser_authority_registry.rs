#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, DocumentEpoch,
    NodeHandleError, Origin,
};

fn loopback_origin() -> Origin {
    Origin::parse("http://127.0.0.1:43127").expect("valid loopback fixture origin")
}

#[test]
fn external_protocol_identifiers_are_scoped_and_never_become_authority()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();

    let first_session = registry.register_session("webdriver-session-A")?;
    let repeated_session = registry.register_session("webdriver-session-A")?;
    let second_session = registry.register_session("webdriver-session-B")?;

    assert_eq!(first_session, repeated_session);
    assert_ne!(first_session, second_session);

    let first_context = registry.register_context(first_session, "frame-root")?;
    let repeated_context = registry.register_context(first_session, "frame-root")?;
    let second_context = registry.register_context(second_session, "frame-root")?;

    assert_eq!(first_context, repeated_context);
    assert_ne!(first_context, second_context);
    assert_eq!(
        registry.current_epoch(first_context)?,
        DocumentEpoch::new(1)?
    );
    Ok(())
}

#[test]
fn document_rotation_invalidates_old_external_node_bindings() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let origin = loopback_origin();

    let first = registry.bind_node(session, context, &origin, "backend-node-17")?;
    let same = registry.bind_node(session, context, &origin, "backend-node-17")?;
    assert_eq!(first.node_id(), same.node_id());

    let next_epoch = registry.advance_document(context)?;
    assert_eq!(next_epoch.value(), 2);
    assert_eq!(
        first.validate_current(session, context, &origin, next_epoch),
        Err(NodeHandleError::StaleDocumentEpoch {
            observed: first.document_epoch(),
            current: next_epoch,
        })
    );

    let rebound = registry.bind_node(session, context, &origin, "backend-node-17")?;
    assert_eq!(rebound.document_epoch(), next_epoch);
    assert_ne!(first.node_id(), rebound.node_id());
    Ok(())
}

#[test]
fn context_cannot_be_reused_by_another_session() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let owner = registry.register_session("owner-session")?;
    let attacker = registry.register_session("attacker-session")?;
    let context = registry.register_context(owner, "shared-looking-context")?;
    let origin = loopback_origin();

    assert_eq!(
        registry.bind_node(attacker, context, &origin, "node"),
        Err(BrowserRegistryError::ContextSessionMismatch {
            expected: owner,
            actual: attacker,
        })
    );
    Ok(())
}

#[test]
fn context_origin_cannot_change_without_document_rotation() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let first_origin = loopback_origin();
    let second_origin = Origin::parse("http://localhost:43127").expect("valid loopback fixture origin");

    registry.bind_node(session, context, &first_origin, "backend-node-17")?;
    assert_eq!(
        registry.bind_node(session, context, &second_origin, "backend-node-18"),
        Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance)
    );
    Ok(())
}

#[test]
fn external_identifiers_are_bounded_without_assuming_protocol_syntax() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();

    assert_eq!(
        registry.register_session(""),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    assert_eq!(
        registry.register_session(&"x".repeat(513)),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );

    let unicode = registry.register_session("세션-opaque-✓")?;
    assert!(unicode.value() > 0);
    Ok(())
}

#[test]
fn authority_identifier_capacity_is_bounded_and_testable() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::with_identifier_limit(1);
    let session = registry.register_session("session-one")?;
    assert_eq!(
        registry.register_session("session-two"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );

    let context = registry.register_context(session, "context-one")?;
    assert_eq!(
        registry.register_context(session, "context-two"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );

    let origin = loopback_origin();
    assert!(
        registry
            .bind_node(session, context, &origin, "node-one")
            .is_ok()
    );
    assert_eq!(
        registry.bind_node(session, context, &origin, "node-two"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );
    Ok(())
}

#[test]
fn unknown_internal_authority_is_rejected_before_node_binding() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let unknown = BrowserSessionId::new(999)?;

    assert_eq!(
        registry.register_context(unknown, "context"),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );

    let known = registry.register_session("known-session")?;
    let context = registry.register_context(known, "known-context")?;
    let origin = loopback_origin();
    assert_eq!(
        registry.bind_node(unknown, context, &origin, "node"),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );
    Ok(())
}
