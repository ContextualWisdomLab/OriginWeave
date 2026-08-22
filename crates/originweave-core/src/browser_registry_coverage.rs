use crate::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, DocumentEpoch,
    ObservedNodeHandle, Origin,
};

fn values<T, E>(result: Result<T, E>) -> Vec<T> {
    result.into_iter().collect()
}

#[test]
fn repeated_node_binding_exercises_the_unit_crate_existing_node_path() {
    let mut registry = BrowserAuthorityRegistry::new();
    let sessions = values(registry.register_session("unit-session"));
    assert_eq!(sessions.len(), 1);
    let session = sessions[0];
    let repeated_sessions = values(registry.register_session("unit-session"));
    assert_eq!(repeated_sessions, sessions);

    let contexts = values(registry.register_context(session, "unit-context"));
    assert_eq!(contexts.len(), 1);
    let context = contexts[0];

    let origins = values(Origin::parse("http://127.0.0.1:43127"));
    assert_eq!(origins.len(), 1);
    let origin = &origins[0];

    let first = values(registry.bind_node(session, context, origin, "unit-node"));
    let repeated = values(registry.bind_node(session, context, origin, "unit-node"));
    assert_eq!(first.len(), 1);
    assert_eq!(repeated.len(), 1);
    assert_eq!(first[0], repeated[0]);
    assert_eq!(registry.validate_node_handle(&first[0]), Ok(()));

    let epochs = values(DocumentEpoch::new(1));
    assert_eq!(epochs.len(), 1);
    let forged = values(ObservedNodeHandle::new(
        session,
        context,
        origin.clone(),
        epochs[0],
        first[0].node_id() + 1,
    ));
    assert_eq!(forged.len(), 1);
    assert_eq!(
        registry.validate_node_handle(&forged[0]),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );

    let mismatched_origins = values(Origin::parse("http://localhost:43127"));
    assert_eq!(mismatched_origins.len(), 1);
    let mismatched = values(ObservedNodeHandle::new(
        session,
        context,
        mismatched_origins[0].clone(),
        epochs[0],
        first[0].node_id(),
    ));
    assert_eq!(mismatched.len(), 1);
    assert_eq!(
        registry.validate_node_handle(&mismatched[0]),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
}

#[test]
fn node_validation_rejects_each_missing_authority_boundary() {
    let mut registry = BrowserAuthorityRegistry::new();
    assert_eq!(
        registry.register_session(""),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    let known_sessions = values(registry.register_session("validation-session"));
    let attacker_sessions = values(registry.register_session("validation-attacker"));
    assert_eq!(known_sessions.len(), 1);
    assert_eq!(attacker_sessions.len(), 1);
    let known = known_sessions[0];
    let attacker = attacker_sessions[0];
    let contexts = values(registry.register_context(known, "validation-context"));
    assert_eq!(contexts.len(), 1);
    let context = contexts[0];
    let origins = values(Origin::parse("http://127.0.0.1:43127"));
    assert_eq!(origins.len(), 1);
    let origin = origins[0].clone();
    let epochs = values(registry.current_epoch(context));
    assert_eq!(epochs.len(), 1);
    let epoch = epochs[0];

    let unknown_sessions = values(BrowserSessionId::new(999));
    assert_eq!(unknown_sessions.len(), 1);
    let unknown_handle = values(ObservedNodeHandle::new(
        unknown_sessions[0],
        context,
        origin.clone(),
        epoch,
        1,
    ));
    assert_eq!(unknown_handle.len(), 1);
    assert_eq!(
        registry.validate_node_handle(&unknown_handle[0]),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );

    let mismatched_handle = values(ObservedNodeHandle::new(
        attacker,
        context,
        origin.clone(),
        epoch,
        1,
    ));
    assert_eq!(mismatched_handle.len(), 1);
    assert_eq!(
        registry.validate_node_handle(&mismatched_handle[0]),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );

    let unbound_handle = values(ObservedNodeHandle::new(
        known,
        context,
        origin.clone(),
        epoch,
        1,
    ));
    assert_eq!(unbound_handle.len(), 1);
    assert_eq!(
        registry.validate_node_handle(&unbound_handle[0]),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );

    assert_eq!(registry.remove_context(context), Ok(()));
    assert_eq!(
        registry.validate_node_handle(&unbound_handle[0]),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
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
        registry.bind_node(attacker, context, &origins[0], "unit-node"),
        Err(BrowserRegistryError::ContextSessionMismatch {
            expected: owner,
            actual: attacker,
        })
    );
}

#[test]
fn session_retirement_covers_unit_success_and_unknown_paths() {
    let mut registry = BrowserAuthorityRegistry::new();
    let sessions = values(registry.register_session("retirement-unit-session"));
    assert_eq!(sessions.len(), 1);
    let session = sessions[0];
    let contexts = values(registry.register_context(session, "retirement-unit-context"));
    assert_eq!(contexts.len(), 1);
    let context = contexts[0];

    assert_eq!(registry.remove_session(session), Ok(()));
    assert_eq!(
        registry.current_epoch(context),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
    assert_eq!(
        registry.remove_session(session),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );
}
