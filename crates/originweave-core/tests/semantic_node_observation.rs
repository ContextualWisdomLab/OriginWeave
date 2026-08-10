use std::collections::BTreeSet;

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, MAX_ACCESSIBLE_NAME_BYTES,
    MAX_SEMANTIC_CHILDREN, MAX_SEMANTIC_ROLE_BYTES, MAX_VISIBLE_TEXT_BYTES, NodeActionKind,
    ObservationChannel, ObservedNodeHandle, Origin, SemanticNodeObservation,
    SemanticNodeObservationError, SemanticNodeObservationInput,
};

fn observed_node_with_id(node_id: u64) -> Result<ObservedNodeHandle, String> {
    let browser_session = BrowserSessionId::new(7).map_err(|error| error.to_string())?;
    let browsing_context = BrowsingContextId::new(11).map_err(|error| error.to_string())?;
    let origin = Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?;
    let document_epoch = DocumentEpoch::new(3).map_err(|error| error.to_string())?;
    ObservedNodeHandle::new(
        browser_session,
        browsing_context,
        origin,
        document_epoch,
        node_id,
    )
    .map_err(|error| error.to_string())
}

fn observed_node() -> Result<ObservedNodeHandle, String> {
    observed_node_with_id(17)
}

fn semantic_input(
    role: String,
    accessible_name: String,
    visible_text: Option<String>,
) -> Result<SemanticNodeObservationInput, String> {
    Ok(SemanticNodeObservationInput {
        handle: observed_node()?,
        parent: None,
        children: Vec::new(),
        role,
        accessible_name,
        visible_text,
        enabled: true,
        visible: true,
        selected: None,
        supported_actions: BTreeSet::from([NodeActionKind::Click, NodeActionKind::TypeText]),
        evidence_channels: BTreeSet::from([
            ObservationChannel::Accessibility,
            ObservationChannel::Dom,
        ]),
    })
}

#[test]
fn semantic_node_preserves_authority_and_bounded_surface() -> Result<(), String> {
    let input = semantic_input(
        "textbox".to_owned(),
        "Email address".to_owned(),
        Some("name@example.test".to_owned()),
    )?;
    let handle = input.handle.clone();
    let observation = SemanticNodeObservation::new(input).map_err(|error| error.to_string())?;

    assert_eq!(observation.handle(), &handle);
    assert_eq!(observation.parent(), None);
    assert!(observation.children().is_empty());
    assert_eq!(observation.role(), "textbox");
    assert_eq!(observation.accessible_name(), "Email address");
    assert_eq!(observation.visible_text(), Some("name@example.test"));
    assert!(observation.is_enabled());
    assert!(observation.is_visible());
    assert_eq!(observation.is_selected(), None);
    assert_eq!(
        observation.supported_actions(),
        &BTreeSet::from([NodeActionKind::Click, NodeActionKind::TypeText])
    );
    assert_eq!(
        observation.evidence_channels(),
        &BTreeSet::from([ObservationChannel::Accessibility, ObservationChannel::Dom,])
    );
    Ok(())
}

#[test]
fn semantic_node_preserves_bounded_authority_scoped_relationships() -> Result<(), String> {
    let parent = observed_node_with_id(16)?;
    let first_child = observed_node_with_id(18)?;
    let second_child = observed_node_with_id(19)?;
    let mut input = semantic_input("group".to_owned(), "Account".to_owned(), None)?;
    input.parent = Some(parent.clone());
    input.children = vec![first_child.clone(), second_child.clone()];

    let observation = SemanticNodeObservation::new(input).map_err(|error| error.to_string())?;
    assert_eq!(observation.parent(), Some(&parent));
    assert_eq!(observation.children(), &[first_child, second_child]);
    Ok(())
}

#[test]
fn semantic_node_bounds_child_relationship_count() -> Result<(), String> {
    let mut boundary = semantic_input("list".to_owned(), "Items".to_owned(), None)?;
    boundary.children = (0..MAX_SEMANTIC_CHILDREN)
        .map(|offset| observed_node_with_id(100 + offset as u64))
        .collect::<Result<Vec<_>, _>>()?;
    let observation = SemanticNodeObservation::new(boundary).map_err(|error| error.to_string())?;
    assert_eq!(observation.children().len(), MAX_SEMANTIC_CHILDREN);

    let mut overflow = semantic_input("list".to_owned(), "Items".to_owned(), None)?;
    overflow.children = (0..=MAX_SEMANTIC_CHILDREN)
        .map(|offset| observed_node_with_id(1_000 + offset as u64))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        SemanticNodeObservation::new(overflow).err(),
        Some(SemanticNodeObservationError::TooManyChildren)
    );
    Ok(())
}

#[test]
fn semantic_node_rejects_relationships_outside_exact_authority() -> Result<(), String> {
    let mut input = semantic_input("group".to_owned(), "Account".to_owned(), None)?;
    let different_origin = Origin::parse("https://other.example")
        .map_err(|error| format!("{error:?}"))?;
    input.parent = Some(
        ObservedNodeHandle::new(
            BrowserSessionId::new(7).map_err(|error| error.to_string())?,
            BrowsingContextId::new(11).map_err(|error| error.to_string())?,
            different_origin,
            DocumentEpoch::new(3).map_err(|error| error.to_string())?,
            16,
        )
        .map_err(|error| error.to_string())?,
    );

    assert_eq!(
        SemanticNodeObservation::new(input).err(),
        Some(SemanticNodeObservationError::RelationshipAuthorityMismatch)
    );
    Ok(())
}

