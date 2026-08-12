use std::collections::BTreeSet;

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeActionKind, ObservationChannel,
    ObservedNodeHandle, Origin, SemanticNodeActionTarget, SemanticNodeActionTargetError,
    SemanticNodeObservation, SemanticNodeObservationInput,
};

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn observation(
    node_id: u64,
    enabled: bool,
    supported_actions: BTreeSet<NodeActionKind>,
) -> Result<SemanticNodeObservation, String> {
    SemanticNodeObservation::new(SemanticNodeObservationInput {
        handle: ObservedNodeHandle::new(
            BrowserSessionId::new(7).map_err(|error| error.to_string())?,
            BrowsingContextId::new(11).map_err(|error| error.to_string())?,
            origin("https://app.example")?,
            DocumentEpoch::new(3).map_err(|error| error.to_string())?,
            node_id,
        )
        .map_err(|error| error.to_string())?,
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
    })
    .map_err(|error| error.to_string())
}

#[test]
fn current_semantic_observation_revalidates_exact_target_action_state() -> Result<(), String> {
    let initial = observation(17, true, BTreeSet::from([NodeActionKind::Click]))?;
    let target = SemanticNodeActionTarget::from_observation(&initial, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;

    let current = observation(17, true, BTreeSet::from([NodeActionKind::Click]))?;
    target
        .validate_current_observation(&current)
        .map_err(|error| error.to_string())?;

    let disabled = observation(17, false, BTreeSet::from([NodeActionKind::Click]))?;
    assert_eq!(
        target.validate_current_observation(&disabled),
        Err(SemanticNodeActionTargetError::NodeNotEnabled)
    );

    let action_removed = observation(17, true, BTreeSet::new())?;
    assert_eq!(
        target.validate_current_observation(&action_removed),
        Err(SemanticNodeActionTargetError::UnsupportedAction)
    );

    let other_node = observation(18, true, BTreeSet::from([NodeActionKind::Click]))?;
    assert_eq!(
        target.validate_current_observation(&other_node),
        Err(SemanticNodeActionTargetError::ObservationAuthorityMismatch)
    );
    Ok(())
}

#[test]
fn scroll_revalidation_preserves_non_enabled_scroll_boundary() -> Result<(), String> {
    let initial = observation(17, false, BTreeSet::from([NodeActionKind::ScrollIntoView]))?;
    let target =
        SemanticNodeActionTarget::from_observation(&initial, NodeActionKind::ScrollIntoView)
            .map_err(|error| error.to_string())?;
    let current = observation(17, false, BTreeSet::from([NodeActionKind::ScrollIntoView]))?;

    target
        .validate_current_observation(&current)
        .map_err(|error| error.to_string())?;
    Ok(())
}
