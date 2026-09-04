use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, ObservedNodeHandle, Origin,
};

#[test]
fn node_handles_cannot_cross_registry_instances_when_numeric_ids_collide()
-> Result<(), Box<dyn Error>> {
    let origin = Origin::parse("http://127.0.0.1:43127")?;

    let mut first_registry = BrowserAuthorityRegistry::new();
    let first_session = first_registry.register_session("first-session")?;
    let first_context = first_registry.register_context(first_session, "first-context")?;
    let first_handle =
        first_registry.bind_node(first_session, first_context, &origin, "first-node")?;

    let mut second_registry = BrowserAuthorityRegistry::new();
    let second_session = second_registry.register_session("second-session")?;
    let second_context = second_registry.register_context(second_session, "second-context")?;
    let second_handle =
        second_registry.bind_node(second_session, second_context, &origin, "second-node")?;
    let forged_matching = ObservedNodeHandle::new(
        second_session,
        second_context,
        origin.clone(),
        second_handle.document_epoch(),
        second_handle.node_id(),
    )?;

    assert_eq!(first_session, second_session);
    assert_eq!(first_context, second_context);
    assert_eq!(first_handle.node_id(), second_handle.node_id());
    assert_ne!(first_handle, second_handle);
    assert_eq!(
        second_registry.validate_node_handle(&first_handle),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert_eq!(
        second_registry.validate_node_handle(&forged_matching),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert_eq!(second_registry.validate_node_handle(&second_handle), Ok(()));
    Ok(())
}

#[test]
fn unissued_handles_do_not_reveal_registered_session_membership() -> Result<(), Box<dyn Error>> {
    let origin = Origin::parse("http://127.0.0.1:43128")?;

    let mut issuing_registry = BrowserAuthorityRegistry::new();
    let known_numeric_session = issuing_registry.register_session("issuing-session")?;
    let unknown_numeric_session = issuing_registry.register_session("issuing-extra-session")?;

    let mut target_registry = BrowserAuthorityRegistry::new();
    let target_session = target_registry.register_session("target-session")?;
    let target_context = target_registry.register_context(target_session, "target-context")?;
    let target_handle =
        target_registry.bind_node(target_session, target_context, &origin, "target-node")?;

    assert_eq!(known_numeric_session, target_session);
    assert_ne!(unknown_numeric_session, target_session);

    let forged_known_session = ObservedNodeHandle::new(
        known_numeric_session,
        target_context,
        origin.clone(),
        target_handle.document_epoch(),
        target_handle.node_id(),
    )?;
    let forged_unknown_session = ObservedNodeHandle::new(
        unknown_numeric_session,
        target_context,
        origin,
        target_handle.document_epoch(),
        target_handle.node_id(),
    )?;

    assert_eq!(
        target_registry.validate_node_handle(&forged_known_session),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert_eq!(
        target_registry.validate_node_handle(&forged_unknown_session),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    Ok(())
}
