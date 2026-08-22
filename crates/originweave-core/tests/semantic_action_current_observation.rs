use std::collections::BTreeSet;

use originweave_core::{
    BrowserAuthorityRegistry, NodeActionKind, ObservationChannel, ObservedNodeHandle, Origin,
    SemanticNodeActionTarget, SemanticNodeActionTargetError, SemanticNodeObservation,
    SemanticNodeObservationInput,
};

struct ObservationAuthorityFixture {
    registry: BrowserAuthorityRegistry,
    handle: ObservedNodeHandle,
    other_handle: ObservedNodeHandle,
}

fn authority_fixture() -> Result<ObservationAuthorityFixture, String> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("current-observation-session")
        .map_err(|error| error.to_string())?;
    let context = registry
        .register_context(session, "current-observation-context")
        .map_err(|error| error.to_string())?;
    let origin = Origin::parse("https://app.example").map_err(|error| format!("{error:?}"))?;
    let handle = registry
        .bind_node(session, context, &origin, "current-observation-node")
        .map_err(|error| error.to_string())?;
    let other_handle = registry
        .bind_node(session, context, &origin, "other-current-observation-node")
        .map_err(|error| error.to_string())?;
    Ok(ObservationAuthorityFixture {
        registry,
        handle,
        other_handle,
    })
}

fn observation(
    registry: &BrowserAuthorityRegistry,
    handle: ObservedNodeHandle,
    enabled: bool,
    supported_actions: BTreeSet<NodeActionKind>,
) -> Result<SemanticNodeObservation, String> {
    SemanticNodeObservation::new(
        SemanticNodeObservationInput {
            handle,
            parent: None,
            children: Vec::new(),
            role: "button".to_owned(),
            accessible_name: "Continue".to_owned(),
            visible_text: Some("Continue".to_owned()),
            enabled,
            visible: true,
            selected: None,
            supported_actions,
            evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
        },
        registry,
    )
    .map_err(|error| error.to_string())
}

#[test]
fn current_semantic_observation_revalidates_exact_target_action_state() -> Result<(), String> {
    let fixture = authority_fixture()?;
    let initial = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    let target = SemanticNodeActionTarget::from_observation(&initial, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;

    let current = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    target
        .validate_current_observation(&current)
        .map_err(|error| error.to_string())?;

    let disabled = observation(
        &fixture.registry,
        fixture.handle.clone(),
        false,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    assert_eq!(
        target.validate_current_observation(&disabled),
        Err(SemanticNodeActionTargetError::NodeNotEnabled)
    );

    let action_removed = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::new(),
    )?;
    assert_eq!(
        target.validate_current_observation(&action_removed),
        Err(SemanticNodeActionTargetError::UnsupportedAction)
    );

    let other_node = observation(
        &fixture.registry,
        fixture.other_handle,
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    assert_eq!(
        target.validate_current_observation(&other_node),
        Err(SemanticNodeActionTargetError::ObservationAuthorityMismatch)
    );
    Ok(())
}

#[test]
fn scroll_revalidation_preserves_non_enabled_scroll_boundary() -> Result<(), String> {
    let fixture = authority_fixture()?;
    let initial = observation(
        &fixture.registry,
        fixture.handle.clone(),
        false,
        BTreeSet::from([NodeActionKind::ScrollIntoView]),
    )?;
    let target =
        SemanticNodeActionTarget::from_observation(&initial, NodeActionKind::ScrollIntoView)
            .map_err(|error| error.to_string())?;
    let current = observation(
        &fixture.registry,
        fixture.handle,
        false,
        BTreeSet::from([NodeActionKind::ScrollIntoView]),
    )?;

    target
        .validate_current_observation(&current)
        .map_err(|error| error.to_string())?;
    Ok(())
}
