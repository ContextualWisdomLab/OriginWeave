use std::collections::BTreeSet;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserSessionId, BrowsingContextId, MAX_ACCESSIBLE_NAME_BYTES,
    MAX_SEMANTIC_CHILDREN, MAX_SEMANTIC_ROLE_BYTES, MAX_VISIBLE_TEXT_BYTES, NodeActionKind,
    ObservationChannel, ObservedNodeHandle, Origin, SemanticNodeObservation,
    SemanticNodeObservationError, SemanticNodeObservationInput,
};

struct Fixture {
    registry: BrowserAuthorityRegistry,
    session: BrowserSessionId,
    context: BrowsingContextId,
    origin: Origin,
    next_external: u64,
}

impl Fixture {
    fn new() -> Result<Self, String> {
        let mut registry = BrowserAuthorityRegistry::new();
        let session = registry
            .register_session("semantic-session")
            .map_err(|error| error.to_string())?;
        let context = registry
            .register_context(session, "semantic-context")
            .map_err(|error| error.to_string())?;
        let origin = Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?;
        Ok(Self {
            registry,
            session,
            context,
            origin,
            next_external: 1,
        })
    }

    fn bind_named(&mut self, external_identifier: &str) -> Result<ObservedNodeHandle, String> {
        self.registry
            .bind_node(
                self.session,
                self.context,
                &self.origin,
                external_identifier,
            )
            .map_err(|error| error.to_string())
    }

    fn bind_next(&mut self) -> Result<ObservedNodeHandle, String> {
        let external_identifier = format!("semantic-node-{}", self.next_external);
        self.next_external += 1;
        self.bind_named(&external_identifier)
    }

    fn input(
        &mut self,
        role: String,
        accessible_name: String,
        visible_text: Option<String>,
    ) -> Result<SemanticNodeObservationInput, String> {
        Ok(SemanticNodeObservationInput {
            handle: self.bind_next()?,
            parent: None,
            children: Vec::new(),
            role,
            accessible_name,
            visible_text,
            enabled: true,
            visible: true,
            selected: None,
            supported_actions: BTreeSet::from([
                NodeActionKind::Click,
                NodeActionKind::TypeText,
            ]),
            evidence_channels: BTreeSet::from([
                ObservationChannel::Accessibility,
                ObservationChannel::Dom,
            ]),
        })
    }
}

#[test]
fn semantic_node_preserves_live_authority_and_bounded_surface() -> Result<(), String> {
    let mut fixture = Fixture::new()?;
    let mut input = fixture.input(
        "textbox".to_owned(),
        "Email address".to_owned(),
        Some("name@example.test".to_owned()),
    )?;
    let handle = input.handle.clone();
    let parent = fixture.bind_next()?;
    let first_child = fixture.bind_next()?;
    let second_child = fixture.bind_next()?;
    input.parent = Some(parent.clone());
    input.children = vec![first_child.clone(), second_child.clone()];
    input.selected = Some(false);

    let observation = SemanticNodeObservation::new(input, &fixture.registry)
        .map_err(|error| error.to_string())?;

    assert_eq!(observation.handle(), &handle);
    assert_eq!(observation.parent(), Some(&parent));
    assert_eq!(observation.children(), &[first_child, second_child]);
    assert_eq!(observation.role(), "textbox");
    assert_eq!(observation.accessible_name(), "Email address");
    assert_eq!(observation.visible_text(), Some("name@example.test"));
    assert!(observation.is_enabled());
    assert!(observation.is_visible());
    assert_eq!(observation.is_selected(), Some(false));
    assert_eq!(
        observation.supported_actions(),
        &BTreeSet::from([NodeActionKind::Click, NodeActionKind::TypeText])
    );
    assert_eq!(
        observation.evidence_channels(),
        &BTreeSet::from([ObservationChannel::Accessibility, ObservationChannel::Dom])
    );
    Ok(())
}

