use std::collections::BTreeSet;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, BrowsingContextId,
    NodeActionKind, ObservationChannel, Origin, SemanticNodeActionTarget,
    SemanticNodeActionTargetError, SemanticNodeObservation, SemanticNodeObservationInput,
};

struct ObservationFixture {
    registry: BrowserAuthorityRegistry,
    session: BrowserSessionId,
    context: BrowsingContextId,
    observation: SemanticNodeObservation,
}

fn observation_fixture() -> Result<ObservationFixture, String> {
    observation_fixture_with_enabled(true)
}

fn observation_fixture_with_enabled(enabled: bool) -> Result<ObservationFixture, String> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("semantic-action-session")
        .map_err(|error| error.to_string())?;
    let context = registry
        .register_context(session, "semantic-action-context")
        .map_err(|error| error.to_string())?;
    let origin = Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?;
    let handle = registry
        .bind_node(session, context, &origin, "semantic-action-node")
        .map_err(|error| error.to_string())?;
    let observation = SemanticNodeObservation::new(
        SemanticNodeObservationInput {
            handle,
            parent: None,
            children: Vec::new(),
            role: "button".to_owned(),
            accessible_name: "Save draft".to_owned(),
            visible_text: Some("Save draft".to_owned()),
            enabled,
            visible: true,
            selected: None,
            supported_actions: BTreeSet::from([
                NodeActionKind::Click,
                NodeActionKind::ScrollIntoView,
            ]),
            evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
        },
        &registry,
    )
    .map_err(|error| error.to_string())?;

    Ok(ObservationFixture {
        registry,
        session,
        context,
        observation,
    })
}

#[test]
fn advertised_node_action_becomes_an_authority_bound_target() -> Result<(), String> {
    let fixture = observation_fixture()?;
    let target =
        SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
            .map_err(|error| error.to_string())?;

    assert_eq!(target.handle(), fixture.observation.handle());
    assert_eq!(target.action(), NodeActionKind::Click);
    Ok(())
}

#[test]
fn unsupported_node_action_fails_closed_without_minting_authority() -> Result<(), String> {
    let fixture = observation_fixture()?;
    assert_eq!(
        SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::TypeText)
            .err(),
        Some(SemanticNodeActionTargetError::UnsupportedAction)
    );
    Ok(())
}

#[test]
fn disabled_interactive_node_action_fails_closed() -> Result<(), String> {
    let fixture = observation_fixture_with_enabled(false)?;
    assert_eq!(
        SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
            .err(),
        Some(SemanticNodeActionTargetError::NodeNotEnabled)
    );
    Ok(())
}

#[test]
fn disabled_node_can_still_be_targeted_for_scroll_only() -> Result<(), String> {
    let fixture = observation_fixture_with_enabled(false)?;
    let target = SemanticNodeActionTarget::from_observation(
        &fixture.observation,
        NodeActionKind::ScrollIntoView,
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(target.action(), NodeActionKind::ScrollIntoView);
    Ok(())
}

#[test]
fn node_action_target_revalidates_live_registry_authority() -> Result<(), String> {
    let fixture = observation_fixture()?;
    let target =
        SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
            .map_err(|error| error.to_string())?;

    target
        .validate_current(&fixture.registry)
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn node_action_target_rejects_retired_context_authority() -> Result<(), String> {
    let mut fixture = observation_fixture()?;
    let target =
        SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
            .map_err(|error| error.to_string())?;
    fixture
        .registry
        .remove_context(fixture.context)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        target.validate_current(&fixture.registry).err(),
        Some(BrowserRegistryError::UnknownBrowsingContext)
    );
    Ok(())
}

#[test]
fn node_action_target_rejects_retired_session_authority() -> Result<(), String> {
    let mut fixture = observation_fixture()?;
    let target =
        SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
            .map_err(|error| error.to_string())?;
    fixture
        .registry
        .remove_session(fixture.session)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        target.validate_current(&fixture.registry).err(),
        Some(BrowserRegistryError::UnknownBrowserSession)
    );
    Ok(())
}

#[test]
fn node_action_target_rejects_stale_document_authority() -> Result<(), String> {
    let mut fixture = observation_fixture()?;
    let target =
        SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
            .map_err(|error| error.to_string())?;
    fixture
        .registry
        .advance_document(fixture.context)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        target.validate_current(&fixture.registry).err(),
        Some(BrowserRegistryError::UnknownNodeAuthority)
    );
    Ok(())
}

#[test]
fn node_action_target_error_is_stable_and_credential_free() {
    assert_eq!(
        SemanticNodeActionTargetError::UnsupportedAction.to_string(),
        "semantic node action is not advertised by the observation"
    );
    assert_eq!(
        SemanticNodeActionTargetError::NodeNotEnabled.to_string(),
        "semantic node is not enabled for the requested action"
    );
}
