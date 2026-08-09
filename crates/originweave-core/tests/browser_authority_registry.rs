use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, NodeHandleError, Origin,
};

fn loopback_origin() -> Origin {
    Origin::parse("http://127.0.0.1:43127").expect("controlled loopback origin")
}

#[test]
fn external_protocol_identifiers_are_scoped_and_never_become_authority() {
    let mut registry = BrowserAuthorityRegistry::new();

    let first_session = registry
        .register_session("webdriver-session-A")
        .expect("first session must register");
    let repeated_session = registry
        .register_session("webdriver-session-A")
        .expect("same external session must resolve consistently");
    let second_session = registry
        .register_session("webdriver-session-B")
        .expect("second session must register");

    assert_eq!(first_session, repeated_session);
    assert_ne!(first_session, second_session);

    let first_context = registry
        .register_context(first_session, "frame-root")
        .expect("first context must register");
    let second_context = registry
        .register_context(second_session, "frame-root")
        .expect("the same adapter context string is session scoped");

    assert_ne!(first_context, second_context);
    assert_eq!(
        registry
            .current_epoch(first_context)
            .expect("known context"),
        originweave_core::DocumentEpoch::new(1).expect("nonzero epoch")
    );
}

#[test]
fn document_rotation_invalidates_old_external_node_bindings() {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("webdriver-session")
        .expect("session must register");
    let context = registry
        .register_context(session, "top-level-context")
        .expect("context must register");
    let origin = loopback_origin();

    let first = registry
        .bind_node(session, context, &origin, "backend-node-17")
        .expect("node must bind");
    let same = registry
        .bind_node(session, context, &origin, "backend-node-17")
        .expect("same node in same document must be stable");
    assert_eq!(first.node_id(), same.node_id());

    let next_epoch = registry
        .advance_document(context)
        .expect("navigation must advance the document epoch");
    assert_eq!(next_epoch.value(), 2);
    assert_eq!(
        first.validate_current(session, context, &origin, next_epoch),
        Err(NodeHandleError::StaleDocumentEpoch {
            observed: first.document_epoch(),
            current: next_epoch,
        })
    );

    let rebound = registry
        .bind_node(session, context, &origin, "backend-node-17")
        .expect("adapter node identifiers may be reused only in the new epoch");
    assert_eq!(rebound.document_epoch(), next_epoch);
    assert_ne!(first.node_id(), rebound.node_id());
}

#[test]
fn context_cannot_be_reused_by_another_session() {
    let mut registry = BrowserAuthorityRegistry::new();
    let owner = registry
        .register_session("owner-session")
        .expect("owner session must register");
    let attacker = registry
        .register_session("attacker-session")
        .expect("second session must register");
    let context = registry
        .register_context(owner, "shared-looking-context")
        .expect("owner context must register");

    let error = registry
        .bind_node(attacker, context, &loopback_origin(), "node")
        .expect_err("cross-session context reuse must fail closed");
    assert_eq!(
        error,
        BrowserRegistryError::ContextSessionMismatch {
            expected: owner,
            actual: attacker,
        }
    );
}

#[test]
fn external_identifiers_are_bounded_without_assuming_protocol_syntax() {
    let mut registry = BrowserAuthorityRegistry::new();

    assert_eq!(
        registry.register_session(""),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    assert_eq!(
        registry.register_session(&"x".repeat(513)),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );

    let unicode = registry
        .register_session("세션-opaque-✓")
        .expect("opaque protocol identifiers may contain bounded Unicode");
    assert!(unicode.value() > 0);
}

#[test]
fn unknown_internal_authority_is_rejected_before_node_binding() {
    let mut registry = BrowserAuthorityRegistry::new();
    let unknown = BrowserSessionId::new(999).expect("nonzero internal identifier");

    assert_eq!(
        registry.register_context(unknown, "context"),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );
}
