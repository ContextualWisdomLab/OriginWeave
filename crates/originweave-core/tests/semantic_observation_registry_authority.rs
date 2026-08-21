use std::collections::BTreeSet;

use originweave_core::{
    BrowserAuthorityRegistry, NodeActionKind, ObservationChannel, ObservedNodeHandle, Origin,
    SemanticNodeObservation, SemanticNodeObservationError, SemanticNodeObservationInput,
};

fn bound_observation_fixture() -> Result<
    (BrowserAuthorityRegistry, ObservedNodeHandle),
    Box<dyn std::error::Error>,
> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("semantic-session")?;
    let context = registry.register_context(session, "semantic-context")?;
    let origin = Origin::parse("https://example.com")?;
    let handle = registry.bind_node(session, context, &origin, "semantic-node")?;
    Ok((registry, handle))
}

fn input(handle: ObservedNodeHandle) -> SemanticNodeObservationInput {
    SemanticNodeObservationInput {
        handle,
        parent: None,
        children: Vec::new(),
        role: "button".to_owned(),
        accessible_name: "Submit".to_owned(),
        visible_text: None,
        enabled: true,
        visible: true,
        selected: None,
        supported_actions: BTreeSet::from([NodeActionKind::Click]),
        evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
    }
}

#[test]
fn semantic_observation_rejects_forged_primary_node_authority() -> Result<(), Box<dyn std::error::Error>> {
    let (registry, bound) = bound_observation_fixture()?;
    let forged = ObservedNodeHandle::new(
        bound.browser_session(),
        bound.browsing_context(),
        bound.origin().clone(),
        bound.document_epoch(),
        bound.node_id() + 10_000,
    )?;

    assert_eq!(
        SemanticNodeObservation::new(input(forged), &registry).err(),
        Some(SemanticNodeObservationError::UnknownNodeAuthority)
    );
    Ok(())
}

#[test]
fn semantic_observation_rejects_forged_related_node_authority() -> Result<(), Box<dyn std::error::Error>> {
    let (registry, bound) = bound_observation_fixture()?;
    let forged_child = ObservedNodeHandle::new(
        bound.browser_session(),
        bound.browsing_context(),
        bound.origin().clone(),
        bound.document_epoch(),
        bound.node_id() + 10_000,
    )?;
    let mut observation_input = input(bound);
    observation_input.children.push(forged_child);

    assert_eq!(
        SemanticNodeObservation::new(observation_input, &registry).err(),
        Some(SemanticNodeObservationError::UnknownNodeAuthority)
    );
    Ok(())
}
