use crate::browser_registry::{BrowserAuthorityRegistry, ObservedNodeHandle};
use crate::{BrowserRegistryError, BrowserSessionId, DocumentEpoch, Origin};

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

    let first = registry
        .bind_nodes(session, context, origin, &["unit-node"])
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let repeated = registry
        .bind_nodes(session, context, origin, &["unit-node"])
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
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
        Err(BrowserRegistryError::UnknownNodeAuthority)
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
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
}

#[test]
fn issued_handles_report_retired_authority_boundaries() {
    let origins = values(Origin::parse("http://127.0.0.1:43127"));
    assert_eq!(origins.len(), 1);
    let origin = &origins[0];

    let mut context_registry = BrowserAuthorityRegistry::new();
    let context_sessions = values(context_registry.register_session("context-retirement-session"));
    assert_eq!(context_sessions.len(), 1);
    let context_session = context_sessions[0];
    let contexts =
        values(context_registry.register_context(context_session, "context-retirement-context"));
    assert_eq!(contexts.len(), 1);
    let context = contexts[0];
    let context_handles = values(context_registry.bind_node(
        context_session,
        context,
        origin,
        "context-retirement-node",
    ));
    assert_eq!(context_handles.len(), 1);
    let context_handle = &context_handles[0];
    assert_eq!(context_registry.remove_context(context), Ok(()));
    assert_eq!(
        context_registry.validate_node_handle(context_handle),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );

    let mut session_registry = BrowserAuthorityRegistry::new();
    let sessions = values(session_registry.register_session("session-retirement-session"));
    assert_eq!(sessions.len(), 1);
    let session = sessions[0];
    let session_contexts =
        values(session_registry.register_context(session, "session-retirement-context"));
    assert_eq!(session_contexts.len(), 1);
    let session_context = session_contexts[0];
    let session_handles = values(session_registry.bind_node(
        session,
        session_context,
        origin,
        "session-retirement-node",
    ));
    assert_eq!(session_handles.len(), 1);
    let session_handle = &session_handles[0];
    assert_eq!(session_registry.remove_session(session), Ok(()));
    assert_eq!(
        session_registry.validate_node_handle(session_handle),
        Err(BrowserRegistryError::UnknownBrowserSession)
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
        registry.bind_nodes(attacker, context, &origins[0], &["unit-node"]),
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
fn failed_node_allocation_does_not_bind_context_origin() {
    let mut registry = BrowserAuthorityRegistry::with_identifier_limit(2);
    let session = values(registry.register_session("allocation-session"))[0];
    let exhausted_context = values(registry.register_context(session, "exhausted-context"))[0];
    let clean_context = values(registry.register_context(session, "clean-context"))[0];
    let first_origin = values(Origin::parse("http://127.0.0.1:43127"))[0].clone();
    let second_origin = values(Origin::parse("http://localhost:43127"))[0].clone();

    assert_eq!(
        values(registry.bind_node(session, exhausted_context, &first_origin, "node-one")).len(),
        1
    );
    assert_eq!(
        values(registry.bind_node(session, exhausted_context, &first_origin, "node-two")).len(),
        1
    );
    assert_eq!(
        registry.bind_node(session, clean_context, &first_origin, "node-three"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );
    assert_eq!(
        registry.bind_node(session, clean_context, &second_origin, "node-three"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );
}

#[test]
fn node_handles_cannot_cross_registry_instances() {
    let origin = values(Origin::parse("http://127.0.0.1:43127"))[0].clone();
    let mut first = BrowserAuthorityRegistry::new();
    let first_session = values(first.register_session("first-session"))[0];
    let first_context = values(first.register_context(first_session, "first-context"))[0];
    let first_handle =
        values(first.bind_node(first_session, first_context, &origin, "first-node"))[0].clone();

    let mut second = BrowserAuthorityRegistry::new();
    let second_session = values(second.register_session("second-session"))[0];
    let second_context = values(second.register_context(second_session, "second-context"))[0];
    let second_handle =
        values(second.bind_node(second_session, second_context, &origin, "second-node"))[0].clone();

    assert_eq!(first_session, second_session);
    assert_eq!(first_context, second_context);
    assert_eq!(first_handle.node_id(), second_handle.node_id());
    assert_ne!(first_handle, second_handle);
    assert_eq!(
        second.validate_node_handle(&first_handle),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert_eq!(second.validate_node_handle(&second_handle), Ok(()));
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
