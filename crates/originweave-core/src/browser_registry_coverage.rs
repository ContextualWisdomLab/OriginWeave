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
    let repeated_contexts = values(registry.register_context(session, "unit-context"));
    assert_eq!(repeated_contexts, contexts);
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
    let oversized_identifier = "x".repeat(513);
    assert_eq!(
        registry.register_session(&oversized_identifier),
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
fn direct_fail_closed_registry_paths_are_exercised_in_the_unit_crate() {
    let mut registry = BrowserAuthorityRegistry::new();
    let unknown_sessions = values(BrowserSessionId::new(999));
    assert_eq!(unknown_sessions.len(), 1);
    let unknown = unknown_sessions[0];

    let sessions = values(registry.register_session("direct-path-session"));
    assert_eq!(sessions.len(), 1);
    let session = sessions[0];
    let contexts = values(registry.register_context(session, "direct-path-context"));
    assert_eq!(contexts.len(), 1);
    let context = contexts[0];
    let origins = values(Origin::parse("http://127.0.0.1:43127"));
    let changed_origins = values(Origin::parse("http://localhost:43127"));
    assert_eq!(origins.len(), 1);
    assert_eq!(changed_origins.len(), 1);

    assert_eq!(
        registry.bind_node(unknown, context, &origins[0], "unknown-session-node"),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );
    assert_eq!(
        values(registry.bind_node(session, context, &origins[0], "live-node")).len(),
        1
    );
    assert_eq!(
        registry.bind_node(session, context, &changed_origins[0], "changed-origin-node"),
        Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance)
    );

    assert_eq!(registry.remove_context(context), Ok(()));
    assert_eq!(
        registry.remove_context(context),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
}

#[test]
fn unit_cfg_allocation_rotation_and_retirement_edges_are_exercised() {
    let mut registry = BrowserAuthorityRegistry::with_identifier_limit(1);
    let sessions = values(registry.register_session("capacity-session"));
    assert_eq!(sessions.len(), 1);
    let session = sessions[0];
    assert_eq!(
        registry.register_session("capacity-session-two"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );

    let contexts = values(registry.register_context(session, "capacity-context"));
    assert_eq!(contexts.len(), 1);
    let context = contexts[0];
    assert_eq!(
        registry.register_context(session, "capacity-context-two"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );

    let origins = values(Origin::parse("http://127.0.0.1:43127"));
    assert_eq!(origins.len(), 1);
    let origin = &origins[0];
    let handles = values(registry.bind_node(session, context, origin, "capacity-node"));
    assert_eq!(handles.len(), 1);
    let handle = &handles[0];
    assert_eq!(
        registry.bind_node(session, context, origin, "capacity-node-two"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );

    let forged = values(ObservedNodeHandle::new(
        session,
        context,
        origin.clone(),
        handle.document_epoch(),
        handle.node_id() + 1,
    ));
    assert_eq!(forged.len(), 1);
    assert_eq!(
        registry.remove_node(&forged[0]),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );

    let next_epochs = values(registry.advance_document(context));
    assert_eq!(next_epochs.len(), 1);
    assert_eq!(next_epochs[0].value(), 2);
    assert_eq!(registry.remove_context(context), Ok(()));
    assert_eq!(
        registry.advance_document(context),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
}

#[test]
fn unit_cfg_adapter_surface_exercises_accessors_equality_default_and_errors() {
    let mut registry = BrowserAuthorityRegistry::default();
    let sessions = values(registry.register_session("adapter-surface-session"));
    assert_eq!(sessions.len(), 1);
    let session = sessions[0];
    let contexts = values(registry.register_context(session, "adapter-surface-context"));
    assert_eq!(contexts.len(), 1);
    let context = contexts[0];
    let origins = values(Origin::parse("http://127.0.0.1:43127"));
    assert_eq!(origins.len(), 1);
    let origin = origins[0].clone();
    let handles = values(registry.bind_node(session, context, &origin, "adapter-surface-node"));
    assert_eq!(handles.len(), 1);
    let handle = &handles[0];

    assert_eq!(handle.browser_session(), session);
    assert_eq!(handle.browsing_context(), context);
    assert_eq!(handle.origin(), &origin);
    assert_eq!(handle.document_epoch().value(), 1);
    assert_ne!(handle.node_id(), 0);

    let unregistered_same = values(ObservedNodeHandle::new(
        session,
        context,
        origin.clone(),
        handle.document_epoch(),
        handle.node_id(),
    ));
    assert_eq!(unregistered_same.len(), 1);
    let second_unregistered_same = values(ObservedNodeHandle::new(
        session,
        context,
        origin.clone(),
        handle.document_epoch(),
        handle.node_id(),
    ));
    assert_eq!(second_unregistered_same.len(), 1);
    assert_ne!(*handle, unregistered_same[0]);
    assert_eq!(unregistered_same[0], second_unregistered_same[0]);

    let unregistered_other = values(ObservedNodeHandle::new(
        session,
        context,
        origin.clone(),
        handle.document_epoch(),
        handle.node_id() + 1,
    ));
    assert_eq!(unregistered_other.len(), 1);
    assert_ne!(unregistered_same[0], unregistered_other[0]);

    assert_eq!(
        registry.validate_node_handle(&unregistered_same[0]),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert_eq!(registry.remove_node(handle), Ok(()));
    assert_eq!(
        registry.validate_node_handle(handle),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert_eq!(registry.remove_context(context), Ok(()));
    assert_eq!(
        registry.bind_node(session, context, &origin, "retired-context-node"),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );

    let display_cases = [
        BrowserRegistryError::InvalidExternalIdentifier,
        BrowserRegistryError::UnknownBrowserSession,
        BrowserRegistryError::UnknownBrowsingContext,
        BrowserRegistryError::ContextSessionMismatch {
            expected: session,
            actual: session,
        },
        BrowserRegistryError::OriginChangedWithoutDocumentAdvance,
        BrowserRegistryError::UnknownNodeAuthority,
        BrowserRegistryError::IdentifierSpaceExhausted,
        BrowserRegistryError::DocumentEpochExhausted,
        BrowserRegistryError::InternalAuthorityInvariant,
    ];
    for error in display_cases {
        assert!(!error.to_string().is_empty());
    }
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
