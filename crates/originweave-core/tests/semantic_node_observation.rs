use std::collections::BTreeSet;

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, MAX_ACCESSIBLE_NAME_BYTES,
    MAX_SEMANTIC_ROLE_BYTES, MAX_VISIBLE_TEXT_BYTES, NodeActionKind, ObservationChannel,
    ObservedNodeHandle, Origin, SemanticNodeObservation, SemanticNodeObservationError,
    SemanticNodeObservationInput,
};

fn observed_node() -> Result<ObservedNodeHandle, String> {
    let browser_session = BrowserSessionId::new(7).map_err(|error| error.to_string())?;
    let browsing_context = BrowsingContextId::new(11).map_err(|error| error.to_string())?;
    let origin = Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?;
    let document_epoch = DocumentEpoch::new(3).map_err(|error| error.to_string())?;
    ObservedNodeHandle::new(
        browser_session,
        browsing_context,
        origin,
        document_epoch,
        17,
    )
    .map_err(|error| error.to_string())
}

fn semantic_input(
    role: String,
    accessible_name: String,
    visible_text: Option<String>,
) -> Result<SemanticNodeObservationInput, String> {
    Ok(SemanticNodeObservationInput {
        handle: observed_node()?,
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
}
