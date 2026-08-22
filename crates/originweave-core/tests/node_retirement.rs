use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, ObservedNodeHandle, Origin,
};

#[test]
fn same_document_node_retirement_revokes_authority_without_reusing_identity()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let origin = Origin::parse("http://127.0.0.1:43127")?;
    let live = registry.bind_node(session, context, &origin, "backend-node-17")?;

    let different_observation = ObservedNodeHandle::new(
        session,
        context,
        origin.clone(),
        live.document_epoch(),
        live.node_id() + 1,
    )?;
    assert_ne!(live, different_observation);

    registry.remove_node(&live)?;
    assert_eq!(
        registry.validate_node_handle(&live),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert_eq!(
        registry.remove_node(&live),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );

    let rebound = registry.bind_node(session, context, &origin, "backend-node-17")?;
    assert_ne!(live.node_id(), rebound.node_id());
    assert_eq!(registry.validate_node_handle(&rebound), Ok(()));
    Ok(())
}
