use std::error::Error;

use crate::{BrowserAuthorityRegistry, Origin};

#[test]
fn repeated_node_binding_exercises_the_unit_crate_existing_node_path() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("unit-session")?;
    let context = registry.register_context(session, "unit-context")?;
    let origins: Vec<_> = Origin::parse("http://127.0.0.1:43127")?
        .into_iter()
        .collect();
    assert_eq!(origins.len(), 1);
    let origin = &origins[0];

    let first = registry.bind_node(session, context, origin, "unit-node")?;
    let repeated = registry.bind_node(session, context, origin, "unit-node")?;

    assert_eq!(first, repeated);
    Ok(())
}