#[test]
fn semantic_node_rejects_self_and_duplicate_child_relationships() -> Result<(), String> {
    let mut self_parent = semantic_input("group".to_owned(), "Account".to_owned(), None)?;
    self_parent.parent = Some(self_parent.handle.clone());
    assert_eq!(
        SemanticNodeObservation::new(self_parent).err(),
        Some(SemanticNodeObservationError::SelfRelationship)
    );

    let mut self_child = semantic_input("group".to_owned(), "Account".to_owned(), None)?;
    self_child.children = vec![self_child.handle.clone()];
    assert_eq!(
        SemanticNodeObservation::new(self_child).err(),
        Some(SemanticNodeObservationError::SelfRelationship)
    );

    let child = observed_node_with_id(18)?;
    let mut duplicate = semantic_input("group".to_owned(), "Account".to_owned(), None)?;
    duplicate.children = vec![child.clone(), child];
    assert_eq!(
        SemanticNodeObservation::new(duplicate).err(),
        Some(SemanticNodeObservationError::DuplicateChild)
    );
    Ok(())
}

#[test]
fn reviewed_text_bounds_are_inclusive_and_visible_text_is_optional() -> Result<(), String> {
    let boundary = SemanticNodeObservation::new(semantic_input(
        "r".repeat(MAX_SEMANTIC_ROLE_BYTES),
        "n".repeat(MAX_ACCESSIBLE_NAME_BYTES),
        Some("v".repeat(MAX_VISIBLE_TEXT_BYTES)),
    )?)
    .map_err(|error| error.to_string())?;
    assert_eq!(boundary.role().len(), MAX_SEMANTIC_ROLE_BYTES);
    assert_eq!(boundary.accessible_name().len(), MAX_ACCESSIBLE_NAME_BYTES);
    assert_eq!(
        boundary.visible_text().map(str::len),
        Some(MAX_VISIBLE_TEXT_BYTES)
    );

    let without_text =
        SemanticNodeObservation::new(semantic_input("button".to_owned(), String::new(), None)?)
            .map_err(|error| error.to_string())?;
    assert_eq!(without_text.visible_text(), None);
    Ok(())
}

#[test]
fn semantic_node_requires_observation_provenance() -> Result<(), String> {
    let mut input = semantic_input("button".to_owned(), "Submit".to_owned(), None)?;
    input.evidence_channels.clear();

    let error = SemanticNodeObservation::new(input).err();
    assert_eq!(
        error,
        Some(SemanticNodeObservationError::MissingEvidenceChannel)
    );
    Ok(())
}

#[test]
fn semantic_node_rejects_unbounded_or_missing_role_text() -> Result<(), String> {
    let empty_role =
        SemanticNodeObservation::new(semantic_input(String::new(), "name".to_owned(), None)?).err();
    assert_eq!(empty_role, Some(SemanticNodeObservationError::EmptyRole));

    let long_role = SemanticNodeObservation::new(semantic_input(
        "r".repeat(MAX_SEMANTIC_ROLE_BYTES + 1),
        "name".to_owned(),
        None,
    )?)
    .err();
    assert_eq!(long_role, Some(SemanticNodeObservationError::RoleTooLong));

    let long_name = SemanticNodeObservation::new(semantic_input(
        "button".to_owned(),
        "n".repeat(MAX_ACCESSIBLE_NAME_BYTES + 1),
        None,
    )?)
    .err();
    assert_eq!(
        long_name,
        Some(SemanticNodeObservationError::AccessibleNameTooLong)
    );

    let long_visible_text = SemanticNodeObservation::new(semantic_input(
        "button".to_owned(),
        "name".to_owned(),
        Some("v".repeat(MAX_VISIBLE_TEXT_BYTES + 1)),
    )?)
    .err();
    assert_eq!(
        long_visible_text,
        Some(SemanticNodeObservationError::VisibleTextTooLong)
    );
    Ok(())
}

#[test]
fn semantic_node_errors_are_stable_and_credential_free() {
    assert_eq!(
        SemanticNodeObservationError::EmptyRole.to_string(),
        "semantic node role must not be empty"
    );
    assert_eq!(
        SemanticNodeObservationError::RoleTooLong.to_string(),
        "semantic node role exceeds 64 UTF-8 bytes"
    );
    assert_eq!(
        SemanticNodeObservationError::AccessibleNameTooLong.to_string(),
        "semantic node accessible name exceeds 512 UTF-8 bytes"
    );
    assert_eq!(
        SemanticNodeObservationError::VisibleTextTooLong.to_string(),
        "semantic node visible text exceeds 4096 UTF-8 bytes"
    );
    assert_eq!(
        SemanticNodeObservationError::MissingEvidenceChannel.to_string(),
        "semantic node observation requires at least one evidence channel"
    );
    assert_eq!(
        SemanticNodeObservationError::TooManyChildren.to_string(),
        "semantic node observation exceeds 128 child relationships"
    );
    assert_eq!(
        SemanticNodeObservationError::RelationshipAuthorityMismatch.to_string(),
        "semantic node relationship crosses its session, context, origin, or document authority"
    );
    assert_eq!(
        SemanticNodeObservationError::SelfRelationship.to_string(),
        "semantic node observation cannot relate the node to itself"
    );
    assert_eq!(
        SemanticNodeObservationError::DuplicateChild.to_string(),
        "semantic node observation contains a duplicate child relationship"
    );
}
