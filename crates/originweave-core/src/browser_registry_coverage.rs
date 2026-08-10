use std::error::Error;

use crate::{BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, Origin};

#[test]
fn repeated_node_binding_exercises_the_unit_crate_existing_node_path() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("unit-session")?;
    let context = registry.register_context(session, "unit-context")?;
    let origins: Vec<_> = Origin::parse("http://127.0.0.1:43127")
        .into_iter()
        .collect();
    assert_eq!(origins.len(), 1);
    let origin = &origins[0];

    let first = registry.bind_node(session, context, origin, "unit-node")?;
    let repeated = registry.bind_node(session, context, origin, "unit-node")?;

    assert_eq!(first, repeated);
    Ok(())
}

#[test]
fn session_authority_failures_are_exercised_in_the_unit_crate() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let unknown = BrowserSessionId::new(999)?;
    assert_eq!(
        registry.register_context(unknown, "unknown-context"),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );

    let owner = registry.register_session("owner-session")?;
    let attacker = registry.register_session("attacker-session")?;
    let context = registry.register_context(owner, "owner-context")?;
    let origins: Vec<_> = Origin::parse("http://127.0.0.1:43127")
        .into_iter()
        .collect();
    assert_eq!(origins.len(), 1);

    assert_eq!(
        registry.bind_node(attacker, context, &origins[0], "unit-node"),
        Err(BrowserRegistryError::ContextSessionMismatch {
            expected: owner,
            actual: attacker,
        })
    );
    Ok(())
}
