use crate::{BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, Origin};

fn values<T, E>(result: Result<T, E>) -> Vec<T> {
    result.into_iter().collect()
}

#[test]
fn repeated_node_binding_exercises_the_unit_crate_existing_node_path() {
    let mut registry = BrowserAuthorityRegistry::new();
    let sessions = values(registry.register_session("unit-session"));
    assert_eq!(sessions.len(), 1);
    let session = sessions[0];

    let contexts = values(registry.register_context(session, "unit-context"));
    assert_eq!(contexts.len(), 1);
    let context = contexts[0];

    let origins = values(Origin::parse("http://127.0.0.1:43127"));
    assert_eq!(origins.len(), 1);
    let origin = &origins[0];

    let first = values(registry.bind_nodes(session, context, origin, &["unit-node"]));
    let repeated = values(registry.bind_nodes(session, context, origin, &["unit-node"]));
    assert_eq!(first.len(), 1);
    assert_eq!(repeated.len(), 1);
    assert_eq!(first[0], repeated[0]);
}

#[test]
fn session_authority_failures_are_exercised_in_the_unit_crate() {
    let mut registry = BrowserAuthorityRegistry::new();
    let unknown_sessions = values(BrowserSessionId::new(999));
    assert_eq!(unknown_sessions.len(), 1);
    let unknown = unknown_sessions[0];
    assert_eq!(
        registry.register_context(unknown, "unknown-context"),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );

    let owner_sessions = values(registry.register_session("owner-session"));
    let attacker_sessions = values(registry.register_session("attacker-session"));
    assert_eq!(owner_sessions.len(), 1);
    assert_eq!(attacker_sessions.len(), 1);
    let owner = owner_sessions[0];
    let attacker = attacker_sessions[0];

    let contexts = values(registry.register_context(owner, "owner-context"));
    let origins = values(Origin::parse("http://127.0.0.1:43127"));
    assert_eq!(contexts.len(), 1);
    assert_eq!(origins.len(), 1);
    let context = contexts[0];

    assert_eq!(
        registry.bind_nodes(attacker, context, &origins[0], &["unit-node"]),
        Err(BrowserRegistryError::ContextSessionMismatch {
            expected: owner,
            actual: attacker,
        })
    );
}
