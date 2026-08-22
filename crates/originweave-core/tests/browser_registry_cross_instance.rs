use std::error::Error;

use originweave_core::{BrowserAuthorityRegistry, BrowserRegistryError, Origin};

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

    assert_eq!(first_session, second_session);
    assert_eq!(first_context, second_context);
    assert_eq!(first_handle.node_id(), second_handle.node_id());
    assert_eq!(
        second_registry.validate_node_handle(&first_handle),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert_eq!(second_registry.validate_node_handle(&second_handle), Ok(()));
    Ok(())
}