#[test]
fn semantic_node_bounds_child_relationship_count() -> Result<(), String> {
    let mut fixture = Fixture::new()?;
    let mut boundary = fixture.input("list".to_owned(), "Items".to_owned(), None)?;
    let mut children = Vec::with_capacity(MAX_SEMANTIC_CHILDREN);
    for _ in 0..MAX_SEMANTIC_CHILDREN {
        children.push(fixture.bind_next()?);
    }
    boundary.children = children;
    let observation = SemanticNodeObservation::new(boundary, &fixture.registry)
        .map_err(|error| error.to_string())?;
    assert_eq!(observation.children().len(), MAX_SEMANTIC_CHILDREN);

    let mut overflow = fixture.input("list".to_owned(), "Items".to_owned(), None)?;
    overflow.children = vec![overflow.handle.clone(); MAX_SEMANTIC_CHILDREN + 1];
    assert_eq!(
        SemanticNodeObservation::new(overflow, &fixture.registry).err(),
        Some(SemanticNodeObservationError::TooManyChildren)
    );
    Ok(())
}

#[test]
fn semantic_node_rejects_live_relationships_from_other_authority() -> Result<(), String> {
    let mut fixture = Fixture::new()?;
    let mut parent_input = fixture.input("group".to_owned(), "Account".to_owned(), None)?;
    let other_context = fixture
        .registry
        .register_context(fixture.session, "other-context")
        .map_err(|error| error.to_string())?;
    let origin = fixture.origin.clone();
    let other_context_node = fixture
        .registry
        .bind_node(fixture.session, other_context, &origin, "other-context-node")
        .map_err(|error| error.to_string())?;
    parent_input.parent = Some(other_context_node);
    assert_eq!(
        SemanticNodeObservation::new(parent_input, &fixture.registry).err(),
        Some(SemanticNodeObservationError::RelationshipAuthorityMismatch)
    );

    let mut child_input = fixture.input("group".to_owned(), "Account".to_owned(), None)?;
    let other_session = fixture
        .registry
        .register_session("other-session")
        .map_err(|error| error.to_string())?;
    let other_session_context = fixture
        .registry
        .register_context(other_session, "other-session-context")
        .map_err(|error| error.to_string())?;
    let other_origin = Origin::parse("https://other.example")
        .map_err(|error| format!("{error:?}"))?;
    let other_session_node = fixture
        .registry
        .bind_node(
            other_session,
            other_session_context,
            &other_origin,
            "other-session-node",
        )
        .map_err(|error| error.to_string())?;
    child_input.children = vec![other_session_node];
    assert_eq!(
        SemanticNodeObservation::new(child_input, &fixture.registry).err(),
        Some(SemanticNodeObservationError::RelationshipAuthorityMismatch)
    );
    Ok(())
}

#[test]
fn semantic_node_rejects_self_and_duplicate_child_relationships() -> Result<(), String> {
    let mut fixture = Fixture::new()?;
    let mut self_parent = fixture.input("group".to_owned(), "Account".to_owned(), None)?;
    self_parent.parent = Some(self_parent.handle.clone());
    assert_eq!(
        SemanticNodeObservation::new(self_parent, &fixture.registry).err(),
        Some(SemanticNodeObservationError::SelfRelationship)
    );

    let mut self_child = fixture.input("group".to_owned(), "Account".to_owned(), None)?;
    self_child.children = vec![self_child.handle.clone()];
    assert_eq!(
        SemanticNodeObservation::new(self_child, &fixture.registry).err(),
        Some(SemanticNodeObservationError::SelfRelationship)
    );

    let child = fixture.bind_next()?;
    let mut duplicate = fixture.input("group".to_owned(), "Account".to_owned(), None)?;
    duplicate.children = vec![child.clone(), child];
    assert_eq!(
        SemanticNodeObservation::new(duplicate, &fixture.registry).err(),
        Some(SemanticNodeObservationError::DuplicateChild)
    );
    Ok(())
}

