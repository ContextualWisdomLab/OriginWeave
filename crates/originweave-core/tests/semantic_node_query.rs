use std::collections::BTreeSet;

use originweave_core::{
    BrowserAuthorityRegistry, MAX_ACCESSIBLE_NAME_BYTES, MAX_SEMANTIC_ROLE_BYTES, NodeActionKind,
    ObservationChannel, Origin, SemanticNodeObservation, SemanticNodeObservationInput,
    SemanticNodeQuery, SemanticNodeQueryError,
};

fn observation() -> Result<SemanticNodeObservation, String> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("semantic-query-session")
        .map_err(|error| error.to_string())?;
    let context = registry
        .register_context(session, "semantic-query-context")
        .map_err(|error| error.to_string())?;
    let origin = Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?;
    let handle = registry
        .bind_node(session, context, &origin, "semantic-query-node")
        .map_err(|error| error.to_string())?;

    SemanticNodeObservation::new(
        SemanticNodeObservationInput {
            handle,
            parent: None,
            children: Vec::new(),
            role: "textbox".to_owned(),
            accessible_name: "Email address".to_owned(),
            visible_text: Some("name@example.test".to_owned()),
            enabled: true,
            visible: true,
            selected: None,
            supported_actions: BTreeSet::from([NodeActionKind::Click, NodeActionKind::TypeText]),
            evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
        },
        &registry,
    )
    .map_err(|error| error.to_string())
}

#[test]
fn semantic_node_query_matches_exact_reviewed_fields_and_action() -> Result<(), String> {
    let observed = observation()?;
    let query = SemanticNodeQuery::new(
        Some("textbox".to_owned()),
        Some("Email address".to_owned()),
        Some(NodeActionKind::TypeText),
    )
    .map_err(|error| error.to_string())?;

    assert!(query.matches(&observed));
    assert_eq!(query.role(), Some("textbox"));
    assert_eq!(query.accessible_name(), Some("Email address"));
    assert_eq!(query.required_action(), Some(NodeActionKind::TypeText));
    Ok(())
}

#[test]
fn semantic_node_query_fails_closed_on_each_exact_selector_mismatch() -> Result<(), String> {
    let observed = observation()?;
    let cases = [
        SemanticNodeQuery::new(Some("button".to_owned()), None, None),
        SemanticNodeQuery::new(None, Some("Different label".to_owned()), None),
        SemanticNodeQuery::new(None, None, Some(NodeActionKind::SelectOption)),
    ];

    for query in cases {
        let query = query.map_err(|error| error.to_string())?;
        assert!(!query.matches(&observed));
    }
    Ok(())
}

#[test]
fn semantic_node_query_requires_at_least_one_selector() {
    assert_eq!(
        SemanticNodeQuery::new(None, None, None).err(),
        Some(SemanticNodeQueryError::EmptySelector)
    );
}

#[test]
fn semantic_node_query_bounds_attacker_controlled_text() {
    assert_eq!(
        SemanticNodeQuery::new(Some("r".repeat(MAX_SEMANTIC_ROLE_BYTES + 1)), None, None).err(),
        Some(SemanticNodeQueryError::RoleTooLong)
    );
    assert_eq!(
        SemanticNodeQuery::new(None, Some("n".repeat(MAX_ACCESSIBLE_NAME_BYTES + 1)), None,).err(),
        Some(SemanticNodeQueryError::AccessibleNameTooLong)
    );
}

#[test]
fn semantic_node_query_errors_are_stable_and_credential_free() {
    assert_eq!(
        SemanticNodeQueryError::EmptySelector.to_string(),
        "semantic node query requires at least one selector"
    );
    assert_eq!(
        SemanticNodeQueryError::RoleTooLong.to_string(),
        "semantic node query role exceeds 64 UTF-8 bytes"
    );
    assert_eq!(
        SemanticNodeQueryError::AccessibleNameTooLong.to_string(),
        "semantic node query accessible name exceeds 512 UTF-8 bytes"
    );
}