#[test]
fn reviewed_text_bounds_are_inclusive_and_visible_text_is_optional() -> Result<(), String> {
    let mut fixture = Fixture::new()?;
    let boundary_input = fixture.input(
        "r".repeat(MAX_SEMANTIC_ROLE_BYTES),
        "n".repeat(MAX_ACCESSIBLE_NAME_BYTES),
        Some("v".repeat(MAX_VISIBLE_TEXT_BYTES)),
    )?;
    let boundary = SemanticNodeObservation::new(boundary_input, &fixture.registry)
        .map_err(|error| error.to_string())?;
    assert_eq!(boundary.role().len(), MAX_SEMANTIC_ROLE_BYTES);
    assert_eq!(boundary.accessible_name().len(), MAX_ACCESSIBLE_NAME_BYTES);
    assert_eq!(
        boundary.visible_text().map(str::len),
        Some(MAX_VISIBLE_TEXT_BYTES)
    );

    let without_text_input = fixture.input("button".to_owned(), String::new(), None)?;
    let without_text = SemanticNodeObservation::new(without_text_input, &fixture.registry)
        .map_err(|error| error.to_string())?;
    assert_eq!(without_text.visible_text(), None);
    Ok(())
}

#[test]
fn semantic_node_rejects_missing_provenance_and_unbounded_text() -> Result<(), String> {
    let mut fixture = Fixture::new()?;

    let mut missing_provenance =
        fixture.input("button".to_owned(), "Submit".to_owned(), None)?;
    missing_provenance.evidence_channels.clear();
    assert_eq!(
        SemanticNodeObservation::new(missing_provenance, &fixture.registry).err(),
        Some(SemanticNodeObservationError::MissingEvidenceChannel)
    );

    let empty_role = fixture.input(String::new(), "name".to_owned(), None)?;
    assert_eq!(
        SemanticNodeObservation::new(empty_role, &fixture.registry).err(),
        Some(SemanticNodeObservationError::EmptyRole)
    );

    let long_role = fixture.input(
        "r".repeat(MAX_SEMANTIC_ROLE_BYTES + 1),
        "name".to_owned(),
        None,
    )?;
    assert_eq!(
        SemanticNodeObservation::new(long_role, &fixture.registry).err(),
        Some(SemanticNodeObservationError::RoleTooLong)
    );

    let long_name = fixture.input(
        "button".to_owned(),
        "n".repeat(MAX_ACCESSIBLE_NAME_BYTES + 1),
        None,
    )?;
    assert_eq!(
        SemanticNodeObservation::new(long_name, &fixture.registry).err(),
        Some(SemanticNodeObservationError::AccessibleNameTooLong)
    );

    let long_visible_text = fixture.input(
        "button".to_owned(),
        "name".to_owned(),
        Some("v".repeat(MAX_VISIBLE_TEXT_BYTES + 1)),
    )?;
    assert_eq!(
        SemanticNodeObservation::new(long_visible_text, &fixture.registry).err(),
        Some(SemanticNodeObservationError::VisibleTextTooLong)
    );
    Ok(())
}

#[test]
fn semantic_node_errors_are_stable_and_credential_free() {
    let expected = [
        (
            SemanticNodeObservationError::EmptyRole,
            "semantic node role must not be empty",
        ),
        (
            SemanticNodeObservationError::RoleTooLong,
            "semantic node role exceeds 64 UTF-8 bytes",
        ),
        (
            SemanticNodeObservationError::AccessibleNameTooLong,
            "semantic node accessible name exceeds 512 UTF-8 bytes",
        ),
        (
            SemanticNodeObservationError::VisibleTextTooLong,
            "semantic node visible text exceeds 4096 UTF-8 bytes",
        ),
        (
            SemanticNodeObservationError::MissingEvidenceChannel,
            "semantic node observation requires at least one evidence channel",
        ),
        (
            SemanticNodeObservationError::TooManyChildren,
            "semantic node observation exceeds 128 child relationships",
        ),
        (
            SemanticNodeObservationError::UnknownNodeAuthority,
            "semantic node observation contains node authority not owned by the active browser registry",
        ),
        (
            SemanticNodeObservationError::RelationshipAuthorityMismatch,
            "semantic node relationship crosses its session, context, origin, or document authority",
        ),
        (
            SemanticNodeObservationError::SelfRelationship,
            "semantic node observation cannot relate the node to itself",
        ),
        (
            SemanticNodeObservationError::DuplicateChild,
            "semantic node observation contains a duplicate child relationship",
        ),
    ];

    for (error, message) in expected {
        assert_eq!(error.to_string(), message);
    }
}
